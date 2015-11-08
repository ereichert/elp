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
    use std::fmt::Write;

    #[test]
    fn debug_should_write_a_message_without_format_args_to_the_dest() {
        let mut buff = String::new();

        write_log!(&mut buff, true, "DEBUG", "This is a test without format args.", );

        assert_eq!(buff, "DEBUG: This is a test without format args.\n");
    }

    #[test]
    fn debug_should_write_a_message_with_one_format_arg_to_the_dest() {
        let mut buff = String::new();

        write_log!(&mut buff, true, "DEBUG", "This is a test with {} format arg.", "one");

        assert_eq!(buff, "DEBUG: This is a test with one format arg.\n");
    }

    #[test]
    fn debug_should_write_a_message_with_format_args_to_the_dest() {
        let mut buff = String::new();

        write_log!(&mut buff, true, "DEBUG", "This is a test with {} {}.", "format", "args");

        assert_eq!(buff, "DEBUG: This is a test with format args.\n");
    }
}
