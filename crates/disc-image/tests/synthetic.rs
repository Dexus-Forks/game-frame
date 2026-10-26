//! A tiny image built here, in every layout the crate reads: a cooked ISO, raw Mode 1 and
//! Mode 2 Form 1 BIN files, and CUE sheets with an audio track after the data.

use disc_image::{ChunkCache, Disc, Memory, Missing, Mode, ReadAt, parse_cue};
use std::sync::Arc;

const S: usize = 2048;
const BLOCKS: usize = 28;
const DATE: [u8; 7] = [97, 1, 13, 13, 53, 19, 0];

fn big() -> Vec<u8> {
    (0..5000u32).map(|i| (i % 251) as u8).collect()
}

fn record(name: &[u8], lba: u32, size: u32, flags: u8) -> Vec<u8> {
    let len = 33 + name.len() + (name.len() + 1) % 2;
    let mut r = vec![0; len];
    r[0] = len as u8;
    r[2..6].copy_from_slice(&lba.to_le_bytes());
    r[6..10].copy_from_slice(&lba.to_be_bytes());
    r[10..14].copy_from_slice(&size.to_le_bytes());
    r[14..18].copy_from_slice(&size.to_be_bytes());
    r[18..25].copy_from_slice(&DATE);
    r[25] = flags;
    r[28] = 1;
    r[32] = name.len() as u8;
    r[33..33 + name.len()].copy_from_slice(name);
    r
}

fn dir(me: u32, parent: u32, entries: &[Vec<u8>]) -> Vec<u8> {
    let mut d = record(&[0], me, S as u32, 2);
    d.extend(record(&[1], parent, S as u32, 2));
    for e in entries {
        d.extend(e);
    }
    d.resize(S, 0);
    d
}

fn utf16(s: &str) -> Vec<u8> {
    s.encode_utf16().flat_map(|u| u.to_be_bytes()).collect()
}

fn descriptor(kind: u8, root: &[u8]) -> Vec<u8> {
    let mut d = vec![0; S];
    d[0] = kind;
    d[1..6].copy_from_slice(b"CD001");
    d[6] = 1;
    d[8..40].fill(b' ');
    d[8..12].copy_from_slice(b"TEST");
    d[40..72].fill(b' ');
    d[40..44].copy_from_slice(b"TINY");
    d[80..84].copy_from_slice(&(BLOCKS as u32).to_le_bytes());
    d[128..130].copy_from_slice(&(S as u16).to_le_bytes());
    d[156..156 + root.len()].copy_from_slice(root);
    d[813..830].copy_from_slice(b"1997011313531900\0");
    d
}

/// The volume, 2048 bytes a sector: HELLO.TXT, SUB/BIG.DAT and a hidden SUB/SECRET.BIN;
/// with `joliet` a second tree names the same files in mixed case.
fn iso(joliet: bool) -> Vec<u8> {
    let mut img = vec![0; BLOCKS * S];
    let mut put = |sector: usize, bytes: &[u8]| {
        img[sector * S..sector * S + bytes.len()].copy_from_slice(bytes)
    };
    put(16, &descriptor(1, &record(&[0], 20, S as u32, 2)));
    if joliet {
        let mut svd = descriptor(2, &record(&[0], 22, S as u32, 2));
        svd[88..91].copy_from_slice(b"%/E");
        put(17, &svd);
    }
    let mut term = vec![0; S];
    term[0] = 255;
    term[1..6].copy_from_slice(b"CD001");
    put(if joliet { 18 } else { 17 }, &term);
    put(
        20,
        &dir(
            20,
            20,
            &[
                record(b"HELLO.TXT;1", 24, 10, 0),
                record(b"SUB", 21, S as u32, 2),
            ],
        ),
    );
    put(
        21,
        &dir(
            21,
            20,
            &[
                record(b"BIG.DAT;1", 25, 5000, 0),
                record(b"SECRET.BIN;1", 24, 5, 1),
            ],
        ),
    );
    put(
        22,
        &dir(
            22,
            22,
            &[
                record(&utf16("Hello world.txt;1"), 24, 10, 0),
                record(&utf16("Sub"), 23, S as u32, 2),
            ],
        ),
    );
    put(
        23,
        &dir(23, 22, &[record(&utf16("Big data.dat;1"), 25, 5000, 0)]),
    );
    put(24, b"hello disc");
    put(25, &big());
    img
}

