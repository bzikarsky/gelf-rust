use libc;

pub fn pid() -> i32 {
    unsafe { libc::getpid() }
}
