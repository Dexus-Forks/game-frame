#!/bin/sh
# zcc: rustc's linker for 32-bit Windows DLLs (i686-pc-windows-gnu). It links through `zig cc`
# and needs no MinGW. Use it for replacement libraries that a game imports by exact name, such
# as glide.dll with `_grGlideInit@0`, S3DTKW.dll with `S3DTK_InitLib` or a ddraw.dll.
#
# Usage: in the crate's .cargo/config.toml (or CARGO_TARGET_I686_PC_WINDOWS_GNU_LINKER):
#
#   [target.i686-pc-windows-gnu]
#   linker = "/path/to/gameframe/scripts/zcc.sh"      # ./gameframe root prints the path
#
#   ZCC_DEF_DIR=DIR cargo build --release --target i686-pc-windows-gnu
#
# Set panic = "abort" in the crate's profiles. zcc runs `zig cc -target x86-windows-gnu` with
# rustc's arguments. It drops the ones zig's lld does not take (-fno-use-linker-plugin,
# --disable-auto-image-base, --large-address-aware, -O1, -lgcc, -lmsvcrt, -lmingwex,
# -lmingw32, -l:libpthread.a). It uses zig's libunwind for -lgcc_eh. It leaves out rsbegin.o
# and rsend.o, which register DWARF frames that panic=abort never needs.
#
# Exports: rustc writes its own .def file, which exports a stdcall function undecorated
# (`grGlideInit`). When DIR/NAME.def exists (NAME from `-o .../NAME.dll`), zcc links with that
# file instead, or else DIR/*/NAME.def (a workspace with one .def in each crate, as
# wrappers/). The DLL then exports exactly the names the game imports. A line such as
# `_grGlideInit@0=grGlideInit@0` exports the decorated name a 1990s EXE asks for. ZCC_DEF
# names one .def file for every DLL instead.
#
# Inputs: rustc's linker arguments. Output: the file rustc asked for (-o). It does not install
# zig or the Rust target. Add the target with `rustup target add i686-pc-windows-gnu`, and zig
# with scripts/setup.sh --install. tests/run.sh tests it with a two-export crate.
if [ "$#" -eq 0 ] || [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
  sed -n '2,28p' "$0" | sed 's/^# \{0,1\}//'
  [ "$#" -eq 0 ] && exit 2 || exit 0
fi
name=
prev=
for a in "$@"; do
  [ "$prev" = "-o" ] && name=$(basename "$a" .dll)
  prev=$a
done
def=${ZCC_DEF:-}
if [ -z "$def" ] && [ -n "${ZCC_DEF_DIR:-}" ]; then
  if [ -f "$ZCC_DEF_DIR/$name.def" ]; then
    def="$ZCC_DEF_DIR/$name.def"
  else
    for f in "$ZCC_DEF_DIR"/*/"$name.def"; do
      [ -f "$f" ] && def=$f && break
    done
  fi
fi
for a in "$@"; do
  shift
  case "$a" in
    -Wl,*.def) if [ -n "$def" ]; then set -- "$@" "$def"; else set -- "$@" "$a"; fi ;;
    -fno-use-linker-plugin|-Wl,--disable-auto-image-base|-Wl,--large-address-aware|-Wl,-O1) ;;
    -lgcc|-lmsvcrt|-lmingwex|-lmingw32|-l:libpthread.a|*/rsbegin.o|*/rsend.o) ;;
    -lgcc_eh) set -- "$@" -lunwind ;;
    *) set -- "$@" "$a" ;;
  esac
done
exec zig cc -target x86-windows-gnu "$@"
