use std::os::raw::{c_int, c_void};
use std::ptr;
use std::mem;

type WPARAM = usize;
type LPARAM = isize;
type LRESULT = isize;

type HWND = *mut c_void;
type HINSTANCE = *mut c_void;

#[link(name = "user32")]
extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
    fn ShowWindow(hWnd: HWND, nCmdShow: c_int) -> i32;
}

pub struct QQApplication {
    h_instance: *mut c_void,
}

impl QQApplication {
    pub fn new() -> Self {
        unsafe {
            let h_instance = GetModuleHandleW(ptr::null());

            if h_instance.is_null() {
                panic!("Ошибка получения экземпляра приложения.");
            }

            QQApplication { h_instance }
        }
    }

    pub fn exec(&self) {
        unsafe {
            let mut msg: MSG = mem::zeroed();
            while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    pub fn get_h_instance(&self) -> *mut c_void {
        self.h_instance
    }
}

#[repr(C)]
pub struct MSG {
    hwnd: *mut c_void,
    message: u32,
    w_param: WPARAM,
    l_param: LPARAM,
    time: u32,
    pt: POINT,
}

#[repr(C)]
pub struct POINT {
    x: i32,
    y: i32,
}

#[link(name = "user32")]
extern "system" {
    fn GetMessageW(
        lpMsg: *mut MSG,
        hWnd: HWND,
        wMsgFilterMin: u32,
        wMsgFilterMax: u32,
    ) -> i32;

    fn TranslateMessage(lpMsg: *const MSG) -> i32;

    fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
}