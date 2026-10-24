//! Intel's inverse transforms, as its Pentium path runs them (docs/formats/video.md, "The
//! transforms"). Each works on a pair of blocks at once: the coefficient buffer holds 8 rows of
//! 9 words (4×4 blocks use 4 rows of 9), and each 32-bit word packs the left block's value in
//! its low half and the right block's in its high half, both biased (`0x0800` for the Slant
//! transforms, `0x4000` for Haar). The arithmetic is Intel's own, word for word, so that a
//! carry between the halves falls exactly as it does in the original.
//!
//! Translated from `IR41_32.DLL` by the project's SWAR translator (each routine names its
//! address): the transform descriptor table at 0x1006A178 lists them by transform number.

#![allow(unused_assignments, unused_variables, unused_mut, clippy::all)]

/// Transform 4, "Inv Slant 8x8": Intel's 0x10009B60 (columns, then rows).
pub(super) fn slant_8x8(m: &mut [u32]) {
    let (mut eax, mut ebx, mut ecx, mut edx, mut esi, mut edi, mut ebp): (u32, u32, u32, u32, u32, u32, u32) = (0, 0, 0, 0, 0, 0, 0);
    ebp = 0x1c; // 10009b60
    loop {
        eax = m[(ebp.wrapping_add(0x24) >> 2) as usize];
        ebx = m[(ebp.wrapping_add(0x6c) >> 2) as usize];
        ecx = m[(ebp.wrapping_add(0x24) >> 2) as usize];
        eax ^= 0xffffffff;
        edx = m[(ebp.wrapping_add(0x6c) >> 2) as usize];
        ebx ^= 0xffffffff;
        eax = eax.wrapping_add(ecx.wrapping_mul(8)).wrapping_add(0x40005);
        esi = m[(ebp.wrapping_add(0x90) >> 2) as usize];
        edx = ebx.wrapping_add(edx.wrapping_mul(8)).wrapping_add(0x40005);
        edi = m[(ebp.wrapping_add(0xb4) >> 2) as usize];
        eax = eax.wrapping_add(ebx.wrapping_mul(4)).wrapping_add(0x68006804);
        ebx = m[(ebp.wrapping_add(0x0) >> 2) as usize];
        eax >>= 0x3;
        edx = edx.wrapping_add(ecx.wrapping_mul(4)).wrapping_add(0x68006800);
        ecx = esi.wrapping_add(edi);
        eax &= 0xffff1fff;
        edx >>= 0x3;
        esi = esi.wrapping_add(0x10001000);
        esi = esi.wrapping_sub(edi);
        edi = ebx.wrapping_add(eax).wrapping_add(0x18001800);
        ebx = ebx.wrapping_add(0x38003800);
        edx &= 0xffff1fff;
        ebx = ebx.wrapping_sub(eax);
        eax = edi.wrapping_add(ecx).wrapping_sub(0x20002000);
        edi = edi.wrapping_sub(ecx);
        m[(ebp.wrapping_add(0x0) >> 2) as usize] = eax;
        eax = ebx.wrapping_add(esi).wrapping_sub(0x20002000);
        ebx = ebx.wrapping_sub(esi);
        ecx = m[(ebp.wrapping_add(0x48) >> 2) as usize];
        m[(ebp.wrapping_add(0xb4) >> 2) as usize] = ebx;
        m[(ebp.wrapping_add(0x90) >> 2) as usize] = eax;
        eax = m[(ebp.wrapping_add(0xfc) >> 2) as usize];
        ebx = m[(ebp.wrapping_add(0xd8) >> 2) as usize];
        esi = edx.wrapping_add(ecx).wrapping_sub(0x10001000);
        m[(ebp.wrapping_add(0x24) >> 2) as usize] = edi;
        edx = edx.wrapping_sub(ecx);
        edi = eax.wrapping_add(ebx);
        eax = eax.wrapping_add(0x10001000);
        eax = eax.wrapping_sub(ebx);
        ecx = esi.wrapping_add(esi.wrapping_mul(4)).wrapping_add(0x20002);
        m[(ebp.wrapping_add(0xd8) >> 2) as usize] = edi;
        edi = m[(ebp.wrapping_add(0x0) >> 2) as usize];
        ebx = eax.wrapping_add(eax.wrapping_mul(4)).wrapping_sub(0x20002);
        ecx = ecx.wrapping_add(eax.wrapping_mul(2)).wrapping_sub(0x30003000);
        ecx >>= 0x2;
        ebx ^= 0xffffffff;
        eax = m[(ebp.wrapping_add(0x24) >> 2) as usize];
        ecx &= 0xffff3fff;
        ebx = ebx.wrapping_add(esi.wrapping_mul(2)).wrapping_add(0x70007001);
        ebx >>= 0x2;
        esi = edi.wrapping_add(ecx).wrapping_sub(0x20002000);
        ebx &= 0xffff3fff;
        edi = edi.wrapping_sub(ecx);
        m[(ebp.wrapping_add(0x0) >> 2) as usize] = esi;
        m[(ebp.wrapping_add(0x6c) >> 2) as usize] = edi;
        esi = eax.wrapping_add(ebx).wrapping_sub(0x20002000);
        eax = eax.wrapping_sub(ebx);
        edi = m[(ebp.wrapping_add(0xd8) >> 2) as usize];
        m[(ebp.wrapping_add(0x48) >> 2) as usize] = eax;
        eax = edx.wrapping_add(edx.wrapping_mul(4)).wrapping_add(0x20002);
        m[(ebp.wrapping_add(0x24) >> 2) as usize] = esi;
        esi = m[(ebp.wrapping_add(0x90) >> 2) as usize];
        ecx = edi.wrapping_add(edi.wrapping_mul(4)).wrapping_sub(0x20002);
        ecx ^= 0xffffffff;
        eax = eax.wrapping_add(edi.wrapping_mul(2)).wrapping_sub(0x30003000);
        eax >>= 0x2;
        ebx = m[(ebp.wrapping_add(0xb4) >> 2) as usize];
        eax &= 0xffff3fff;
        ecx = ecx.wrapping_add(edx.wrapping_mul(2)).wrapping_add(0x70007001);
        ecx >>= 0x2;
        edx = esi.wrapping_add(eax).wrapping_sub(0x20002000);
        ecx &= 0xffff3fff;
        m[(ebp.wrapping_add(0x90) >> 2) as usize] = edx;
        esi = esi.wrapping_sub(eax);
        eax = ebx.wrapping_add(ecx).wrapping_sub(0x20002000);
        m[(ebp.wrapping_add(0xfc) >> 2) as usize] = esi;
        ebx = ebx.wrapping_sub(ecx);
        m[(ebp.wrapping_add(0xb4) >> 2) as usize] = eax;
        m[(ebp.wrapping_add(0xd8) >> 2) as usize] = ebx;
        let (r, borrow) = ebp.overflowing_sub(0x4); ebp = r; // 10009cfc
        if borrow { break; }
    }
    ebp = 0xfc; // 10009d05
    loop {
        eax = m[(ebp.wrapping_add(0x4) >> 2) as usize];
        ebx = m[(ebp.wrapping_add(0xc) >> 2) as usize];
        ecx = m[(ebp.wrapping_add(0x4) >> 2) as usize];
        eax ^= 0xffffffff;
        edx = m[(ebp.wrapping_add(0xc) >> 2) as usize];
        ebx ^= 0xffffffff;
        eax = eax.wrapping_add(ecx.wrapping_mul(8)).wrapping_add(0x40005);
        esi = m[(ebp.wrapping_add(0x10) >> 2) as usize];
        edx = ebx.wrapping_add(edx.wrapping_mul(8)).wrapping_add(0x40005);
        edi = m[(ebp.wrapping_add(0x14) >> 2) as usize];
        eax = eax.wrapping_add(ebx.wrapping_mul(4)).wrapping_add(0x50005004);
        ebx = m[(ebp.wrapping_add(0x0) >> 2) as usize];
        eax >>= 0x3;
        edx = edx.wrapping_add(ecx.wrapping_mul(4)).wrapping_sub(0x30003000);
        ecx = esi.wrapping_add(edi);
        eax &= 0xffff1fff;
        edx >>= 0x3;
        esi = esi.wrapping_add(0x20002000);
        esi = esi.wrapping_sub(edi);
        edi = ebx.wrapping_add(eax).wrapping_add(0x40014001);
        ebx = ebx.wrapping_add(0x60016001);
        edx &= 0xffff1fff;
        ebx = ebx.wrapping_sub(eax);
        eax = edi.wrapping_add(ecx).wrapping_sub(0x40004000);
        edi = edi.wrapping_sub(ecx);
        m[(ebp.wrapping_add(0x0) >> 2) as usize] = eax;
        eax = ebx.wrapping_add(esi).wrapping_sub(0x40004000);
        ebx = ebx.wrapping_sub(esi);
        ecx = m[(ebp.wrapping_add(0x8) >> 2) as usize];
        m[(ebp.wrapping_add(0x14) >> 2) as usize] = ebx;
        m[(ebp.wrapping_add(0x10) >> 2) as usize] = eax;
        eax = m[(ebp.wrapping_add(0x1c) >> 2) as usize];
        ebx = m[(ebp.wrapping_add(0x18) >> 2) as usize];
        esi = edx.wrapping_add(ecx);
        m[(ebp.wrapping_add(0x4) >> 2) as usize] = edi;
        edi = eax.wrapping_add(ebx);
        eax = eax.wrapping_add(0x20002000);
        eax = eax.wrapping_sub(ebx);
        edx = edx.wrapping_add(0x20002000);
        edx = edx.wrapping_sub(ecx);
        ecx = esi.wrapping_add(esi.wrapping_mul(4)).wrapping_add(0x20002);
        ebx = eax.wrapping_add(eax.wrapping_mul(4)).wrapping_sub(0x20002);
        m[(ebp.wrapping_add(0x18) >> 2) as usize] = edi;
        edi = m[(ebp.wrapping_add(0x0) >> 2) as usize];
        ecx = ecx.wrapping_add(eax.wrapping_mul(2)).wrapping_sub(0x60006000);
        ecx >>= 0x2;
        ebx ^= 0xffffffff;
        eax = m[(ebp.wrapping_add(0x4) >> 2) as usize];
        ecx &= 0xffff3fff;
        ebx = ebx.wrapping_add(esi.wrapping_mul(2)).wrapping_sub(0x1fff1fff);
        ebx >>= 0x2;
        esi = edi.wrapping_add(ecx).wrapping_sub(0x40004000);
        ebx &= 0xffff3fff;
        esi = esi.wrapping_add(esi);
        esi &= 0xfffcfffc;
        edi = edi.wrapping_sub(ecx);
        m[(ebp.wrapping_add(0x0) >> 2) as usize] = esi;
        edi = edi.wrapping_add(edi);
        esi = eax.wrapping_add(ebx).wrapping_sub(0x40004000);
        edi &= 0xfffcfffc;
        m[(ebp.wrapping_add(0xc) >> 2) as usize] = edi;
        edi = m[(ebp.wrapping_add(0x18) >> 2) as usize];
        esi = esi.wrapping_add(esi);
        eax = eax.wrapping_sub(ebx);
        eax = eax.wrapping_add(eax);
        ecx = edi.wrapping_add(edi.wrapping_mul(4)).wrapping_sub(0x20002);
        eax &= 0xfffcfffc;
        ecx ^= 0xffffffff;
        m[(ebp.wrapping_add(0x8) >> 2) as usize] = eax;
        eax = edx.wrapping_add(edx.wrapping_mul(4)).wrapping_add(0x20002);
        esi &= 0xfffcfffc;
        ecx = ecx.wrapping_add(edx.wrapping_mul(2)).wrapping_sub(0x1fff1fff);
        ecx >>= 0x2;
        eax = eax.wrapping_add(edi.wrapping_mul(2)).wrapping_sub(0x60006000);
        eax >>= 0x2;
        m[(ebp.wrapping_add(0x4) >> 2) as usize] = esi;
        eax &= 0xffff3fff;
        esi = m[(ebp.wrapping_add(0x10) >> 2) as usize];
        ecx &= 0xffff3fff;
        ebx = m[(ebp.wrapping_add(0x14) >> 2) as usize];
        edx = esi.wrapping_add(eax).wrapping_sub(0x40004000);
        esi = esi.wrapping_sub(eax);
        edx = edx.wrapping_add(edx);
        eax = ebx.wrapping_add(ecx).wrapping_sub(0x40004000);
        edx &= 0xfffcfffc;
        esi = esi.wrapping_add(esi);
        m[(ebp.wrapping_add(0x10) >> 2) as usize] = edx;
        ebx = ebx.wrapping_sub(ecx);
        esi &= 0xfffcfffc;
        ebx = ebx.wrapping_add(ebx);
        m[(ebp.wrapping_add(0x1c) >> 2) as usize] = esi;
        eax = eax.wrapping_add(eax);
        ebx &= 0xfffcfffc;
        eax &= 0xfffcfffc;
        m[(ebp.wrapping_add(0x18) >> 2) as usize] = ebx;
        m[(ebp.wrapping_add(0x14) >> 2) as usize] = eax;
        let (r, borrow) = ebp.overflowing_sub(0x24); ebp = r; // 10009eb6
        if borrow { break; }
    }
    eax ^= eax;
    let _ = (eax, ebx, ecx, edx, esi, edi, ebp);
}

