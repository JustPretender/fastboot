pub mod fastboot;

#[cfg(test)]
mod tests {
    use fastboot::Fastboot;
    use std::cell::RefCell;
    use std::error::Error;
    use std::fmt;
    use std::io;

    extern crate double;
    use self::double::Mock;

    #[derive(Debug, Clone)]
    struct CloneableError {
        pub kind: io::ErrorKind,
        pub description: String,
    }

    impl Error for CloneableError {
        fn description(&self) -> &str {
            self.description.as_str()
        }
    }

    impl fmt::Display for CloneableError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.description())
        }
    }

    #[derive(Debug, Clone)]
    struct MockUsb {
        pub read: Mock<*mut u8, Result<usize, CloneableError>>,
        pub write: Mock<Vec<u8>, Result<usize, CloneableError>>,
        pub flush: Mock<(), Result<(), CloneableError>>,
    }

    impl Default for MockUsb {
        fn default() -> Self {
            MockUsb {
                read: Mock::new(Ok(0)),
                write: Mock::new(Ok(0)),
                flush: Mock::new(Ok(())),
            }
        }
    }

    impl io::Read for MockUsb {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if buf.len() == 0 {
                return Ok(0);
            }

            let arg = buf.as_mut_ptr();
            self.read.call(arg).map_err(|err| {
                let (kind, description) = (err.kind, err.description);
                io::Error::new(kind, description)
            })
        }
    }

    impl io::Write for MockUsb {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if buf.len() == 0 {
                return Ok(0);
            }

            let arg = buf.to_vec();
            self.write.call(arg).map_err(|err| {
                let (kind, description) = (err.kind, err.description);
                io::Error::new(kind, description)
            })
        }

        fn flush(&mut self) -> io::Result<()> {
            let arg = ();
            self.flush.call(arg).map_err(|err| {
                let (kind, description) = (err.kind, err.description);
                io::Error::new(kind, description)
            })
        }
    }

    #[test]
    fn test_getvar() {
        let mut mock = MockUsb::default();

        mock.write
            .return_value_for("getvar:version".as_bytes(), Ok(14));
        mock.read.use_closure(Box::new(|buf| {
            let reply = "OKAY1.0";
            unsafe { reply.as_ptr().copy_to_nonoverlapping(buf, reply.len()) };
            Ok(reply.len())
        }));
        assert_eq!(Ok("1.0".to_owned()), mock.getvar("version"));

        mock.write
            .return_value_for("getvar:something".as_bytes(), Ok(16));
        mock.read.use_closure(Box::new(|buf| {
            let reply = "FAIL";
            unsafe { reply.as_ptr().copy_to_nonoverlapping(buf, reply.len()) };
            Ok(reply.len())
        }));
        assert_eq!(Err("".to_owned()), mock.getvar("something"));
    }

    #[test]
    fn test_download() {
        let mut mock = MockUsb::default();

        mock.write
            .return_value_for("download:00000004".as_bytes(), Ok(17));
        mock.write.return_value_for("data".as_bytes(), Ok(4));
        let flag = RefCell::new(false);
        mock.read.use_closure(Box::new(move |buf| {
            let reply = if !*flag.borrow() {
                *flag.borrow_mut() = true;
                "DATA00000004"
            } else {
                "OKAY"
            };
            unsafe { reply.as_ptr().copy_to_nonoverlapping(buf, reply.len()) };
            Ok(reply.len())
        }));
        assert_eq!(Ok(()), mock.download("data".as_bytes()));

        mock.write
            .return_value_for("download:00000400".as_bytes(), Ok(17));
        mock.read.use_closure(Box::new(move |buf| {
            let reply = "FAIL";
            unsafe { reply.as_ptr().copy_to_nonoverlapping(buf, reply.len()) };
            Ok(reply.len())
        }));
        assert_eq!(Err("".to_owned()), mock.download(&vec![0; 1024]));
    }

    #[test]
    fn test_flash() {
        let mut mock = MockUsb::default();

        mock.write
            .return_value_for("flash:mmc0:dead".as_bytes(), Ok(15));
        mock.read.use_closure(Box::new(|buf| {
            let reply = "OKAY1.0";
            unsafe { reply.as_ptr().copy_to_nonoverlapping(buf, reply.len()) };
            Ok(reply.len())
        }));
        assert_eq!(Ok(()), mock.flash("mmc0:dead"));

        mock.write
            .return_value_for("flash:something".as_bytes(), Ok(15));
        mock.read.use_closure(Box::new(|buf| {
            let reply = "FAIL";
            unsafe { reply.as_ptr().copy_to_nonoverlapping(buf, reply.len()) };
            Ok(reply.len())
        }));
        assert_eq!(Err("".to_owned()), mock.flash("something"));
    }

    #[test]
    fn test_erase() {
        let mut mock = MockUsb::default();

        mock.write
            .return_value_for("erase:mmc0:dead".as_bytes(), Ok(15));
        mock.read.use_closure(Box::new(|buf| {
            let reply = "OKAY1.0";
            unsafe { reply.as_ptr().copy_to_nonoverlapping(buf, reply.len()) };
            Ok(reply.len())
        }));
        assert_eq!(Ok(()), mock.erase("mmc0:dead"));

        mock.write
            .return_value_for("erase:something".as_bytes(), Ok(15));
        mock.read.use_closure(Box::new(|buf| {
            let reply = "FAIL";
            unsafe { reply.as_ptr().copy_to_nonoverlapping(buf, reply.len()) };
            Ok(reply.len())
        }));
        assert_eq!(Err("".to_owned()), mock.erase("something"));
    }

    #[test]
    fn test_reboot() {
        let mut mock = MockUsb::default();

        mock.write.return_value_for("reboot".as_bytes(), Ok(6));
        mock.read.use_closure(Box::new(|buf| {
            let reply = "OKAY1.0";
            unsafe { reply.as_ptr().copy_to_nonoverlapping(buf, reply.len()) };
            Ok(reply.len())
        }));
        assert_eq!(Ok(()), mock.reboot());
    }
}
