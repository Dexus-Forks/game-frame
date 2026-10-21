//! Encode test tones with FFmpeg's `wmav2` encoder, decode them with this crate and with FFmpeg,
//! and compare. Skipped when `ffmpeg` and `ffprobe` are not installed.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use gameframe_wma::{Decoder, Params, WMAV2};

const MIN_SNR_DB: f64 = 60.0;

fn have(tool: &str) -> bool {
    Command::new(tool)
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn run(cmd: &mut Command) -> Vec<u8> {
    let out = cmd.stderr(Stdio::inherit()).output().expect("run tool");
    assert!(out.status.success(), "{cmd:?} failed");
    out.stdout
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gameframe-wma-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// FFmpeg's encoder writes `flags2` = 1 (exponent VLCs, no bit reservoir, fixed block length).
const FFMPEG_FLAGS: u16 = 1;

struct Encoded {
    params: Params,
    packets: Vec<u8>,
    reference: Vec<f32>,
}

fn encode(dir: &Path, source: &str, channels: u16, rate: u32, bitrate: u32) -> Encoded {
    let wma = dir.join("tone.wma");
    run(Command::new("ffmpeg").args(["-hide_banner", "-loglevel", "error", "-y", "-f", "lavfi", "-i", source])
        .args(["-ac", &channels.to_string(), "-ar", &rate.to_string(), "-c:a", "wmav2", "-b:a", &bitrate.to_string()])
        .arg(&wma));
    let packets = run(Command::new("ffmpeg").args(["-hide_banner", "-loglevel", "error", "-i"]).arg(&wma)
        .args(["-map", "0:a", "-c", "copy", "-f", "data", "-"]));
    let sizes = String::from_utf8(run(Command::new("ffprobe").args(["-v", "error", "-select_streams", "a",
        "-show_entries", "packet=size", "-of", "csv=p=0"]).arg(&wma))).unwrap();
    let block_align: u16 = sizes.lines().next().unwrap().trim().parse().unwrap();
    let reference = run(Command::new("ffmpeg").args(["-hide_banner", "-loglevel", "error", "-i"]).arg(&wma)
        .args(["-f", "f32le", "-acodec", "pcm_f32le", "-"]))
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
        .collect();
    let params = Params {
        tag: WMAV2,
        channels,
        sample_rate: rate,
        avg_bytes_per_sec: bitrate / 8,
        block_align,
        flags: FFMPEG_FLAGS,
    };
    Encoded { params, packets, reference }
}

fn decode(e: &Encoded) -> Vec<f32> {
    let mut decoder = Decoder::new(&e.params).expect("decoder");
    let mut pcm = Vec::new();
    for packet in e.packets.chunks(usize::from(e.params.block_align)) {
        decoder.decode_packet(packet, &mut pcm).expect("packet");
    }
    pcm
}

fn snr_db(reference: &[f32], ours: &[f32]) -> f64 {
    let (mut signal, mut noise) = (0f64, 0f64);
    for (&r, &o) in reference.iter().zip(ours) {
        signal += f64::from(r).powi(2);
        noise += (f64::from(r) - f64::from(o)).powi(2);
    }
    if noise == 0.0 { f64::INFINITY } else { 10.0 * (signal / noise).log10() }
}

#[test]
fn matches_ffmpeg() {
    if !have("ffmpeg") || !have("ffprobe") {
        eprintln!("skipping: ffmpeg or ffprobe not installed");
        return;
    }
    let cases = [
        ("sine=frequency=440:duration=2", 2, 44_100, 128_000),
        ("sine=frequency=1000:duration=1", 1, 22_050, 32_000),
        ("anoisesrc=d=1:c=pink:a=0.3", 2, 44_100, 96_000),
    ];
    for (i, (source, channels, rate, bitrate)) in cases.into_iter().enumerate() {
        let dir = temp_dir(&i.to_string());
        let e = encode(&dir, source, channels, rate, bitrate);
        let ours = decode(&e);
        let n = ours.len().min(e.reference.len());
        assert!(n > e.reference.len() / 2, "case {i}: decoded {} of {} samples", ours.len(), e.reference.len());
        let snr = snr_db(&e.reference[..n], &ours[..n]);
        eprintln!("case {i} ({source}, {channels} ch, {rate} Hz): {n} samples, SNR {snr:.1} dB");
        assert!(snr >= MIN_SNR_DB, "case {i}: SNR {snr:.1} dB is below {MIN_SNR_DB} dB");
        std::fs::remove_dir_all(&dir).ok();
    }
}
