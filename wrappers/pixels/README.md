# pixels: 16-bit colour of 1990s PC games

`pixels` turns RGB555 and RGB565 pixels into 8-bit RGB. Each channel widens by bit replication
(5 bits `abcde` become `abcdeabc`), as the chips' own colour paths did. It also reads 16-bit,
8-bit paletted, BGRA and Windows DIB pictures into RGBA. `glide` and `raster` use it.

Tests: `cargo test --release --target HOST -p pixels` (`../README.md`). Licence: Apache-2.0
(code), CC0-1.0 (this text).
