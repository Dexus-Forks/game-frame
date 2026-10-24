//! The window Glide draws into. The early Glide 2 `grSstOpen` takes no window: a Voodoo drew
//! full screen over the desktop. So this finds the game's own window, makes it a borderless
//! window of the output size, and the render thread draws into it with OpenGL. The game's
//! window is the one of class `WRAPPER_WINDOW_CLASS` (`shim::config`), which may still be
//! hidden; without that variable, the first visible top-level window of the process. Without
//! a game window it makes one.

use crate::log::log;
use crate::win32::*;
use std::ptr::null_mut;

unsafe extern "system" fn find_proc(w: HWND, out: isize) -> BOOL {
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(w, &mut pid) };
    if pid != unsafe { GetCurrentProcessId() } {
        return 1;
    }
    let mut name = [0u8; 64];
    let n = unsafe { GetClassNameA(w, name.as_mut_ptr(), 64) }.max(0) as usize;
    let found = unsafe { &mut *(out as *mut (HWND, bool)) };
    // The named window may still be hidden at the first grSstOpen; any other must be
    // visible, and counts as the game's only when no class is named.
    let class = shim::config::window_class();
    let named = class.is_some_and(|c| &name[..n] == c);
    if !named && unsafe { IsWindowVisible(w) } == 0 {
        return 1;
    }
    let game = named || class.is_none();
    if found.0.is_null() || (game && !found.1) {
        *found = (w, game);
    }
    1
}

unsafe extern "system" fn wndproc(w: HWND, m: u32, wp: usize, lp: isize) -> isize {
    unsafe { DefWindowProcA(w, m, wp, lp) }
}

fn own_window(w: u32, h: u32) -> HWND {
    unsafe {
        let inst = GetModuleHandleA(std::ptr::null());
        let class = c"GlideWrapper".as_ptr().cast();
        let wc = WNDCLASSA {
            style: 0,
            wndproc,
            cls_extra: 0,
            wnd_extra: 0,
            instance: inst,
            icon: null_mut(),
            cursor: null_mut(),
            background: null_mut(),
            menu_name: std::ptr::null(),
            class_name: class,
        };
        RegisterClassA(&wc);
        CreateWindowExA(0, class, c"Glide".as_ptr().cast(), 0x8000_0000 | WS_VISIBLE, 0, 0, w as i32, h as i32, null_mut(), null_mut(), inst, null_mut())
    }
}

/// Find (or make) the window and size it to `w × h` pixels with no frame. Runs on the thread
/// that called `grSstOpen`, which owns the game's window: from any other thread the style
/// change would wait on that thread's message loop.
pub fn prepare(w: u32, h: u32) -> HWND {
    static OWN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let own = OWN.load(std::sync::atomic::Ordering::Relaxed) as HWND;
    let mut found: (HWND, bool) = (null_mut(), false);
    unsafe { EnumWindows(find_proc, &mut found as *mut _ as isize) };
    let hwnd = if found.1 {
        // The game's window exists now: the one made below, if any, goes away.
        if !own.is_null() {
            unsafe { ShowWindow(own, 0) };
        }
        found.0
    } else if !own.is_null() {
        own
    } else {
        log!("no game window yet: making one");
        let own = own_window(w, h);
        OWN.store(own as usize, std::sync::atomic::Ordering::Relaxed);
        own
    };
    unsafe {
        // WS_POPUP | WS_VISIBLE: a borderless window at (0, 0), like the full screen it was.
        SetWindowLongA(hwnd, GWL_STYLE, (0x8000_0000u32 | WS_VISIBLE) as i32);
        let mut r = RECT { left: 0, top: 0, right: w as i32, bottom: h as i32 };
        AdjustWindowRectEx(&mut r, GetWindowLongA(hwnd, GWL_STYLE) as u32, 0, GetWindowLongA(hwnd, GWL_EXSTYLE) as u32);
        SetWindowPos(hwnd, null_mut(), 0, 0, r.right - r.left, r.bottom - r.top, SWP_NOZORDER | 0x20);
    }
    log!("window {hwnd:?} at {w}x{h}");
    hwnd
}

/// The window the game gave (`grSstWinOpen`), made borderless at `w × h`.
pub fn size(hwnd: HWND, w: u32, h: u32) -> HWND {
    unsafe {
        SetWindowLongA(hwnd, GWL_STYLE, (0x8000_0000u32 | WS_VISIBLE) as i32);
        SetWindowPos(hwnd, null_mut(), 0, 0, w as i32, h as i32, SWP_NOZORDER | 0x20);
    }
    log!("window {hwnd:?} (the game's) at {w}x{h}");
    hwnd
}

/// The client size now.
pub fn client(hwnd: HWND) -> (i32, i32) {
    let mut r = RECT::default();
    unsafe { GetClientRect(hwnd, &mut r) };
    (r.right - r.left, r.bottom - r.top)
}

/// Give a window's device context the pixel format OpenGL draws with.
pub fn pixel_format(hwnd: HWND) -> Result<HDC, String> {
    unsafe {
        let dc = GetDC(hwnd);
        let pfd = PIXELFORMATDESCRIPTOR {
            size: std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
            version: 1,
            flags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
            color_bits: 24,
            alpha_bits: 8,
            depth_bits: 24,
            ..Default::default()
        };
        let f = ChoosePixelFormat(dc, &pfd);
        if f == 0 || SetPixelFormat(dc, f, &pfd) == 0 {
            return Err(format!("no pixel format (ChoosePixelFormat {f})"));
        }
        Ok(dc)
    }
}

/// An OpenGL context on the window's device context, made current on this thread.
pub fn context(hwnd: HWND) -> Result<HDC, String> {
    unsafe {
        let dc = pixel_format(hwnd)?;
        let rc = wglCreateContext(dc);
        if rc.is_null() || wglMakeCurrent(dc, rc) == 0 {
            return Err("wglCreateContext failed".into());
        }
        Ok(dc)
    }
}