/// Transform 11, "Inv Slant 4x4": Intel's 0x10009ED0.
pub(super) fn slant_4x4(m: &mut [u32]) {
    let (mut eax, mut ebx, mut ecx, mut edx, mut esi, mut edi, mut ebp): (u32, u32, u32, u32, u32, u32, u32) = (0, 0, 0, 0, 0, 0, 0);
    ebp = 0xc; // 10009ed0
    loop {
        eax = m[(ebp.wrapping_add(0x6c) >> 2) as usize];
        ebx = m[(ebp.wrapping_add(0x24) >> 2) as usize];
        ecx = m[(ebp.wrapping_add(0x0) >> 2) as usize];
        edx = m[(ebp.wrapping_add(0x48) >> 2) as usize];
        edi = ebx.wrapping_add(ebx.wrapping_mul(4)).wrapping_add(0x20002);
        esi = eax.wrapping_add(eax.wrapping_mul(4)).wrapping_sub(0x20002);
        esi ^= 0xffffffff;
        ecx = ecx.wrapping_add(0x20002000);
        edi = edi.wrapping_add(eax.wrapping_mul(2)).wrapping_add(0x8000800);
        esi = esi.wrapping_add(ebx.wrapping_mul(2)).wrapping_add(0x58005801);
        edi >>= 0x2;
        eax = ecx.wrapping_add(edx).wrapping_sub(0x10001000);
        esi >>= 0x2;
        edi &= 0xffff3fff;
        ecx = ecx.wrapping_sub(edx);
        esi &= 0xffff3fff;
        edx = eax.wrapping_add(edi).wrapping_sub(0x20002000);
        eax = eax.wrapping_sub(edi);
        edi = ecx.wrapping_add(esi).wrapping_sub(0x20002000);
        m[(ebp.wrapping_add(0x0) >> 2) as usize] = edx;
        m[(ebp.wrapping_add(0x24) >> 2) as usize] = edi;
        ecx = ecx.wrapping_sub(esi);
        m[(ebp.wrapping_add(0x48) >> 2) as usize] = ecx;
        m[(ebp.wrapping_add(0x6c) >> 2) as usize] = eax;
        let (r, borrow) = ebp.overflowing_sub(0x4); ebp = r; // 10009f4a
        if borrow { break; }
    }
    ebp = 0x6c; // 10009f53
    loop {
        eax = m[(ebp.wrapping_add(0xc) >> 2) as usize];
        ebx = m[(ebp.wrapping_add(0x4) >> 2) as usize];
        ecx = m[(ebp.wrapping_add(0x0) >> 2) as usize];
        edx = m[(ebp.wrapping_add(0x8) >> 2) as usize];
        edi = ebx.wrapping_add(ebx.wrapping_mul(4)).wrapping_add(0x20002);
        esi = eax.wrapping_add(eax.wrapping_mul(4)).wrapping_sub(0x20002);
        esi ^= 0xffffffff;
        ecx = ecx.wrapping_add(0x20002000);
        edi = edi.wrapping_add(eax.wrapping_mul(2)).wrapping_add(0x10001000);
        esi = esi.wrapping_add(ebx.wrapping_mul(2)).wrapping_sub(0x4fff4fff);
        edi >>= 0x2;
        eax = ecx.wrapping_add(edx).wrapping_sub(0x20002000);
        esi >>= 0x2;
        edi &= 0xffff3fff;
        ecx = ecx.wrapping_sub(edx);
        esi &= 0xffff3fff;
        edx = eax.wrapping_add(edi).wrapping_sub(0x1fff1fff);
        eax = eax.wrapping_add(0x20012001);
        eax = eax.wrapping_sub(edi);
        edi = ecx.wrapping_add(esi).wrapping_sub(0x1fff1fff);
        ecx = ecx.wrapping_add(0x20012001);
        edx = edx.wrapping_add(edx);
        edx &= 0xfffcfffc;
        edi = edi.wrapping_add(edi);
        edi &= 0xfffcfffc;
        m[(ebp.wrapping_add(0x0) >> 2) as usize] = edx;
        ecx = ecx.wrapping_sub(esi);
        m[(ebp.wrapping_add(0x4) >> 2) as usize] = edi;
        ecx = ecx.wrapping_add(ecx);
        eax = eax.wrapping_add(eax);
        ecx &= 0xfffcfffc;
        eax &= 0xfffcfffc;
        m[(ebp.wrapping_add(0x8) >> 2) as usize] = ecx;
        m[(ebp.wrapping_add(0xc) >> 2) as usize] = eax;
        let (r, borrow) = ebp.overflowing_sub(0x24); ebp = r; // 10009ff7
        if borrow { break; }
    }
    eax ^= eax;
    let _ = (eax, ebx, ecx, edx, esi, edi, ebp);
}

