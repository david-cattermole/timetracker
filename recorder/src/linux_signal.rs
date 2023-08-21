use libc;

/// Installs signal handler function ('function_handler') for signal
/// 'signal_number' into the currently running process.
///
/// https://www.man7.org/linux/man-pages/man2/sigaction.2.html
pub fn install_signal_handler(signal_number: libc::c_int, function_handler: usize) {
    let mut signal_action = unsafe { std::mem::zeroed::<libc::sigaction>() };

    signal_action.sa_sigaction = function_handler;
    unsafe { libc::sigemptyset(&mut signal_action.sa_mask as *mut libc::sigset_t) };
    signal_action.sa_flags = 0;

    // NOTE: We use 'sigaction' rather than 'signal' because
    // 'sigaction' is more portable than 'signal'.
    if unsafe {
        libc::sigaction(
            signal_number,
            &signal_action as *const libc::sigaction,
            std::ptr::null_mut(),
        )
    } == -1
    {
        panic!("Failed to set signal ({}) handler", signal_number);
    }
}