/// Wrap cooked sectors in 2352-byte Mode 1 or Mode 2/Form 1 sectors for the image fixture.
/// The header and user-data offsets are populated; the remaining bytes stay zero.
fn raw(cooked: &[u8], mode: u8) -> Vec<u8> {
    let mut out = Vec::new();
    for (i, s) in cooked.chunks(S).enumerate() {
        let mut r = vec![0u8; 2352];
        r[1..11].fill(0xFF);
        let lba = i + 150;
        let bcd = |v: usize| ((v / 10) << 4 | v % 10) as u8;
        r[12..15].copy_from_slice(&[bcd(lba / 75 / 60), bcd(lba / 75 % 60), bcd(lba % 75)]);
        r[15] = mode;
        if mode == 1 {
            r[16..16 + S].copy_from_slice(s);
        } else {
            r[18] = 0x08; // subheader: data, Form 1
            r[22] = 0x08;
            r[24..24 + S].copy_from_slice(s);
        }
        out.extend(r);
    }
    out
}

fn mem(bytes: Vec<u8>) -> Arc<dyn ReadAt> {
    Arc::new(Memory::new(bytes))
}

fn check(disc: &Disc, joliet: bool) {
    let v = disc.volume().unwrap();
    assert_eq!(
        (
            v.info.system.as_str(),
            v.info.volume.as_str(),
            v.info.blocks,
            v.info.joliet
        ),
        ("TEST", "TINY", BLOCKS as u32, joliet)
    );
    assert_eq!(
        v.info.created.unwrap().to_string(),
        "1997-01-13 13:53:19 +00:00"
    );
    let paths: Vec<&str> = v.entries().iter().map(|e| e.path.as_str()).collect();
    let (hello, bigname) = if joliet {
        assert_eq!(paths, ["Hello world.txt", "Sub", "Sub/Big data.dat"]);
        ("hello WORLD.TXT", "sub/big DATA.dat")
    } else {
        assert_eq!(paths, ["HELLO.TXT", "SUB", "SUB/BIG.DAT", "SUB/SECRET.BIN"]);
        assert!(v.find("sub\\secret.bin").unwrap().hidden);
        ("hello.txt", "/SUB/BIG.DAT")
    };
    let h = v.find(hello).unwrap();
    assert_eq!(
        (h.size, h.lba, h.date.unwrap().to_string().as_str()),
        (10, 24, "1997-01-13 13:53:19 +00:00")
    );
    assert_eq!(v.read(h).unwrap(), b"hello disc");
    let b = v.find(bigname).unwrap();
    assert_eq!(v.read(b).unwrap(), big());
    // A range across a sector boundary, and one at the very end.
    assert_eq!(v.read_range(b, 2040, 20).unwrap(), big()[2040..2060]);
    assert_eq!(v.read_range(b, 4999, 1).unwrap(), [big()[4999]]);
    assert!(v.read_range(b, 4999, 2).is_err());
    assert_eq!(v.files().count(), if joliet { 2 } else { 3 });
}

#[test]
fn cooked_iso() {
    for joliet in [false, true] {
        let d = Disc::from_raw(mem(iso(joliet)), "tiny.iso").unwrap();
        assert_eq!(
            (
                d.tracks[0].mode,
                d.tracks[0].sector_size,
                d.tracks[0].sectors
            ),
            (Mode::Mode1, 2048, BLOCKS as u32)
        );
        check(&d, joliet);
    }
}

