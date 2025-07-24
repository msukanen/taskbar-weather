use std::thread;
use windows::core::PCWSTR;
use windows::Win32::UI::Shell::{SHAppBarMessage, ABM_GETTASKBARPOS, APPBARDATA, ABE_TOP, ABE_LEFT, ABE_RIGHT};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetDesktopWindow, GetWindowLongPtrW, SetParent, SetWindowLongPtrW,
    SetWindowPos, GWL_EXSTYLE, SWP_FRAMECHANGED, SWP_NOSIZE,
    SWP_NOZORDER, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
};

use crate::platform::HideAndSeek;
use crate::Overlay;

const WINDOW_TITLE: &str = "TaskbarWeatherAppThigyDoohickey"; // NOTE: this has to be EXACTLY the same as it's in ui/overlay.slint!
const WINDOW_WIDTH: i32 = 160; // NOTE: match with ui/overlay.slint!
const WINDOW_HEIGHT: i32 = 40; // NOTE: match with ui/overlay.slint!

impl HideAndSeek for Overlay {
    fn stealth(&self) {
        
        thread::spawn(move || {
            use std::time::Duration;
            
            // Give the application a full second to initialize and settle down.
            thread::sleep(Duration::from_millis(1000));

            let title_wide: Vec<u16> = WINDOW_TITLE
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            // Window *should* be stable by now …
            log::debug!("Hum-di-dum, wait is over… Let see about (re-)positioning...");

            // Fiddle with window parent and some EX_STYLE options to hide us from alt-tab and taskbar.
            unsafe {
                if let Ok(slint_hwnd) = FindWindowW(PCWSTR::null(), PCWSTR::from_raw(title_wide.as_ptr())) {

                    let mut app_bar_data: APPBARDATA = std::mem::zeroed();
                    app_bar_data.cbSize = std::mem::size_of::<APPBARDATA>() as u32;

                    // Figure out where the 'task bar' is, but if not found for some weird reason, fall back to default (x,y).
                    let (x, y) = if SHAppBarMessage(ABM_GETTASKBARPOS, &mut app_bar_data) != 0 {
                        let taskbar_rect = app_bar_data.rc;
                        match app_bar_data.uEdge {
                            ABE_TOP => (taskbar_rect.right - WINDOW_WIDTH - 10, taskbar_rect.bottom + 10),
                            ABE_LEFT => (taskbar_rect.right + 10, taskbar_rect.bottom - WINDOW_HEIGHT - 10),
                            ABE_RIGHT => (taskbar_rect.left - WINDOW_WIDTH - 10, taskbar_rect.bottom - WINDOW_HEIGHT - 10),
                            _ => (taskbar_rect.right - WINDOW_WIDTH - 10, taskbar_rect.top - WINDOW_HEIGHT - 10)
                        }
                    } else {
                        // Taskbar's amiss? Oh well – default coords it is then.
                        (20, 20)
                    };
                    
                    let desktop_hwnd = GetDesktopWindow();
                    SetParent(slint_hwnd, Some(desktop_hwnd)).ok();

                    let ex_style = GetWindowLongPtrW(slint_hwnd, GWL_EXSTYLE);
                    let new_ex_style = (ex_style & !(WS_EX_APPWINDOW.0 as isize)) | (WS_EX_TOOLWINDOW.0 as isize);
                    SetWindowLongPtrW(slint_hwnd, GWL_EXSTYLE, new_ex_style);

                    SetWindowPos(
                        slint_hwnd,
                        None,
                        x, y, 0, 0,
                        SWP_FRAMECHANGED | SWP_NOSIZE | SWP_NOZORDER,
                    ).ok();
                    
                } else {
                    log::error!("Could not find the application window after waiting for 1 second.");
                }
            }
            log::info!("Up and running!");
        });
    }
}
