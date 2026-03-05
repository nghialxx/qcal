#![windows_subsystem = "windows"]

use std::ptr::null;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

const HOTKEY_ID: i32 = 1;
const IDC_INPUT: usize = 101;
const IDC_RESULT: usize = 102;
const W: i32 = 400;
const H: i32 = 72;

static mut HWND_MAIN: HWND = 0;
static mut HWND_INPUT: HWND = 0;
static mut HWND_RESULT: HWND = 0;
static mut HFONT_UI: HFONT = 0;
static mut HBRUSH_BG: HBRUSH = 0;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

fn fmt(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        let s = format!("{:.10}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

unsafe fn show(hwnd: HWND) {
    let mut mi = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        rcMonitor: RECT { left: 0, top: 0, right: 0, bottom: 0 },
        rcWork: RECT { left: 0, top: 0, right: 0, bottom: 0 },
        dwFlags: 0,
    };
    GetMonitorInfoW(MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY), &mut mi);
    let x = mi.rcWork.left + (mi.rcWork.right - mi.rcWork.left - W) / 2;
    let y = mi.rcWork.top + (mi.rcWork.bottom - mi.rcWork.top) / 3;
    SetWindowPos(hwnd, HWND_TOPMOST, x, y, W, H, SWP_SHOWWINDOW);
    SetForegroundWindow(hwnd);
    SetFocus(HWND_INPUT);
}

unsafe fn hide(hwnd: HWND) {
    ShowWindow(hwnd, SW_HIDE);
    SetWindowTextW(HWND_INPUT, wide("").as_ptr());
    SetWindowTextW(HWND_RESULT, wide("").as_ptr());
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            HBRUSH_BG = CreateSolidBrush(0x001E1E2E);
            let hi = GetModuleHandleW(null());
            HWND_INPUT = CreateWindowExW(
                0, wide("EDIT").as_ptr(), null(),
                WS_CHILD | WS_VISIBLE | ES_LEFT as u32 | ES_AUTOHSCROLL as u32,
                10, 8, W - 20, 26, hwnd, IDC_INPUT as HMENU, hi, null(),
            );
            HWND_RESULT = CreateWindowExW(
                0, wide("STATIC").as_ptr(), wide("").as_ptr(),
                WS_CHILD | WS_VISIBLE | 0x00000200u32,
                10, 40, W - 20, 24, hwnd, IDC_RESULT as HMENU, hi, null(),
            );
            HFONT_UI = CreateFontW(
                19, 0, 0, 0, 400, 0, 0, 0,
                DEFAULT_CHARSET as u32, OUT_DEFAULT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32,
                CLEARTYPE_QUALITY as u32, (DEFAULT_PITCH | FF_DONTCARE) as u32,
                wide("Segoe UI").as_ptr(),
            );
            SendMessageW(HWND_INPUT, WM_SETFONT, HFONT_UI as WPARAM, 1);
            SendMessageW(HWND_RESULT, WM_SETFONT, HFONT_UI as WPARAM, 1);
            0
        }
        WM_COMMAND => {
            if (wp >> 16) as u16 == EN_CHANGE as u16 && (wp & 0xFFFF) == IDC_INPUT {
                let mut buf = [0u16; 512];
                let len = GetWindowTextW(HWND_INPUT, buf.as_mut_ptr(), 512) as usize;
                let expr = String::from_utf16_lossy(&buf[..len]);
                let result = meval::eval_str(&expr).map(fmt).unwrap_or_default();
                SetWindowTextW(HWND_RESULT, wide(&result).as_ptr());
            }
            0
        }
        WM_ACTIVATE => {
            if wp & 0xFFFF == WA_INACTIVE as WPARAM {
                hide(hwnd);
            }
            0
        }
        WM_HOTKEY => {
            if wp as i32 == HOTKEY_ID {
                show(hwnd);
            }
            0
        }
        WM_CTLCOLOREDIT => {
            SetTextColor(wp as HDC, 0x00CDD6F4);
            SetBkColor(wp as HDC, 0x001E1E2E);
            HBRUSH_BG
        }
        WM_CTLCOLORSTATIC => {
            SetTextColor(wp as HDC, 0x0089B4FA);
            SetBkColor(wp as HDC, 0x001E1E2E);
            HBRUSH_BG
        }
        WM_ERASEBKGND => {
            let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            GetClientRect(hwnd, &mut r);
            FillRect(wp as HDC, &r, HBRUSH_BG);
            1
        }
        WM_DESTROY => {
            DeleteObject(HBRUSH_BG);
            DeleteObject(HFONT_UI);
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

fn main() {
    unsafe {
        let hi = GetModuleHandleW(null());
        let cls = wide("QuickCalc");
        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hi,
            hIcon: 0,
            hCursor: LoadCursorW(0, IDC_ARROW),
            hbrBackground: 0,
            lpszMenuName: null(),
            lpszClassName: cls.as_ptr(),
        };
        RegisterClassW(&wc);

        HWND_MAIN = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            cls.as_ptr(), wide("").as_ptr(),
            WS_POPUP | WS_BORDER,
            0, 0, W, H,
            0, 0, hi, null(),
        );

        RegisterHotKey(HWND_MAIN, HOTKEY_ID, MOD_ALT as u32, VK_SPACE as u32);

        let mut msg = std::mem::zeroed::<MSG>();
        loop {
            if GetMessageW(&mut msg, 0, 0, 0) <= 0 { break; }
            if msg.message == WM_KEYDOWN && msg.wParam == VK_ESCAPE as WPARAM {
                hide(HWND_MAIN);
                continue;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        UnregisterHotKey(HWND_MAIN, HOTKEY_ID);
    }
}