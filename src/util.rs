use libc;
use std::env;

pub fn pid() -> i32 {
    unsafe { libc::getpid() }
}
