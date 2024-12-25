use std::os::raw::{c_int, c_uint, c_void};
use std::ptr;
use std::mem;
use crate::Extensions::QQApplication::{QQApplication, MSG, POINT};

type WPARAM = usize;
type LPARAM = isize;
type LRESULT = isize;
type WNDPROC = unsafe extern "system" fn(*mut c_void, c_uint, WPARAM, LPARAM) -> LRESULT;

const WS_OVERLAPPEDWINDOW: c_uint = 0x00CF0000;
const WM_DESTROY: c_uint = 0x0002;
const WM_PAINT: c_uint = 0x000F;
const SM_CXSCREEN: c_int = 0;
const SM_CYSCREEN: c_int = 1;

const RGB: fn(c_int, c_int, c_int) -> u32 = |r, g, b| ((r & 0xff) | ((g & 0xff) << 8) | ((b & 0xff) << 16)) as u32;

#[repr(C)]
struct WNDCLASSEXW {
    cb_size: u32,
    style: u32,
    lpfn_wnd_proc: WNDPROC,
    cb_cls_extra: c_int,
    cb_wnd_extra: c_int,
    h_instance: *mut c_void,
    h_icon: *mut c_void,
    h_cursor: *mut c_void,
    hbr_background: *mut c_void,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
    h_icon_sm: *mut c_void,
}

#[repr(C)]
struct PAINTSTRUCT {
    hdc: *mut c_void,
    f_erase: i32,
    rc_paint: RECT,
    f_restore: i32,
    f_inc_update: i32,
    rgb_reserved: [u8; 32],
}

#[repr(C)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

type HWND = *mut c_void;
type HINSTANCE = *mut c_void;
type HBRUSH = *mut c_void;

#[link(name = "user32")]
extern "system" {
    fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        X: i32,
        Y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: HWND,
        hMenu: *mut c_void,
        hInstance: HINSTANCE,
        lpParam: *mut c_void,
    ) -> HWND;

    fn DefWindowProcW(
        hWnd: HWND,
        Msg: u32,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> LRESULT;

    fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;

    fn GetMessageW(
        lpMsg: *mut MSG,
        hWnd: HWND,
        wMsgFilterMin: u32,
        wMsgFilterMax: u32,
    ) -> i32;

    fn PostQuitMessage(nExitCode: c_int) -> ();

    fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> u16;

    fn ShowWindow(hWnd: HWND, nCmdShow: c_int) -> i32;

    fn TranslateMessage(lpMsg: *const MSG) -> i32;

    fn BeginPaint(hWnd: HWND, lpPaint: *mut PAINTSTRUCT) -> *mut c_void;

    fn EndPaint(hWnd: HWND, lpPaint: *const PAINTSTRUCT) -> i32;

    fn SetWindowPos(
        hWnd: HWND,
        hWndInsertAfter: HWND,
        X: i32,
        Y: i32,
        cx: i32,
        cy: i32,
        uFlags: u32,
    ) -> i32;

    fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;

    fn GetSystemMetrics(nIndex: c_int) -> c_int;
}

#[link(name = "gdi32")]
extern "system" {
    fn CreateSolidBrush(crColor: u32) -> HBRUSH;

    fn DeleteObject(hObject: *mut c_void) -> i32;

    fn FillRect(hdc: *mut c_void, lprc: *const RECT, hbr: HBRUSH) -> i32;
}

#[no_mangle]
pub unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            let brush = CreateSolidBrush(RGB(255, 255, 255));
            FillRect(hdc, &ps.rc_paint, brush);
            DeleteObject(brush);
            EndPaint(hwnd, &ps);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn center_window(hwnd: HWND, width: i32, height: i32) {
    let screen_width = GetSystemMetrics(SM_CXSCREEN);
    let screen_height = GetSystemMetrics(SM_CYSCREEN);
    let x = (screen_width - width) / 2;
    let y = (screen_height - height) / 2;
    SetWindowPos(hwnd, ptr::null_mut(), x, y, width, height, 0);
}

pub struct QQMainWindow {
    hwnd: HWND,
}

impl QQMainWindow {
    pub fn new(app: &QQApplication, class_name: &str, window_title: &str, width: i32, height: i32) -> Self {
        unsafe {
            let class_name: Vec<u16> = class_name.encode_utf16().chain(Some(0)).collect();
            let window_title: Vec<u16> = window_title.encode_utf16().chain(Some(0)).collect();

            let h_instance = app.get_h_instance();

            let wc = WNDCLASSEXW {
                cb_size: mem::size_of::<WNDCLASSEXW>() as u32,
                style: 0,
                lpfn_wnd_proc: window_proc,
                cb_cls_extra: 0,
                cb_wnd_extra: 0,
                h_instance,
                h_icon: ptr::null_mut(),
                h_cursor: ptr::null_mut(),
                hbr_background: CreateSolidBrush(RGB(255, 255, 255)) as *mut c_void,
                lpsz_menu_name: ptr::null(),
                lpsz_class_name: class_name.as_ptr(),
                h_icon_sm: ptr::null_mut(),
            };

            if RegisterClassExW(&wc) == 0 {
                panic!("Ошибка регистрации класса окна.");
            }

            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_title.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                width,
                height,
                ptr::null_mut(),
                ptr::null_mut(),
                h_instance,
                ptr::null_mut(),
            );

            if hwnd.is_null() {
                panic!("Ошибка создания окна.");
            }

            center_window(hwnd, width, height);
            ShowWindow(hwnd, 5);

            QQMainWindow { hwnd }
        }
    }
}
