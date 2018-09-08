use std;
use std::io::{Read, Write};

pub type FbResult<T> = Result<T, String>;

enum Reply {
    OKAY(String),
    DATA(usize),
    FAIL(String),
    INFO(String),
}

impl<'s> From<&'s mut [u8]> for Reply {
    fn from(reply: &'s mut [u8]) -> Self {
        let reply = String::from_utf8_lossy(reply);
        // Split a reply at OKAY/FAIL/DATA
        let (first, second) = reply.split_at(4);
        match first {
            "OKAY" => Reply::OKAY(second.to_owned()),
            "INFO" => Reply::INFO(second.to_owned()),
            "FAIL" => Reply::FAIL(second.to_owned()),
            "DATA" => match usize::from_str_radix(second, 16) {
                Ok(size) => Reply::DATA(size),
                _ => Reply::FAIL("Failed to decode DATA size".to_owned()),
            },
            _ => {
                eprintln!("Received: {}", reply);
                Reply::FAIL(reply.to_string())
            }
        }
    }
}

const FB_MAX_REPLY_LEN: usize = 64;
// According to U-Boot documentation, Fastboot is a synchronous protocol. Therefor
// we should always wait for a reply to our "request". This function will block until
// a reply or an error (except timeout) is received from USB I/O implementation.
// See u-boot/doc/README.android-fastboot-protocol
fn fb_send<T: Read + Write>(io: &mut T, payload: &[u8]) -> FbResult<Reply> {
    io.write_all(payload).map_err(|err| err.to_string())?;
    loop {
        let mut buff = [0; FB_MAX_REPLY_LEN];
        match io.read(&mut buff) {
            Ok(received) => return Ok(Reply::from(&mut buff[..received])),
            Err(err) => {
                match err.kind() {
                    std::io::ErrorKind::TimedOut => {
                        // Trait can't possible now what is a timeout set by a particular Read/Write implementation
                        // so it will *not* consider TimedOut a fatal error. Instead it will just try again
                        // until a reply or another error is received.
                        continue;
                    }
                    _ => {
                        return Err(err.to_string());
                    }
                };
            }
        };
    }
}

pub trait Fastboot {
    fn getvar(&mut self, var: &str) -> FbResult<String>;
    fn download(&mut self, data: &[u8]) -> FbResult<()>;
    fn flash(&mut self, partition: &str) -> FbResult<()>;
    fn erase(&mut self, partition: &str) -> FbResult<()>;
    fn reboot(&mut self) -> FbResult<()>;
}

impl<T: Read + Write> Fastboot for T {
    fn getvar(&mut self, var: &str) -> FbResult<String> {
        let cmd = "getvar:".to_owned() + var;
        let reply = fb_send(self, cmd.as_bytes())?;
        match reply {
            Reply::OKAY(variable) => Ok(variable),
            Reply::FAIL(message) => Err(message),
            _ => Err("Unknown failure".to_owned()),
        }
    }

    fn download(&mut self, data: &[u8]) -> FbResult<()> {
        let cmd = "download:".to_owned() + &format!("{:08x}", data.len());
        let reply = fb_send(self, cmd.as_bytes())?;

        match reply {
            Reply::DATA(size) if size == data.len() => {
                let reply = fb_send(self, data)?;
                match reply {
                    Reply::OKAY(_) => Ok(()),
                    Reply::FAIL(message) => Err(message),
                    _ => Err("Unknown failure".to_owned()),
                }
            }
            Reply::FAIL(message) => Err(message),
            _ => Err("Unknown failure".to_owned()),
        }
    }

    fn flash(&mut self, partition: &str) -> FbResult<()> {
        let cmd = "flash:".to_owned() + &partition;
        let reply = fb_send(self, cmd.as_bytes())?;
        match reply {
            Reply::OKAY(_) => Ok(()),
            Reply::FAIL(message) => Err(message),
            _ => Err("Unknown failure".to_owned()),
        }
    }

    fn erase(&mut self, partition: &str) -> FbResult<()> {
        let cmd = "erase:".to_owned() + &partition;
        let reply = fb_send(self, cmd.as_bytes())?;
        match reply {
            Reply::OKAY(_) => Ok(()),
            Reply::FAIL(message) => Err(message),
            _ => Err("Unknown failure".to_owned()),
        }
    }

