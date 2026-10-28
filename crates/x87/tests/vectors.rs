//! The crate against real x87 instructions run under GameFrame's emulator (`make_vectors.py`).
//! `GAMEFRAME_X87_VECTORS=FILE` checks a larger file made with `--cases N` as well.

use gameframe_x87::{Control, Ext};

fn bytes80(hex: &str) -> [u8; 10] {
    let mut b = [0u8; 10];
    for (i, out) in b.iter_mut().rev().enumerate() {
        *out = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap();
    }
    b
}

fn hex80(v: Ext) -> String {
    match v.to_bytes() {
        Some(b) => b.iter().rev().map(|x| format!("{x:02x}")).collect(),
        None => format!("{v:?}"),
    }
}

fn int(hex: &str) -> u64 {
    u64::from_str_radix(hex, 16).unwrap()
}

/// Check every line of a vector file; return the number of values compared.
fn check(text: &str) -> usize {
    let mut compared = 0;
    let mut failures = Vec::new();
    for line in text
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        let f: Vec<&str> = line.split_whitespace().collect();
        let cw = Control::from_word(int(f[1]) as u16);
        let a = Ext::from_bytes(bytes80(f[2])).unwrap();
        let mut expect = |what: &str, got: String, want: &str| {
            compared += 1;
            if got != want && failures.len() < 20 {
                failures.push(format!("{what}: got {got}, want {want}  ({line})"));
            }
        };
        if f[0] == "conv" {
            expect(
                "fst qword",
                format!("{:016x}", a.to_f64_with(cw).to_bits()),
                f[3],
            );
            expect(
                "fst dword",
                format!("{:08x}", a.to_f32_with(cw).to_bits()),
                f[4],
            );
            expect("fist word", format!("{:04x}", a.fist_i16(cw) as u16), f[5]);
            expect("fist dword", format!("{:08x}", a.fist_i32(cw) as u32), f[6]);
            expect(
                "fistp qword",
                format!("{:016x}", a.fist_i64(cw) as u64),
                f[7],
            );
            expect("frndint", hex80(a.frndint(cw)), f[8]);
            expect(
                "fsqrt",
                hex80(a.fabs().checked_fsqrt_with(cw).unwrap()),
                f[9],
            );
            continue;
        }
        let b = Ext::from_bytes(bytes80(f[3])).unwrap();
        let got = match f[0] {
            "add" => a.fadd_with(b, cw),
            "sub" => a.fsub_with(b, cw),
            "mul" => a.fmul_with(b, cw),
            "div" => match a.checked_fdiv_with(b, cw) {
                Some(v) => v,
                None => continue, // a zero divisor: an infinity or a NaN, not modelled
            },
            op => panic!("unknown operation {op}"),
        };
        expect(f[0], hex80(got), f[4]);
    }
    assert!(
        failures.is_empty(),
        "{} differences:\n{}",
        failures.len(),
        failures.join("\n")
    );
    compared
}

#[test]
fn equal_to_the_x87_under_the_emulator() {
    let n = check(include_str!("vectors.txt"));
    assert!(n > 3000, "only {n} values compared");
}

#[test]
fn a_larger_vector_file_from_the_environment() {
    if let Ok(path) = std::env::var("GAMEFRAME_X87_VECTORS") {
        let text = std::fs::read_to_string(&path).expect("GAMEFRAME_X87_VECTORS");
        println!("{} values equal in {path}", check(&text));
    }
}