/// Transform 0, "Inv Haar 8x8": Intel's 0x10015310.
pub(super) fn haar_8x8(m: &mut [u32]) {
    let (mut eax, mut ebx, mut ecx, mut edx, mut esi, mut edi, mut ebp): (u32, u32, u32, u32, u32, u32, u32) = (0, 0, 0, 0, 0, 0, 0);
    ebp = 0xc; // 10015310
    loop {
        eax = m[(ebp.wrapping_add(0x10) >> 2) as usize];
        eax = eax.wrapping_add(0xc000c000);
        ebx = m[(ebp.wrapping_add(0x34) >> 2) as usize];
        eax = eax.wrapping_sub(ebx);
        edx = m[(ebp.wrapping_add(0x58) >> 2) as usize];
        ecx = m[(ebp.wrapping_add(0x7c) >> 2) as usize];
        edi = m[(ebp.wrapping_add(0x10c) >> 2) as usize];
        ebx = eax.wrapping_add(ebx.wrapping_mul(2)).wrapping_add(0x7fff8000);
        esi = m[(ebp.wrapping_add(0xe8) >> 2) as usize];
        eax = eax.wrapping_sub(ecx);
        ebx = ebx.wrapping_sub(edx);
        edi = edi.wrapping_mul(2).wrapping_sub(0x40004000);
        esi = esi.wrapping_mul(2).wrapping_sub(0x40004000);
        ecx = eax.wrapping_add(ecx.wrapping_mul(2)).wrapping_add(0x7fff8000);
        eax = eax.wrapping_sub(edi);
        ecx = ecx.wrapping_sub(esi);
        edx = ebx.wrapping_add(edx.wrapping_mul(2)).wrapping_add(0x7fff8000);
        edi = eax.wrapping_add(edi.wrapping_mul(2)).wrapping_add(0x7fff8000);
        eax &= 0xfffcfffc;
        esi = ecx.wrapping_add(esi.wrapping_mul(2)).wrapping_add(0x7fff8000);
        ecx &= 0xfffcfffc;
        m[(ebp.wrapping_add(0x10c) >> 2) as usize] = eax;
        edi &= 0xfffcfffc;
        eax = m[(ebp.wrapping_add(0xc4) >> 2) as usize];
        m[(ebp.wrapping_add(0xc4) >> 2) as usize] = ecx;
        ecx = m[(ebp.wrapping_add(0xa0) >> 2) as usize];
        esi &= 0xfffcfffc;
        eax = eax.wrapping_mul(2).wrapping_sub(0x40004000);
        m[(ebp.wrapping_add(0xe8) >> 2) as usize] = edi;
        ecx = ecx.wrapping_mul(2).wrapping_sub(0x40004000);
        ebx = ebx.wrapping_sub(eax);
        edx = edx.wrapping_sub(ecx);
        m[(ebp.wrapping_add(0xa0) >> 2) as usize] = esi;
        eax = ebx.wrapping_add(eax.wrapping_mul(2)).wrapping_add(0x7fff8000);
        ebx &= 0xfffcfffc;
        ecx = edx.wrapping_add(ecx.wrapping_mul(2)).wrapping_add(0x7fff8000);
        edx &= 0xfffcfffc;
        m[(ebp.wrapping_add(0x7c) >> 2) as usize] = ebx;
        eax &= 0xfffcfffc;
        m[(ebp.wrapping_add(0x34) >> 2) as usize] = edx;
        ecx &= 0xfffcfffc;
        m[(ebp.wrapping_add(0x10) >> 2) as usize] = ecx;
        m[(ebp.wrapping_add(0x58) >> 2) as usize] = eax;
        let (r, borrow) = ebp.overflowing_sub(0x4); ebp = r; // 10015403
        if (r as i32) < 0 { break; }
    }
    ebp = 0xc; // 1001540c
    loop {
        eax = m[(ebp.wrapping_add(0x0) >> 2) as usize];
        eax = eax.wrapping_add(0xc000c000);
        ebx = m[(ebp.wrapping_add(0x24) >> 2) as usize];
        eax = eax.wrapping_sub(ebx);
        edx = m[(ebp.wrapping_add(0x48) >> 2) as usize];
        ecx = m[(ebp.wrapping_add(0x6c) >> 2) as usize];
        edi = m[(ebp.wrapping_add(0xfc) >> 2) as usize];
        ebx = eax.wrapping_add(ebx.wrapping_mul(2)).wrapping_add(0x7fff8000);
        eax = eax.wrapping_sub(ecx);
        ebx = ebx.wrapping_sub(edx);
        esi = m[(ebp.wrapping_add(0xd8) >> 2) as usize];
        ecx = eax.wrapping_add(ecx.wrapping_mul(2)).wrapping_add(0x7fff8000);
        eax = eax.wrapping_sub(edi);
        ecx = ecx.wrapping_sub(esi);
        edx = ebx.wrapping_add(edx.wrapping_mul(2)).wrapping_add(0x7fff8000);
        edi = eax.wrapping_add(edi.wrapping_mul(2)).wrapping_add(0x7fff8000);
        eax &= 0xfffefffe;
        esi = ecx.wrapping_add(esi.wrapping_mul(2)).wrapping_add(0x7fff8000);
        ecx &= 0xfffefffe;
        m[(ebp.wrapping_add(0xfc) >> 2) as usize] = eax;
        edi &= 0xfffefffe;
        eax = m[(ebp.wrapping_add(0xb4) >> 2) as usize];
        m[(ebp.wrapping_add(0xb4) >> 2) as usize] = ecx;
        ecx = m[(ebp.wrapping_add(0x90) >> 2) as usize];
        esi &= 0xfffefffe;
        ebx = ebx.wrapping_sub(eax);
        m[(ebp.wrapping_add(0xd8) >> 2) as usize] = edi;
        edx = edx.wrapping_sub(ecx);
        m[(ebp.wrapping_add(0x90) >> 2) as usize] = esi;
        eax = ebx.wrapping_add(eax.wrapping_mul(2)).wrapping_add(0x7fff8000);
        ebx &= 0xfffefffe;
        ecx = edx.wrapping_add(ecx.wrapping_mul(2)).wrapping_add(0x7fff8000);
        edx &= 0xfffefffe;
        m[(ebp.wrapping_add(0x6c) >> 2) as usize] = ebx;
        eax &= 0xfffefffe;
        m[(ebp.wrapping_add(0x24) >> 2) as usize] = edx;
        ecx &= 0xfffefffe;
        m[(ebp.wrapping_add(0x0) >> 2) as usize] = ecx;
        m[(ebp.wrapping_add(0x48) >> 2) as usize] = eax;
        let (r, borrow) = ebp.overflowing_sub(0x4); ebp = r; // 100154dd
        if (r as i32) < 0 { break; }
    }
    ebp = 0xfc; // 100154e6
    loop {
        eax = m[(ebp.wrapping_add(0x0) >> 2) as usize];
        eax >>= 0x1;
        ebx = m[(ebp.wrapping_add(0x4) >> 2) as usize];
        ebx >>= 0x1;
        eax = eax.wrapping_add(0x80008000);
        ecx = m[(ebp.wrapping_add(0xc) >> 2) as usize];
        eax = eax.wrapping_sub(ebx);
        ecx >>= 0x1;
        edx = m[(ebp.wrapping_add(0x8) >> 2) as usize];
        edx >>= 0x1;
        ebx = eax.wrapping_add(ebx.wrapping_mul(2)).wrapping_sub(0x40004000);
        edi = m[(ebp.wrapping_add(0x1c) >> 2) as usize];
        eax = eax.wrapping_sub(ecx);
        edi >>= 0x1;
        esi = m[(ebp.wrapping_add(0x18) >> 2) as usize];
        esi >>= 0x1;
        ebx = ebx.wrapping_sub(edx);
        ecx = eax.wrapping_add(ecx.wrapping_mul(2)).wrapping_sub(0x40004000);
        eax = eax.wrapping_sub(edi);
        edx = ebx.wrapping_add(edx.wrapping_mul(2)).wrapping_sub(0x40004000);
        ecx = ecx.wrapping_sub(esi);
        edi = eax.wrapping_add(edi.wrapping_mul(2)).wrapping_sub(0x40004000);
        eax &= 0xfffcfffc;
        esi = ecx.wrapping_add(esi.wrapping_mul(2)).wrapping_sub(0x40004000);
        m[(ebp.wrapping_add(0x1c) >> 2) as usize] = eax;
        ecx &= 0xfffcfffc;
        eax = m[(ebp.wrapping_add(0x14) >> 2) as usize];
        m[(ebp.wrapping_add(0x14) >> 2) as usize] = ecx;
        ecx = m[(ebp.wrapping_add(0x10) >> 2) as usize];
        eax >>= 0x1;
        edi &= 0xfffcfffc;
        ecx >>= 0x1;
        m[(ebp.wrapping_add(0x18) >> 2) as usize] = edi;
        ebx = ebx.wrapping_sub(eax);
        esi &= 0xfffcfffc;
        m[(ebp.wrapping_add(0x10) >> 2) as usize] = esi;
        edx = edx.wrapping_sub(ecx);
        eax = ebx.wrapping_add(eax.wrapping_mul(2)).wrapping_sub(0x40004000);
        ebx &= 0xfffcfffc;
        ecx = edx.wrapping_add(ecx.wrapping_mul(2)).wrapping_sub(0x40004000);
        edx &= 0xfffcfffc;
        m[(ebp.wrapping_add(0xc) >> 2) as usize] = ebx;
        eax &= 0xfffcfffc;
        m[(ebp.wrapping_add(0x4) >> 2) as usize] = edx;
        ecx &= 0xfffcfffc;
        m[(ebp.wrapping_add(0x0) >> 2) as usize] = ecx;
        m[(ebp.wrapping_add(0x8) >> 2) as usize] = eax;
        let (r, borrow) = ebp.overflowing_sub(0x24); ebp = r; // 100155b7
        if (r as i32) < 0 { break; }
    }
    let _ = (eax, ebx, ecx, edx, esi, edi, ebp);
}

