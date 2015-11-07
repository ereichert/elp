#[macro_export]
macro_rules! debug {
    ($debug:ident, $fmt:expr, $($arg:tt)*) => {
        if $debug {
            use std::io::Write;
            match writeln!(&mut ::std::io::stderr(), concat!("DEBUG: ", $fmt), $($arg)*) {
                Ok(_) => {},
                Err(x) => panic!("Unable to write to stderr: {}", x),
            }
        }
    };

    ($debug:ident, $msg:expr) => { debug!($debug, $msg, ) }
}
