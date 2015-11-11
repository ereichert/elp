#[macro_export]
macro_rules! write_log {
    ($dest:expr, $debug:ident, $level:expr, $fmt:expr, $($arg:tt)*) => {
        if $debug {
            use std::io::Write;
            match writeln!($dest, concat!(concat!($level, ": "), $fmt), $($arg)*) {
                Ok(_) => {},
                Err(x) => panic!("Unable to write to stderr: {}", x),
            }
        }
    };
}

#[macro_export]
macro_rules! debug {
    ($debug:ident, $msg:expr) => { write_log!(&mut ::std::io::stderr(), $debug, "DEBUG", $msg, ) };

    ($debug:ident, $fmt:expr, $($arg:tt)*) => { write_log!(&mut ::std::io::stderr(), $debug, "DEBUG", $fmt, $($arg)*) };
}

#[cfg(test)]
mod test {
    extern crate gag;

    use std::fmt::Write;
    use std::io::Read;
    use self::gag::BufferRedirect;
    use std::sync::{Mutex};

    //Gag only allows one redirect per output channel at a time.  Since the tests run in
    //parallel I need a way to synchronize the redirect.  Hence, the mutex.
    lazy_static! {
        static ref STDERR_MUTEX: Mutex<()> = Mutex::new(());
    }

    #[test]
    fn debug_should_write_a_message_without_format_args_to_the_dest() {
        let _l = STDERR_MUTEX.lock().unwrap();
        let mut buff = BufferRedirect::stderr().unwrap();

        debug!(true, "This is a test without format args.");

        let mut output = String::new();
        buff.read_to_string(&mut output).unwrap();
        assert_eq!(&output[..], "DEBUG: This is a test without format args.\n");
    }

    #[test]
    fn debug_should_write_a_message_with_one_format_arg_to_the_dest() {
        let _l = STDERR_MUTEX.lock().unwrap();
        let mut buff = BufferRedirect::stderr().unwrap();

        debug!(true, "This is a test with {} format arg.", "one");

        let mut output = String::new();
        buff.read_to_string(&mut output).unwrap();
        assert_eq!(&output[..], "DEBUG: This is a test with one format arg.\n");
    }

    #[test]
    fn debug_should_write_a_message_with_format_args_to_the_dest() {
        let _l = STDERR_MUTEX.lock().unwrap();
        let mut buff = BufferRedirect::stderr().unwrap();

        debug!(true, "This is a test with {} {}.", "format", "args");

        let mut output = String::new();
        buff.read_to_string(&mut output).unwrap();
        assert_eq!(&output[..], "DEBUG: This is a test with format args.\n");
    }
}