#[test]
fn raw_mode1_and_mode2_form1() {
    for (mode, want) in [(1, Mode::Mode1), (2, Mode::Mode2)] {
        let d = Disc::from_raw(mem(raw(&iso(true), mode)), "tiny.bin").unwrap();
        assert_eq!((d.tracks[0].mode, d.tracks[0].sector_size), (want, 2352));
        assert!(!d.tracks[0].is_form2(24).unwrap());
        check(&d, true);
    }
}

#[test]
fn not_an_image() {
    assert!(Disc::from_raw(mem(vec![7; 1000]), "junk").is_err());
}

#[test]
fn cue_with_data_and_audio_in_one_file() {
    let mut bin = raw(&iso(false), 1);
    bin.extend(vec![0x11; 10 * 2352]);
    let cue = parse_cue("FILE \"tiny.bin\" BINARY\n TRACK 01 MODE1/2352\n  INDEX 01 00:00:00\n TRACK 02 AUDIO\n  PREGAP 00:02:00\n  INDEX 01 00:00:28\n").unwrap();
    let d = Disc::from_cue(&cue, |f| {
        assert_eq!(f.name, "tiny.bin");
        Ok(mem(bin.clone()))
    })
    .unwrap();
    assert_eq!(d.tracks.len(), 2);
    assert_eq!((d.tracks[0].sectors, d.tracks[0].start), (BLOCKS as u32, 0));
    let a = &d.tracks[1];
    assert_eq!(
        (a.mode, a.sectors, a.start, a.pregap),
        (Mode::Audio, 10, BLOCKS as u32 + 150, 150)
    );
    assert_eq!(a.read_raw(0, 1).unwrap(), vec![0x11; 2352]);
    assert!(a.read_user(0, 1).is_err());
    check(&d, false);
}

#[test]
fn cue_with_one_file_per_track() {
    let cue = parse_cue("FILE \"d.bin\" BINARY\n TRACK 01 MODE2/2352\n  INDEX 01 00:00:00\nFILE \"a.bin\" BINARY\n TRACK 02 AUDIO\n  INDEX 00 00:00:00\n  INDEX 01 00:02:00\n").unwrap();
    let d = Disc::from_cue(&cue, |f| {
        Ok(mem(if f.name == "d.bin" {
            raw(&iso(true), 2)
        } else {
            vec![0; 160 * 2352]
        }))
    })
    .unwrap();
    assert_eq!(
        (d.tracks[1].start, d.tracks[1].sectors),
        (BLOCKS as u32 + 150, 10)
    );
    check(&d, true);
}

/// A source that fetches on demand: reads fail with the chunks they need until they are in.
#[test]
fn chunk_cache_reports_what_is_missing() {
    let img = raw(&iso(true), 1);
    let cache = Arc::new(ChunkCache::new(7, img.len() as u64, 10_000));
    let fill = |m: &Missing| {
        for &i in &m.chunks {
            let r = cache.range(i);
            cache
                .insert(i, img[r.start as usize..r.end as usize].to_vec())
                .unwrap();
        }
    };
    let src: Arc<dyn ReadAt> = cache.clone();
    let mut fetches = 0;
    let volume = loop {
        match Disc::from_raw(src.clone(), "remote.bin").and_then(|d| d.volume()) {
            Ok(v) => break v,
            Err(e) => {
                let m = Missing::of(&e).expect("only missing chunks");
                assert_eq!(m.cache, 7);
                fill(m);
                fetches += 1;
            }
        }
    };
    assert!(fetches > 0 && cache.held() < img.len().div_ceil(10_000));
    assert_eq!(
        volume
            .read(volume.find("hello world.txt").unwrap())
            .unwrap(),
        b"hello disc"
    );
    assert!(cache.insert(0, vec![1, 2, 3]).is_err());
}
