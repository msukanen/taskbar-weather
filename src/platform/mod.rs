pub mod nogui;
#[cfg(target_os = "windows")]
pub mod win;

/// A trait for UI's "stealth mode".
pub trait HideAndSeek {
    /// Enter stealth mode.  This is generally a one-way trip,
    /// but may vary by OS and/or implementation.
    fn stealth(&self);
}
