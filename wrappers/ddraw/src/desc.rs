//! `DDSURFACEDESC` (0x6c bytes) and `DDPIXELFORMAT` (0x20 bytes), read and written by
//! offset.

// DDSD_*
pub const CAPS: u32 = 0x1;
pub const HEIGHT: u32 = 0x2;
pub const WIDTH: u32 = 0x4;
pub const PITCH: u32 = 0x8;
pub const BACKBUFFERCOUNT: u32 = 0x20;
pub const LPSURFACE: u32 = 0x800;
pub const PIXELFORMAT: u32 = 0x1000;

/// What a description says.
#[derive(Clone, Copy, Debug, Default)]
pub struct Desc {
    pub w: u32,
    pub h: u32,
    pub pitch: u32,
    pub bpp: u32,
    pub caps: u32,
    pub surface: u32,
    pub back_count: u32,
}

/// The fields `CreateSurface` reads, with `flags` (`DDSD_*`) saying which are set.
pub unsafe fn read(d: *const u32) -> (u32, Desc) {
    let w = |o: usize| unsafe { d.add(o / 4).read_unaligned() };
    let flags = w(4);
    let desc = Desc {
        h: w(8),
        w: w(0xc),
        back_count: w(0x14),
        bpp: if flags & PIXELFORMAT != 0 { w(0x54) } else { 0 },
        caps: w(0x68),
        ..Default::default()
    };
    (flags, desc)
}

/// The 16-bit layout: RGB565, or RGB555 with `DDRAW_PIXELS=555` (the S3 build: the ViRGE
/// draws 15-bit pixels only, so its 16-bit modes were 5-5-5, *inferred*).
pub fn masks16() -> [u32; 3] {
    if std::env::var("DDRAW_PIXELS").is_ok_and(|v| v == "555") { [0x7c00, 0x03e0, 0x001f] } else { [0xf800, 0x07e0, 0x001f] }
}

/// 16-bit RGB (`masks16`), or palette indices for 8 bits.
pub unsafe fn write_pixel_format(pf: *mut u32, bpp: u32) {
    let [r, g, b] = masks16();
    let words: [u32; 8] = if bpp == 8 {
        [0x20, 0x60, 0, 8, 0, 0, 0, 0] // DDPF_RGB | DDPF_PALETTEINDEXED8
    } else {
        [0x20, 0x40, 0, 16, r, g, b, 0] // DDPF_RGB
    };
    for (i, v) in words.iter().enumerate() {
        unsafe { pf.add(i).write_unaligned(*v) };
    }
}

pub unsafe fn write(d: *mut u32, s: &Desc) {
    unsafe {
        let size = d.read_unaligned().max(0x6c) as usize;
        std::ptr::write_bytes(d.cast::<u8>(), 0, size.min(0x7c));
        let put = |o: usize, v: u32| d.add(o / 4).write_unaligned(v);
        put(0, 0x6c);
        put(4, CAPS | HEIGHT | WIDTH | PITCH | PIXELFORMAT | LPSURFACE | if s.back_count > 0 { BACKBUFFERCOUNT } else { 0 });
        put(8, s.h);
        put(0xc, s.w);
        put(0x10, s.pitch);
        put(0x14, s.back_count);
        put(0x24, s.surface);
        put(0x68, s.caps);
        write_pixel_format(d.add(0x48 / 4), s.bpp);
    }
}
