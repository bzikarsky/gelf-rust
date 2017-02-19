use libc;

/// Return the process-id (pid) of the current process
pub fn pid() -> i32 {
    unsafe { libc::getpid() }
}
