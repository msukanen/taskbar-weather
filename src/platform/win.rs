use std::thread;
use windows::core::PCWSTR;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetDesktopWindow, GetWindowLongPtrW, SetParent, SetWindowLongPtrW,
    SetWindowPos, GWL_EXSTYLE, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE,
    SWP_NOZORDER, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
};

use crate::platform::HideAndSeek;
use crate::Overlay;

const WINDOW_TITLE: &str = "TaskbarWeatherAppThigyDoohickey"; // this has to be EXACTLY the same as it's in ui/overlay.slint

impl HideAndSeek for Overlay {
    fn stealth(&self) {
        thread::spawn(move || {
            use std::time::Duration;
            
            log::debug!("La-di-da-da - waiting a while…");
            // Give the application a full second to initialize and settle down.
            thread::sleep(Duration::from_millis(1000));

            let title_wide: Vec<u16> = WINDOW_TITLE
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            // Window *should* be stable by now …
            log::debug!("Hum-di-dum, wait is over…");

            // Fiddle with window parent and some EX_STYLE options to hide us from alt-tab and taskbar.
            unsafe {
                if let Ok(slint_hwnd) = FindWindowW(PCWSTR::null(), PCWSTR::from_raw(title_wide.as_ptr())) {
                    
                    let desktop_hwnd = GetDesktopWindow();
                    SetParent(slint_hwnd, Some(desktop_hwnd)).ok();

                    let ex_style = GetWindowLongPtrW(slint_hwnd, GWL_EXSTYLE);
                    let new_ex_style = (ex_style & !(WS_EX_APPWINDOW.0 as isize)) | (WS_EX_TOOLWINDOW.0 as isize);
                    SetWindowLongPtrW(slint_hwnd, GWL_EXSTYLE, new_ex_style);

                    SetWindowPos(
                        slint_hwnd,
                        None,
                        0, 0, 0, 0,
                        SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER,
                    ).ok();
                    
                } else {
                    log::error!("Could not find the application window after waiting for 1 second.");
                }
            }
            log::info!("Up and running!");
        });
    }
}
