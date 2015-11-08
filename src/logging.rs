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