/// Transform 1, "Inv Haar 1x8" (rows only): Intel's 0x10015230.
pub(super) fn haar_1x8(m: &mut [u32]) {
    let (mut eax, mut ebx, mut ecx, mut edx, mut esi, mut edi, mut ebp): (u32, u32, u32, u32, u32, u32, u32) = (0, 0, 0, 0, 0, 0, 0);
    ebp = 0xfc; // 10015230
    loop {
        eax = m[(ebp.wrapping_add(0x0) >> 2) as usize];
        eax = eax.wrapping_add(0xc000c000);
        ebx = m[(ebp.wrapping_add(0x4) >> 2) as usize];
        eax = eax.wrapping_sub(ebx);
        edx = m[(ebp.wrapping_add(0x8) >> 2) as usize];
        ecx = m[(ebp.wrapping_add(0xc) >> 2) as usize];
        edi = m[(ebp.wrapping_add(0x1c) >> 2) as usize];
        ebx = eax.wrapping_add(ebx.wrapping_mul(2)).wrapping_add(0x7fff8000);
        esi = m[(ebp.wrapping_add(0x18) >> 2) as usize];
        eax = eax.wrapping_sub(ecx);
        ebx = ebx.wrapping_sub(edx);
        edi = edi.wrapping_mul(2).wrapping_sub(0x40004000);
        esi = esi.wrapping_mul(2).wrapping_sub(0x40004000);
        ecx = eax.wrapping_add(ecx.wrapping_mul(2)).wrapping_add(0x7fff8000);
        eax = eax.wrapping_sub(edi);
        ecx = ecx.wrapping_sub(esi);
        edx = ebx.wrapping_add(edx.wrapping_mul(2)).wrapping_add(0x7fff8000);
        edi = eax.wrapping_add(edi.wrapping_mul(2)).wrapping_add(0x7fff8000);
        eax &= 0xfffcfffc;
        esi = ecx.wrapping_add(esi.wrapping_mul(2)).wrapping_add(0x7fff8000);
        ecx &= 0xfffcfffc;
        m[(ebp.wrapping_add(0x1c) >> 2) as usize] = eax;
        edi &= 0xfffcfffc;
        eax = m[(ebp.wrapping_add(0x14) >> 2) as usize];
        m[(ebp.wrapping_add(0x14) >> 2) as usize] = ecx;
        ecx = m[(ebp.wrapping_add(0x10) >> 2) as usize];
        esi &= 0xfffcfffc;
        eax = eax.wrapping_mul(2).wrapping_sub(0x40004000);
        m[(ebp.wrapping_add(0x18) >> 2) as usize] = edi;
        ecx = ecx.wrapping_mul(2).wrapping_sub(0x40004000);
        ebx = ebx.wrapping_sub(eax);
        edx = edx.wrapping_sub(ecx);
        m[(ebp.wrapping_add(0x10) >> 2) as usize] = esi;
        eax = ebx.wrapping_add(eax.wrapping_mul(2)).wrapping_add(0x7fff8000);
        ebx &= 0xfffcfffc;
        ecx = edx.wrapping_add(ecx.wrapping_mul(2)).wrapping_add(0x7fff8000);
        edx &= 0xfffcfffc;
        m[(ebp.wrapping_add(0xc) >> 2) as usize] = ebx;
        eax &= 0xfffcfffc;
        m[(ebp.wrapping_add(0x4) >> 2) as usize] = edx;
        ecx &= 0xfffcfffc;
        m[(ebp.wrapping_add(0x0) >> 2) as usize] = ecx;
        m[(ebp.wrapping_add(0x8) >> 2) as usize] = eax;
        let (r, borrow) = ebp.overflowing_sub(0x24); ebp = r; // 10015305
        if (r as i32) < 0 { break; }
    }
    let _ = (eax, ebx, ecx, edx, esi, edi, ebp);
}