    fn reboot(&mut self) -> FbResult<()> {
        let cmd = "reboot";
        let reply = fb_send(self, cmd.as_bytes())?;
        match reply {
            Reply::OKAY(_) => Ok(()),
            Reply::FAIL(message) => Err(message),
            _ => Err("Unknown failure".to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Result;

    #[derive(Default)]
    struct UsbMock {
        replies: Vec<String>,
        downloading: bool,
        download_limit: usize,
    }

    impl UsbMock {
        fn _getvar(&mut self, var: &str) -> String {
            match var {
                "version" => "OKAY1.0".to_owned(),
                _ => "FAIL".to_owned(),
            }
        }

        fn _download(&mut self, hex: &str) -> String {
            if let Ok(size) = usize::from_str_radix(hex, 16) {
                match size {
                    size if size <= self.download_limit => {
                        self.downloading = true;
                        return "DATA".to_owned() + hex;
                    }
                    _ => {
                        return "FAIL".to_owned();
                    }
                };
            }

            "FAIL".to_owned()
        }

        fn _data(&mut self, data: &[u8]) -> String {
            match data.len() {
                size if size <= self.download_limit && self.downloading => {
                    self.downloading = false;
                    "OKAY".to_owned()
                }
                _ => "FAIL".to_owned(),
            }
        }

        fn _flash(&mut self, part: &str) -> String {
            match part {
                "mmc0:dead" => "OKAY1.0".to_owned(),
                _ => "FAIL".to_owned(),
            }
        }

        fn _erase(&mut self, part: &str) -> String {
            match part {
                "mmc0:dead" => "OKAY1.0".to_owned(),
                _ => "FAIL".to_owned(),
            }
        }

        fn _reboot(&mut self) -> String {
            "OKAY".to_owned()
        }
    }

    impl Read for UsbMock {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            if buf.len() == 0 {
                return Ok(0);
            }

            let reply = self.replies.pop().unwrap();
            let size = std::cmp::min(buf.len(), reply.len());
            let (left, _) = buf.split_at_mut(size);
            left.copy_from_slice(&(reply.as_bytes())[..size]);

            Ok(size)
        }
    }

    impl Write for UsbMock {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            if buf.len() == 0 {
                return Ok(0);
            }

            let req = String::from_utf8_lossy(buf);
            if req.starts_with("getvar:") {
                let (_, var) = req.split_at(7);
                let reply = self._getvar(var);
                self.replies.push(reply);
            } else if req.starts_with("download:") {
                let (_, hex) = req.split_at(9);
                let reply = self._download(hex);
                self.replies.push(reply);
            } else if req.starts_with("flash:") {
                let (_, part) = req.split_at(6);
                let reply = self._flash(part);
                self.replies.push(reply);
            } else if req.starts_with("erase:") {
                let (_, part) = req.split_at(6);
                let reply = self._erase(part);
                self.replies.push(reply);
            } else if req.starts_with("reboot") {
                let reply = self._reboot();
                self.replies.push(reply);
            } else {
                let reply = self._data(buf);
                self.replies.push(reply);
            }

            Ok(buf.len())
        }

        fn flush(&mut self) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_getvar() {
        let mut mock = UsbMock::default();
        assert_eq!(Ok("1.0".to_owned()), mock.getvar("version"));
        assert_eq!(Err("".to_owned()), mock.getvar("something"));
    }

    #[test]
    fn test_download() {
        let mut mock = UsbMock {
            replies: Vec::new(),
            downloading: false,
            download_limit: 512,
        };

        assert_eq!(Ok(()), mock.download("data".as_bytes()));
        assert_eq!(Err("".to_owned()), mock.download(&vec![0; 1024]));
    }

    #[test]
    fn test_flash() {
        let mut mock = UsbMock::default();
        assert_eq!(Ok(()), mock.flash("mmc0:dead"));
        assert_eq!(Err("".to_owned()), mock.flash("something"));
    }

    #[test]
    fn test_erase() {
        let mut mock = UsbMock::default();
        assert_eq!(Ok(()), mock.erase("mmc0:dead"));
        assert_eq!(Err("".to_owned()), mock.erase("something"));
    }

    #[test]
    fn test_reboot() {
        let mut mock = UsbMock::default();
        assert_eq!(Ok(()), mock.reboot());
    }
}
