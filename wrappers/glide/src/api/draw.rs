//! Drawing calls: triangles and lines of `GrVertex`.

use crate::cmd::{Cmd, Vertex, push};

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grDrawTriangle(a: *const Vertex, b: *const Vertex, c: *const Vertex) {
    if a.is_null() || b.is_null() || c.is_null() {
        return;
    }
    super::lfb::finish_write();
    unsafe { push(Cmd::Triangle([a.read_unaligned(), b.read_unaligned(), c.read_unaligned()])) };
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grDrawLine(a: *const Vertex, b: *const Vertex) {
    if a.is_null() || b.is_null() {
        return;
    }
    super::lfb::finish_write();
    unsafe { push(Cmd::Line([a.read_unaligned(), b.read_unaligned()])) };
}
