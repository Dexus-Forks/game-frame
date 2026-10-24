//! The few Win32 calls the library needs (user32, gdi32, kernel32, opengl32), declared by hand
//! so the crate has no dependencies.

#![allow(non_snake_case, clippy::upper_case_acronyms)]

use std::ffi::c_void;

pub type HWND = *mut c_void;
pub type HDC = *mut c_void;
pub type HGLRC = *mut c_void;
pub type HMODULE = *mut c_void;
pub type BOOL = i32;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
pub struct WNDCLASSA {
    pub style: u32,
    pub wndproc: unsafe extern "system" fn(HWND, u32, usize, isize) -> isize,
    pub cls_extra: i32,
    pub wnd_extra: i32,
    pub instance: HMODULE,
    pub icon: *mut c_void,
    pub cursor: *mut c_void,
    pub background: *mut c_void,
    pub menu_name: *const u8,
    pub class_name: *const u8,
}

#[repr(C)]
#[derive(Default)]
pub struct PIXELFORMATDESCRIPTOR {
    pub size: u16,
    pub version: u16,
    pub flags: u32,
    pub pixel_type: u8,
    pub color_bits: u8,
    pub red_bits: u8,
    pub red_shift: u8,
    pub green_bits: u8,
    pub green_shift: u8,
    pub blue_bits: u8,
    pub blue_shift: u8,
    pub alpha_bits: u8,
    pub alpha_shift: u8,
    pub accum_bits: u8,
    pub accum_red_bits: u8,
    pub accum_green_bits: u8,
    pub accum_blue_bits: u8,
    pub accum_alpha_bits: u8,
    pub depth_bits: u8,
    pub stencil_bits: u8,
    pub aux_buffers: u8,
    pub layer_type: u8,
    pub reserved: u8,
    pub layer_mask: u32,
    pub visible_mask: u32,
    pub damage_mask: u32,
}

pub const PFD_DRAW_TO_WINDOW: u32 = 0x4;
pub const PFD_SUPPORT_OPENGL: u32 = 0x20;
pub const PFD_DOUBLEBUFFER: u32 = 0x1;
pub const SWP_NOZORDER: u32 = 0x4;
pub const GWL_STYLE: i32 = -16;
pub const GWL_EXSTYLE: i32 = -20;
pub const WS_VISIBLE: u32 = 0x1000_0000;

#[link(name = "user32")]
unsafe extern "system" {
    pub fn EnumWindows(f: unsafe extern "system" fn(HWND, isize) -> BOOL, l: isize) -> BOOL;
    pub fn GetWindowThreadProcessId(w: HWND, pid: *mut u32) -> u32;
    pub fn GetClassNameA(w: HWND, name: *mut u8, max: i32) -> i32;
    pub fn IsWindowVisible(w: HWND) -> BOOL;
    pub fn ShowWindow(w: HWND, cmd: i32) -> BOOL;
    pub fn GetDC(w: HWND) -> HDC;
    pub fn GetClientRect(w: HWND, r: *mut RECT) -> BOOL;
    pub fn SetWindowPos(w: HWND, after: HWND, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> BOOL;
    pub fn AdjustWindowRectEx(r: *mut RECT, style: u32, menu: BOOL, ex: u32) -> BOOL;
    pub fn GetWindowLongA(w: HWND, index: i32) -> i32;
    pub fn SetWindowLongA(w: HWND, index: i32, v: i32) -> i32;
    pub fn RegisterClassA(c: *const WNDCLASSA) -> u16;
    pub fn CreateWindowExA(
        ex: u32, class: *const u8, title: *const u8, style: u32, x: i32, y: i32, w: i32, h: i32,
        parent: HWND, menu: *mut c_void, instance: HMODULE, param: *mut c_void,
    ) -> HWND;
    pub fn DefWindowProcA(w: HWND, m: u32, wp: usize, lp: isize) -> isize;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    pub fn ChoosePixelFormat(dc: HDC, pfd: *const PIXELFORMATDESCRIPTOR) -> i32;
    pub fn SetPixelFormat(dc: HDC, format: i32, pfd: *const PIXELFORMATDESCRIPTOR) -> BOOL;
    pub fn SwapBuffers(dc: HDC) -> BOOL;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn GetCurrentProcessId() -> u32;
    pub fn GetModuleHandleA(name: *const u8) -> HMODULE;
    pub fn GetProcAddress(m: HMODULE, name: *const u8) -> *const c_void;
    pub fn LoadLibraryA(name: *const u8) -> HMODULE;
}

#[link(name = "opengl32")]
unsafe extern "system" {
    pub fn wglCreateContext(dc: HDC) -> HGLRC;
    pub fn wglMakeCurrent(dc: HDC, rc: HGLRC) -> BOOL;
    pub fn wglGetProcAddress(name: *const u8) -> *const c_void;
    pub fn wglGetCurrentContext() -> HGLRC;
}