/// Transform 2, "Inv Haar 8x1" (columns only): Intel's 0x10015130.
pub(super) fn haar_8x1(m: &mut [u32]) {
    let (mut eax, mut ebx, mut ecx, mut edx, mut esi, mut edi, mut ebp): (u32, u32, u32, u32, u32, u32, u32) = (0, 0, 0, 0, 0, 0, 0);
    ebp = 0x1c; // 10015130
    loop {
        eax = m[(ebp.wrapping_add(0x0) >> 2) as usize];
        eax = eax.wrapping_add(0xc000c000);
        ebx = m[(ebp.wrapping_add(0x24) >> 2) as usize];
        eax = eax.wrapping_sub(ebx);
        edx = m[(ebp.wrapping_add(0x48) >> 2) as usize];
        ecx = m[(ebp.wrapping_add(0x6c) >> 2) as usize];
        edi = m[(ebp.wrapping_add(0xfc) >> 2) as usize];
        ebx = eax.wrapping_add(ebx.wrapping_mul(2)).wrapping_add(0x7fff8000);
        esi = m[(ebp.wrapping_add(0xd8) >> 2) as usize];
        eax = eax.wrapping_sub(ecx);
        ebx = ebx.wrapping_sub(edx);
        edi = edi.wrapping_mul(2).wrapping_sub(0x40004000);
        esi = esi.wrapping_mul(2).wrapping_sub(0x40004000);
        ecx = eax.wrapping_add(ecx.wrapping_mul(2)).wrapping_add(0x7fff8000);
        eax = eax.wrapping_sub(edi);
        ecx = ecx.wrapping_sub(esi);
        edx = ebx.wrapping_add(edx.wrapping_mul(2)).wrapping_add(0x7fff8000);
        edi = eax.wrapping_add(edi.wrapping_mul(2)).wrapping_add(0x7fff8000);
        eax &= 0xfffcfffc;
        esi = ecx.wrapping_add(esi.wrapping_mul(2)).wrapping_add(0x7fff8000);
        ecx &= 0xfffcfffc;
        m[(ebp.wrapping_add(0xfc) >> 2) as usize] = eax;
        edi &= 0xfffcfffc;
        eax = m[(ebp.wrapping_add(0xb4) >> 2) as usize];
        m[(ebp.wrapping_add(0xb4) >> 2) as usize] = ecx;
        ecx = m[(ebp.wrapping_add(0x90) >> 2) as usize];
        esi &= 0xfffcfffc;
        eax = eax.wrapping_mul(2).wrapping_sub(0x40004000);
        m[(ebp.wrapping_add(0xd8) >> 2) as usize] = edi;
        ecx = ecx.wrapping_mul(2).wrapping_sub(0x40004000);
        ebx = ebx.wrapping_sub(eax);
        edx = edx.wrapping_sub(ecx);
        m[(ebp.wrapping_add(0x90) >> 2) as usize] = esi;
        eax = ebx.wrapping_add(eax.wrapping_mul(2)).wrapping_add(0x7fff8000);
        ebx &= 0xfffcfffc;
        ecx = edx.wrapping_add(ecx.wrapping_mul(2)).wrapping_add(0x7fff8000);
        edx &= 0xfffcfffc;
        m[(ebp.wrapping_add(0x6c) >> 2) as usize] = ebx;
        eax &= 0xfffcfffc;
        m[(ebp.wrapping_add(0x24) >> 2) as usize] = edx;
        ecx &= 0xfffcfffc;
        m[(ebp.wrapping_add(0x0) >> 2) as usize] = ecx;
        m[(ebp.wrapping_add(0x48) >> 2) as usize] = eax;
        let (r, borrow) = ebp.overflowing_sub(0x4); ebp = r; // 1001521d
        if (r as i32) < 0 { break; }
    }
    let _ = (eax, ebx, ecx, edx, esi, edi, ebp);
}

