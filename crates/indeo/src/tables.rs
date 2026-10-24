//! Intel's tables, read at run time from the player's own `IR41_32.DLL` (Gold's `INSTALL`
//! folder, R4.11.15.91). Each is named by its address in the DLL (image base 0x10000000) and
//! never copied into this project (POD's docs/formats/video.md, "Intel's tables").

use super::Error;

/// The DLL's image: its sections, so that an address reads the bytes the loader would map.
struct Image<'a> {
    file: &'a [u8],
    /// (virtual address, virtual size, file offset, file size) of each section.
    sections: Vec<(u32, u32, u32, u32)>,
}

impl<'a> Image<'a> {
    fn parse(file: &'a [u8]) -> Result<Self, Error> {
        let u16_at = |o: usize| file.get(o..o + 2).map(|b| u16::from_le_bytes([b[0], b[1]]));
        let u32_at = |o: usize| file.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()));
        let bad = || Error::IndeoTables("not a PE image".into());
        let pe = u32_at(0x3C).ok_or_else(bad)? as usize;
        if file.get(pe..pe + 4) != Some(b"PE\0\0") {
            return Err(bad());
        }
        let count = u16_at(pe + 6).ok_or_else(bad)? as usize;
        let optional = u16_at(pe + 20).ok_or_else(bad)? as usize;
        if u32_at(pe + 24 + 28) != Some(0x1000_0000) {
            return Err(Error::IndeoTables("image base is not 0x10000000".into()));
        }
        let mut sections = Vec::new();
        for i in 0..count {
            let s = pe + 24 + optional + 40 * i;
            let get = |k: usize| u32_at(s + k).ok_or_else(bad);
            sections.push((0x1000_0000 + get(12)?, get(8)?, get(20)?, get(16)?));
        }
        Ok(Self { file, sections })
    }

    /// `n` bytes at the address `va`; bytes past a section's file data are zero.
    fn read(&self, va: u32, n: usize) -> Result<Vec<u8>, Error> {
        for &(start, vsize, off, fsize) in &self.sections {
            if va >= start && va - start < vsize.max(fsize) {
                let rel = (va - start) as usize;
                let mut out = vec![0; n];
                for (i, b) in out.iter_mut().enumerate() {
                    if rel + i < fsize as usize {
                        *b = *self.file.get(off as usize + rel + i).unwrap_or(&0);
                    }
                }
                return Ok(out);
            }
        }
        Err(Error::IndeoTables(format!("address {va:#x} is in no section")))
    }
}

/// Intel's run/value map (0x100765DC + 0x14C × n, 9 maps): byte +1 on holds the number of
/// levels each run has (run 0 first, to the first zero), and +0x48 a permutation of the 256
/// symbols. `FUN_100015d0` expands it: the symbols the permutation sends to 0 and 1 end a block
/// and escape; then, for each run in turn, its levels −n … −1 and +1 … +n.
#[derive(Clone)]
pub(super) struct RunValueMap {
    pub run_counts: Vec<u8>,
    pub permutation: [u8; 256],
}

/// The tables the decoder reads from Intel's DLL.
#[derive(Clone)]
pub struct Tables {
    /// Codebook descriptors (17-byte records: the number of rows, then each row's bits): the
    /// macroblock codebooks at 0x10064AA0 and the block codebooks at 0x10064B40, 8 of each.
    pub(super) mb_codebooks: Vec<Vec<u8>>,
    pub(super) block_codebooks: Vec<Vec<u8>>,
    pub(super) rv_maps: Vec<RunValueMap>,
    /// The 7 scan orders, 64 bytes each from 0x100A44F8 to 0x100A46B8 (positions in an
    /// 8-wide block; a 4×4 scan uses its first 16).
    pub(super) scans: Vec<[u8; 64]>,
    /// The quantiser matrices from 0x100784F8: 16-bit steps indexed
    /// `((matrix × 2 + set) × 32 + q) × 64 + position`.
    pub(super) quant: Vec<u16>,
}

/// The number of quantiser matrices the table at 0x100784F8 holds: 22 × 2 sets × 32 levels ×
/// 64 steps fill the 0x2C000 bytes up to the scan orders at 0x100A44F8.
const QUANT_MATRICES: usize = 22;

impl Tables {
    /// Tables of one's own, for streams made with them (tests, or a stream encoder): eight
    /// macroblock and eight block codebook descriptors (a row count, then each row's bits),
    /// run/value maps (each run's number of levels, from run 0, and the symbol permutation),
    /// scan orders as Intel holds them after start-up (8 positions a row), and quantiser
    /// steps `((matrix × 2 + set) × 32 + q) × 64 + position` (set 1 for intra blocks).
    pub fn from_parts(
        mb_codebooks: Vec<Vec<u8>>,
        block_codebooks: Vec<Vec<u8>>,
        rv_maps: Vec<(Vec<u8>, [u8; 256])>,
        scans: Vec<[u8; 64]>,
        quant: Vec<u16>,
    ) -> Self {
        let rv_maps = rv_maps.into_iter().map(|(run_counts, permutation)| RunValueMap { run_counts, permutation }).collect();
        Self { mb_codebooks, block_codebooks, rv_maps, scans, quant }
    }

    /// Read the tables from the bytes of Intel's `IR41_32.DLL`.
    pub fn from_dll(file: &[u8]) -> Result<Self, Error> {
        let img = Image::parse(file)?;
        let records = |base: u32| -> Result<Vec<Vec<u8>>, Error> {
            (0..8).map(|i| img.read(base + 17 * i, 17)).collect()
        };
        let mut rv_maps = Vec::new();
        for i in 0..9 {
            let b = img.read(0x1007_65DC + 0x14C * i, 0x14C)?;
            let run_counts = b[1..0x45].iter().copied().take_while(|&c| c != 0).collect();
            let permutation = b[0x48..0x148].try_into().unwrap();
            rv_maps.push(RunValueMap { run_counts, permutation });
        }
        // FUN_10001400 at start-up: a 4×4 scan (its last two bytes equal) is stored with 4
        // positions a row; Intel widens its 16 positions to 8 a row.
        let mut scans: Vec<[u8; 64]> = (0..10)
            .map(|i| img.read(0x100A_44F8 + 64 * i, 64).map(|b| b.try_into().unwrap()))
            .collect::<Result<_, _>>()?;
        for s in &mut scans {
            if s[0x3E] == s[0x3F] {
                for v in &mut s[..16] {
                    *v += *v & 0xC;
                }
            }
        }
        scans.truncate(7);
        let quant_bytes = img.read(0x1007_84F8, QUANT_MATRICES * 2 * 32 * 64 * 2)?;
        let mut quant: Vec<u16> = quant_bytes.chunks(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        // FUN_10001440: the same for a 4×4 matrix (its first table's step 63 is zero), table by
        // table (each with step 63 zero), from the last step down.
        for m in quant.chunks_mut(2 * 32 * 64) {
            if m[63] != 0 {
                continue;
            }
            for t in m.chunks_mut(64) {
                if t[63] == 0 {
                    for u in (4..16).rev() {
                        t[u + (u & 0xC)] = t[u];
                    }
                }
            }
        }
        Ok(Self {
            mb_codebooks: records(0x1006_4AA0)?,
            block_codebooks: records(0x1006_4B40)?,
            rv_maps,
            scans,
            quant,
        })
    }
}
