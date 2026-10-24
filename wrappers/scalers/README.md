# scalers: frame scalers for a finished picture

`scalers` scales a finished frame of `0x00RRGGBB` pixels by a whole multiple. `ddraw` uses it
to show a 640 × 480 picture in a larger window.

- `nearest`: each pixel becomes a `k × k` block.
- `scale2x`: Scale2x (EPX), once per doubling. Edges between flat colours become diagonal.
  Nothing is blurred. Other factors finish with `nearest`.
- `crt`: `nearest`, then each pixel's last row darkened as a scanline, and a faint red, green
  and blue column mask from 3×. It is a look. No monitor was measured.

GameFrame's [gameframe-scale](../../crates/scale/README.md) crate has the same first two
scalers for any pixel type. This crate stays here because `ddraw`'s `crt` filter adds the
column mask.

Tests: `cargo test --release --target HOST -p scalers` (`../README.md`). Licence: Apache-2.0
(code), CC0-1.0 (this text).
