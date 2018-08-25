use std;
use std::io::{Read, Write};

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

fn transfer<T: Read + Write>(
    device: &mut T,
    payload: &[u8],
    block: bool,
) -> std::io::Result<Reply> {
    device.write_all(payload)?;
    loop {
        let mut buff = [0; FB_MAX_REPLY_LEN];
        match device.read(&mut buff) {
            Ok(received) => return Ok(Reply::from(&mut buff[..received])),
            Err(err) => {
                match err.kind() {
                    std::io::ErrorKind::TimedOut if block => {
                        continue;
                    }
                    _ => {
                        return Err(err);
                    }
                };
            }
        };
    }
}

pub trait Fastboot {
    fn getvar(&mut self, var: &str) -> Result<String, String>;
    fn download(&mut self, data: &[u8]) -> Result<(), String>;
    fn flash(&mut self, partition: &str) -> Result<(), String>;
    fn erase(&mut self, partition: &str) -> Result<(), String>;
    fn reboot(&mut self) -> Result<(), String>;
}

impl<T: Read + Write> Fastboot for T {
    fn getvar(&mut self, var: &str) -> Result<String, String> {
        let cmd = "getvar:".to_owned() + var;
        let reply = transfer(self, cmd.as_bytes(), true).map_err(|err| err.to_string())?;
        match reply {
            Reply::OKAY(variable) => Ok(variable),
            Reply::FAIL(message) => Err(message),
            _ => Err("Unknown failure".to_owned()),
        }
    }

    fn download(&mut self, data: &[u8]) -> Result<(), String> {
        let cmd = "download:".to_owned() + &format!("{:08x}", data.len());
        let reply = transfer(self, cmd.as_bytes(), false).map_err(|err| err.to_string())?;

        match reply {
            Reply::DATA(size) if size == data.len() => {
                let reply = transfer(self, data, true).map_err(|err| err.to_string())?;
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

    fn flash(&mut self, partition: &str) -> Result<(), String> {
        let cmd = "flash:".to_owned() + &partition;
        let reply = transfer(self, cmd.as_bytes(), true).map_err(|err| err.to_string())?;
        match reply {
            Reply::OKAY(_) => Ok(()),
            Reply::FAIL(message) => Err(message),
            _ => Err("Unknown failure".to_owned()),
        }
    }

    fn erase(&mut self, partition: &str) -> Result<(), String> {
        let cmd = "erase:".to_owned() + &partition;
        let reply = transfer(self, cmd.as_bytes(), true).map_err(|err| err.to_string())?;
        match reply {
            Reply::OKAY(_) => Ok(()),
            Reply::FAIL(message) => Err(message),
            _ => Err("Unknown failure".to_owned()),
        }
    }

    fn reboot(&mut self) -> Result<(), String> {
        let cmd = "reboot";
        let reply = transfer(self, cmd.as_bytes(), true).map_err(|err| err.to_string())?;
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
