#[cfg(target_os = "windows")]
pub mod win;

pub trait HideAndSeek {
    fn stealth(&self);
}
