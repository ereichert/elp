#[macro_use]
extern crate log;

pub mod elb_log_files;

pub struct RuntimeContext {
    pub debug: bool,
}
