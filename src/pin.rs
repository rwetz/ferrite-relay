//! Native styling is restricted to Relay's own HWND.
pub fn pin(window: &gpui::Window) {
    #[cfg(windows)]
    unsafe {
        use raw_window_handle::HasWindowHandle;
        use std::ffi::c_void;
        #[link(name = "user32")]
        unsafe extern "system" {
            fn GetWindowLongPtrW(hwnd: *mut c_void, index: i32) -> isize;
            fn SetWindowLongPtrW(hwnd: *mut c_void, index: i32, value: isize) -> isize;
            fn SetWindowPos(
                hwnd: *mut c_void,
                after: *mut c_void,
                x: i32,
                y: i32,
                w: i32,
                h: i32,
                flags: u32,
            ) -> i32;
        }
        if let Ok(raw_window_handle::RawWindowHandle::Win32(h)) =
            HasWindowHandle::window_handle(window).map(|h| h.as_raw())
        {
            let hwnd = h.hwnd.get() as *mut c_void;
            let style = GetWindowLongPtrW(hwnd, -16);
            SetWindowLongPtrW(hwnd, -16, style & !(0x00C00000 | 0x00040000)); // caption + thick frame
            SetWindowPos(
                hwnd,
                -1isize as *mut c_void,
                0,
                0,
                0,
                0,
                0x0001 | 0x0002 | 0x0010 | 0x0020,
            ); // topmost, preserve geometry, no activation
        }
    }
    #[cfg(not(windows))]
    let _ = window;
}
