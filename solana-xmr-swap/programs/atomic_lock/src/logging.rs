#[cfg(feature = "debug-logs")]
use anchor_lang::prelude::msg;

#[cfg(feature = "debug-logs")]
pub fn debug_log(message: &str) {
    msg!(message);
}

#[cfg(not(feature = "debug-logs"))]
pub fn debug_log(_message: &str) {}
