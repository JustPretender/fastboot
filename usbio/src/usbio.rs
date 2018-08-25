extern crate libusb;

use std;
use std::io::{Error, ErrorKind, Read, Result, Write};
use std::option::Option;
use std::time::Duration;
use usbio::libusb::{Context, DeviceHandle, Direction, TransferType};

macro_rules! iocall {
    ($ex: expr) => {
        $ex.map_err(|e| err_to_io_err(e))
    };
}

pub struct UsbContext {
    context: Context,
}

impl UsbContext {
    pub fn new() -> Self {
        UsbContext {
            context: Context::new().expect("libusb_init"),
        }
    }

    pub fn open(&self, vid: u16, pid: u16) -> Result<UsbDevice> {
        let mut handle = None;
        let mut e_in = None;
        let mut e_out = None;

        for device in iocall!(self.context.devices())?.iter() {
            let device_desc = iocall!(device.device_descriptor())?;
            if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
                handle = Some(iocall!(device.open())?);
                let config_desc = iocall!(device.active_config_descriptor())?;
                for interface in config_desc.interfaces() {
                    for interface_desc in interface.descriptors() {
                        for endpoint_desc in interface_desc.endpoint_descriptors() {
                            if endpoint_desc.direction() == Direction::In
                                && endpoint_desc.transfer_type() == TransferType::Bulk
                            {
                                e_in = Some(Endpoint {
                                    iface: interface_desc.interface_number(),
                                    address: endpoint_desc.address(),
                                    max_packet_size: endpoint_desc.max_packet_size(),
                                });
                            }
                            if endpoint_desc.direction() == Direction::Out
                                && endpoint_desc.transfer_type() == TransferType::Bulk
                            {
                                e_out = Some(Endpoint {
                                    iface: interface_desc.interface_number(),
                                    address: endpoint_desc.address(),
                                    max_packet_size: endpoint_desc.max_packet_size(),
                                });
                            }
                        }
                    }
                }
            }
        }

        if let (Some(mut handle), Some(e_in), Some(e_out)) = (handle, e_in, e_out) {
            iocall!(handle.claim_interface(e_in.iface))?;
            iocall!(handle.claim_interface(e_out.iface))?;

            Ok(UsbDevice {
                handle: handle,
                e_in: e_in,
                e_out: e_out,
                tx_done_cb: None,
                timeout: Duration::from_secs(1),
            })
        } else {
            Err(Error::from(ErrorKind::NotFound))
        }
    }
}

struct Endpoint {
    iface: u8,
    address: u8,
    max_packet_size: u16,
}

pub struct UsbDevice<'a> {
    handle: DeviceHandle<'a>,
    e_in: Endpoint,
    e_out: Endpoint,
    tx_done_cb: Option<Box<FnMut(u64) + 'a>>,
    timeout: std::time::Duration,
}

impl<'a> UsbDevice<'a> {
    pub fn set_tx_done_cb(&mut self, cb: Option<Box<FnMut(u64)>>) {
        self.tx_done_cb = cb;
    }
}

impl<'a> Read for UsbDevice<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }

        let transfer_size = std::cmp::min(self.e_in.max_packet_size as usize, buf.len());
        iocall!(
            self.handle
                .read_bulk(self.e_in.address, &mut buf[..transfer_size], self.timeout)
        )
    }
}

impl<'a> Write for UsbDevice<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }

        let transfer_size = std::cmp::min(self.e_out.max_packet_size as usize, buf.len());
        let transferred = iocall!(self.handle.write_bulk(
            self.e_out.address,
            &buf[..transfer_size],
            self.timeout
        ))?;

        if let Some(ref mut cb) = self.tx_done_cb {
            cb(transferred as u64);
        }
        Ok(transferred as usize)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

fn err_to_io_err(e: libusb::Error) -> Error {
    Error::from(match e {
        libusb::Error::Pipe => ErrorKind::BrokenPipe,
        libusb::Error::Timeout => ErrorKind::TimedOut,
        libusb::Error::Interrupted => ErrorKind::Interrupted,
        libusb::Error::Access => ErrorKind::PermissionDenied,
        libusb::Error::Busy => ErrorKind::AddrInUse,
        libusb::Error::NotSupported => ErrorKind::NotFound,
        _ => ErrorKind::Other,
    })
}
