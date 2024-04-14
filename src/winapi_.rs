use windows::{
    core::{HSTRING, PCWSTR},
    Win32::{
        Foundation::{GetLastError, HWND},
        System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE},
        UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK, SHOW_WINDOW_CMD},
    },
};

use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, SetWindowPos, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
};

pub fn set_window_always_on_top(title: &str, aot: bool) {
    unsafe {
        let wnd_title = HSTRING::from(title);
        let hwnd = FindWindowW(PCWSTR::null(), &wnd_title);

        let flag = if aot { HWND_TOPMOST } else { HWND_NOTOPMOST };

        let _ = SetWindowPos(
            hwnd,
            flag,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOACTIVATE | SWP_NOSIZE,
        );
    }
}

pub fn open_file_in_default_application(filepath: &str) {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);
        dbg!(filepath);
        let _i = ShellExecuteW(
            HWND::default(),
            PCWSTR::null(),
            &HSTRING::from(filepath),
            PCWSTR::null(),
            PCWSTR::null(),
            SHOW_WINDOW_CMD(0),
        );

        let error = GetLastError();
        if error.is_ok() {
            return;
        }

        let message = HSTRING::from(format!(
            "Failed to open file. {}",
            error.to_hresult().to_string()
        ));

        MessageBoxW(
            HWND::default(),
            &message,
            &HSTRING::from("Error"),
            MB_OK | MB_ICONERROR,
        );
    }
}
