//! Free texture memory as PowerVR's driver counts it (SGLMID4.dll of driver 3.1.0).
//! `sgl_get_free_texture_mem` returns a byte counter
//! (Original: `sgl_get_free_texture_mem` at `10015830`). The counter starts at the board's
//! memory, less what the driver takes at start-up. Each texture made takes a fixed charge from
//! it, looked up by map type and size index. Deleting the texture gives the charge back. Plain
//! Rust, tested on the host.

use std::collections::HashMap;

/// The texture memory of a PCX2 board: 4 MB (Original: the board probe at `100436b4`).
pub const BOARD_MEMORY: u32 = 4 << 20;

/// `TSPParamSize` in KB, as the driver's installer writes it into `sglhw.ini` (`MIDAS.INF`,
/// `[Midas4_UpdateInis]`; *observed*). The driver's own default, with no setting, is 128
/// (`10001460` called at `100013a8`; *observed*). `api::device::install_ini` writes 256.
pub const TSP_PARAM_KB: u32 = 256;

/// The counter at start-up: the board's memory less two TSP parameter buffers of
/// `TSPParamSize` KB (Original: `1001b3e0`, twice) and 4 bytes (Original: `1001b370`), taken
/// after the initialiser `10005400` (*inferred* from the order of the calls).
pub const START: u32 = BOARD_MEMORY - 2 * (TSP_PARAM_KB << 10) - 4;

/// Bytes for size index 0 to 3 (32, 64, 128, 256 pixels square), for maps without mipmaps
/// (Original: the table at `0x100c1768`).
const PLAIN: [u32; 4] = [2048, 8192, 32768, 131072];

/// The same for mipmapped maps (Original: the table at `0x100c1778`).
const MIPMAPPED: [u32; 4] = [2732, 10924, 43692, 174764];

/// What every caller adds for a mipmapped map: 0x154 bytes (Original: `1001aecc`,
/// `sgl_direct_create_texture` at `1002fa80`, `sgl_set_texture_extended` at `1002df80`).
const MIPMAP_EXTRA: u32 = 0x154;

/// `sgl_map_16bit_mm` and `sgl_map_trans16_mm` (sgl.h's `sgl_map_types`).
fn mipmapped(map_type: i32) -> bool {
    matches!(map_type, 1 | 4)
}

/// `sgl_map_8bit`.
const MAP_8BIT: i32 = 2;

/// The bytes one texture takes, from its map type (`sgl_map_types`, 0 to 4) and size index
/// (`sgl_map_sizes`, 0 to 3). Original: `1002fc70` (the table look-up and the halving for
/// 8-bit maps) and its callers (the extra bytes for mipmapped maps). `None` for a type or size
/// outside sgl.h's lists.
pub fn charge(map_type: i32, map_size: i32) -> Option<u32> {
    if !(0..=4).contains(&map_type) || !(0..=3).contains(&map_size) {
        return None;
    }
    let i = map_size as usize;
    if mipmapped(map_type) {
        return Some(MIPMAPPED[i] + MIPMAP_EXTRA);
    }
    let bytes = PLAIN[i];
    Some(if map_type == MAP_8BIT { bytes / 2 } else { bytes })
}

/// The charge of each texture that exists, by name.
#[derive(Default, Debug)]
pub struct TexMem {
    charges: HashMap<u32, u32>,
}

impl TexMem {
    /// Take a texture's charge (Original: the allocator `1001ab60`, which subtracts exactly
    /// the charge). Deviation: a type or size outside sgl.h's lists is charged nothing, and a
    /// texture is never refused for lack of memory; the driver's checks there are not traced.
    pub fn take(&mut self, name: u32, map_type: i32, map_size: i32) {
        self.charges.insert(name, charge(map_type, map_size).unwrap_or(0));
    }

    /// Give a deleted texture's charge back (Original: `1001ad20`, from `sgl_delete_texture`
    /// at `10014900`).
    pub fn give_back(&mut self, name: u32) {
        self.charges.remove(&name);
    }

    pub fn give_back_all(&mut self) {
        self.charges.clear();
    }

    /// The counter: the start less the charges of the textures that exist now. Fragmentation
    /// does not change it (*inferred* from the allocator).
    pub fn free(&self) -> u32 {
        let used: u64 = self.charges.values().map(|&c| c as u64).sum();
        (START as u64).saturating_sub(used) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table() {
        let row = |t| (0..4).map(|s| charge(t, s).unwrap()).collect::<Vec<_>>();
        assert_eq!(row(0), [2048, 8192, 32768, 131072]);
        assert_eq!(row(3), [2048, 8192, 32768, 131072]);
        assert_eq!(row(2), [1024, 4096, 16384, 65536]);
        assert_eq!(row(1), [3072, 11264, 44032, 175104]);
        assert_eq!(row(4), [3072, 11264, 44032, 175104]);
        assert_eq!((charge(5, 0), charge(0, 4), charge(-1, 0), charge(0, -1)), (None, None, None, None));
    }

    #[test]
    fn counter() {
        let mut m = TexMem::default();
        assert_eq!(m.free(), 3670012);
        m.take(2, 0, 3);
        m.take(3, 4, 2);
        assert_eq!(m.free(), START - 131072 - 44032);
        m.give_back(2);
        assert_eq!(m.free(), START - 44032);
        m.give_back(2);
        assert_eq!(m.free(), START - 44032);
        m.take(4, 9, 9);
        assert_eq!(m.free(), START - 44032);
        m.give_back_all();
        assert_eq!(m.free(), START);
    }
}
