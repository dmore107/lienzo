// Windows: portapapeles, foco, pegado, colores del sistema y app de origen.
pub use imp::*;

#[cfg(windows)]
mod imp {
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::DataExchange::*,
        System::Memory::{GlobalLock, GlobalUnlock},
        System::Threading::{GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION},
        UI::Input::KeyboardAndMouse::*,
        UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId, MessageBoxW, SetForegroundWindow, MB_ICONERROR},
    };

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn seq() -> u32 {
        unsafe { GetClipboardSequenceNumber() }
    }

    /// Respeta a los gestores de contraseñas y apps que piden no guardar su contenido.
    pub fn excluded() -> bool {
        unsafe {
            let ex = RegisterClipboardFormatW(wide("ExcludeClipboardContentFromMonitorProcessing").as_ptr());
            if ex != 0 && IsClipboardFormatAvailable(ex) != 0 {
                return true;
            }
            let hist = RegisterClipboardFormatW(wide("CanIncludeInClipboardHistory").as_ptr());
            if hist != 0 && IsClipboardFormatAvailable(hist) != 0 {
                let mut allowed = true;
                if OpenClipboard(std::ptr::null_mut()) != 0 {
                    let h = GetClipboardData(hist);
                    if !h.is_null() {
                        let p = GlobalLock(h) as *const u32;
                        if !p.is_null() {
                            allowed = *p != 0;
                            GlobalUnlock(h);
                        }
                    }
                    CloseClipboard();
                }
                return !allowed;
            }
            false
        }
    }

    /// Nombre del programa que puso el contenido en el portapapeles (p. ej. "chrome").
    pub fn clipboard_source() -> String {
        unsafe {
            let mut hwnd = GetClipboardOwner();
            if hwnd.is_null() {
                hwnd = GetForegroundWindow();
            }
            if hwnd.is_null() {
                return String::new();
            }
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == 0 || pid == GetCurrentProcessId() {
                return String::new();
            }
            let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if h.is_null() {
                return String::new();
            }
            let mut buf = [0u16; 520];
            let mut len = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut len);
            CloseHandle(h);
            if ok == 0 {
                return String::new();
            }
            let path = String::from_utf16_lossy(&buf[..len as usize]);
            std::path::Path::new(&path).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
        }
    }

    pub fn foreground() -> isize {
        unsafe { GetForegroundWindow() as isize }
    }

    pub fn focus(h: isize) {
        if h != 0 {
            unsafe { SetForegroundWindow(h as _) };
        }
    }

    pub fn alert(text: &str) {
        let (t, c) = (wide(text), wide("Lienzo"));
        unsafe { MessageBoxW(std::ptr::null_mut(), t.as_ptr(), c.as_ptr(), MB_ICONERROR) };
    }

    fn reg_dword(sub: &str, name: &str) -> Option<u32> {
        use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
        let (sub, name) = (wide(sub), wide(name));
        let mut data: u32 = 0;
        let mut size: u32 = 4;
        let r = unsafe {
            RegGetValueW(HKEY_CURRENT_USER, sub.as_ptr(), name.as_ptr(), RRF_RT_REG_DWORD, std::ptr::null_mut(), &mut data as *mut u32 as *mut _, &mut size)
        };
        if r == 0 { Some(data) } else { None }
    }

    /// Color de acento de Windows (#rrggbb) y si las apps usan modo oscuro.
    pub fn system_colors() -> (String, bool) {
        let accent = reg_dword("Software\\Microsoft\\Windows\\DWM", "AccentColor")
            .map(|v| (v & 0xff, (v >> 8) & 0xff, (v >> 16) & 0xff)) // ABGR
            .or_else(|| reg_dword("Software\\Microsoft\\Windows\\DWM", "ColorizationColor").map(|v| ((v >> 16) & 0xff, (v >> 8) & 0xff, v & 0xff))) // ARGB
            .unwrap_or((0, 120, 212));
        let dark = reg_dword("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize", "AppsUseLightTheme").map(|v| v == 0).unwrap_or(false);
        (format!("#{:02x}{:02x}{:02x}", accent.0, accent.1, accent.2), dark)
    }

    pub fn send_ctrl_v() {
        fn key(vk: u16, up: bool) -> INPUT {
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT { wVk: vk, wScan: 0, dwFlags: if up { KEYEVENTF_KEYUP } else { 0 }, time: 0, dwExtraInfo: 0 },
                },
            }
        }
        // Suelta Alt/Shift/Win por si siguen presionadas por el atajo
        let inputs = [
            key(VK_MENU, true),
            key(VK_SHIFT, true),
            key(VK_LWIN, true),
            key(VK_RWIN, true),
            key(VK_CONTROL, false),
            key(0x56, false),
            key(0x56, true),
            key(VK_CONTROL, true),
        ];
        unsafe { SendInput(inputs.len() as u32, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32) };
    }
}

#[cfg(not(windows))]
mod imp {
    pub fn seq() -> u32 { 0 }
    pub fn excluded() -> bool { false }
    pub fn clipboard_source() -> String { String::new() }
    pub fn foreground() -> isize { 0 }
    pub fn focus(_: isize) {}
    pub fn alert(text: &str) { eprintln!("{text}") }
    pub fn send_ctrl_v() {}
    pub fn system_colors() -> (String, bool) { ("#0078d4".into(), false) }
}
