use core::time::Duration;
use std::thread::sleep;

use indicatif::ProgressBar;

/// Sets the current message on a [`ProgressBar`] if it is not `None`.
pub fn set_optional_message(progress: Option<&ProgressBar>, msg: &str) {
    if let Some(pb) = progress {
        pb.set_message(String::from(msg))
    }
}

/// Prints a debug error message and causes the current thread to temporarily
/// sleep for development debugging purposes.
#[allow(dead_code)]
pub fn debug_print(msg: &str) {
    eprintln!("{}", msg);
    sleep(Duration::from_millis(3000));
}