/// Transform 3, "No Xfrm 8x8": Intel's 0x100155D0, which only rescales.
pub(super) fn none_8x8(m: &mut [u32]) {
    let (mut eax, mut ebx, mut ecx, mut edx, mut esi, mut edi, mut ebp): (u32, u32, u32, u32, u32, u32, u32) = (0, 0, 0, 0, 0, 0, 0);
    esi = 0xfc; // 100155d0
    loop {
        eax = m[(esi.wrapping_add(0x0) >> 2) as usize];
        ebx = m[(esi.wrapping_add(0x4) >> 2) as usize];
        ecx = m[(esi.wrapping_add(0x8) >> 2) as usize];
        edx = m[(esi.wrapping_add(0xc) >> 2) as usize];
        eax = eax.wrapping_mul(4).wrapping_add(0x3fff4000);
        ebx = ebx.wrapping_mul(4).wrapping_add(0x3fff4000);
        m[(esi.wrapping_add(0x0) >> 2) as usize] = eax;
        m[(esi.wrapping_add(0x4) >> 2) as usize] = ebx;
        ecx = ecx.wrapping_mul(4).wrapping_add(0x3fff4000);
        edx = edx.wrapping_mul(4).wrapping_add(0x3fff4000);
        m[(esi.wrapping_add(0x8) >> 2) as usize] = ecx;
        m[(esi.wrapping_add(0xc) >> 2) as usize] = edx;
        eax = m[(esi.wrapping_add(0x10) >> 2) as usize];
        ebx = m[(esi.wrapping_add(0x14) >> 2) as usize];
        ecx = m[(esi.wrapping_add(0x18) >> 2) as usize];
        edx = m[(esi.wrapping_add(0x1c) >> 2) as usize];
        eax = eax.wrapping_mul(4).wrapping_add(0x3fff4000);
        ebx = ebx.wrapping_mul(4).wrapping_add(0x3fff4000);
        m[(esi.wrapping_add(0x10) >> 2) as usize] = eax;
        m[(esi.wrapping_add(0x14) >> 2) as usize] = ebx;
        ecx = ecx.wrapping_mul(4).wrapping_add(0x3fff4000);
        edx = edx.wrapping_mul(4).wrapping_add(0x3fff4000);
        m[(esi.wrapping_add(0x18) >> 2) as usize] = ecx;
        m[(esi.wrapping_add(0x1c) >> 2) as usize] = edx;
        let (r, borrow) = esi.overflowing_sub(0x24); esi = r; // 1001564d
        if (r as i32) < 0 { break; }
    }
    eax ^= eax;
    let _ = (eax, ebx, ecx, edx, esi, edi, ebp);
}

