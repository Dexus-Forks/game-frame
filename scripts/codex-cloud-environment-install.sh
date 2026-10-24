#!/usr/bin/env bash
set -euo pipefail
cd /workspace/game-frame
mkdir -p /workspace/toolchains /workspace/cache
python3 -m venv .venv
.venv/bin/pip install -r requirements.txt -r requirements-live.txt -r requirements-llm.txt ziglang==0.15.2
ln -sfn /workspace/game-frame/.venv/lib/python3.12/site-packages/ziglang/zig /workspace/toolchains/zig
cat > /workspace/toolchains/gameframe-env.sh <<'ENV'
export PATH=/workspace/toolchains:/workspace/toolchains/cargo/bin:$PATH
export CARGO_HOME=/workspace/toolchains/cargo
export RUSTUP_HOME=/workspace/toolchains/rustup
export CARGO_BUILD_JOBS=4
export PYTHONDONTWRITEBYTECODE=1
export ZIG_GLOBAL_CACHE_DIR=/workspace/cache/zig
export XDG_CACHE_HOME=/workspace/cache
export JAVA_HOME=/workspace/toolchains/jdk-package/usr/lib/jvm/java-21-openjdk-amd64
export XDG_CONFIG_HOME=/workspace/config
export GHIDRA_INSTALL_DIR=/workspace/toolchains/ghidra_12.0.4_PUBLIC
ENV
source /workspace/toolchains/gameframe-env.sh
if ! command -v rustup >/dev/null; then
  curl -fL https://sh.rustup.rs -o /workspace/cache/rustup-init.sh
  sh /workspace/cache/rustup-init.sh -y --no-modify-path --profile minimal --default-toolchain 1.99.1
fi
rustup target add i686-pc-windows-gnu wasm32-unknown-unknown
if [[ ! -x "$GHIDRA_INSTALL_DIR/support/analyzeHeadless" ]]; then
  curl -fL https://github.com/NationalSecurityAgency/ghidra/releases/download/Ghidra_12.0.4_build/ghidra_12.0.4_PUBLIC_20260303.zip -o /workspace/cache/ghidra.zip
  echo 'c3b458661d69e26e203d739c0c82d143cc8a4a29d9e571f099c2cf4bda62a120  /workspace/cache/ghidra.zip' | sha256sum -c -
  unzip -q /workspace/cache/ghidra.zip -d /workspace/toolchains
fi
if ! command -v xxd >/dev/null || [[ ! -x "$JAVA_HOME/bin/javac" ]]; then
  mkdir -p /workspace/cache/apt/lists/partial /workspace/cache/apt/archives/partial /workspace/config
  printf 'deb https://deb.debian.org/debian trixie main\n' > /workspace/cache/apt/sources.list
  apt-get -o Dir::Etc::sourcelist=/workspace/cache/apt/sources.list -o Dir::Etc::sourceparts=- -o Dir::State::lists=/workspace/cache/apt/lists -o Dir::Cache::archives=/workspace/cache/apt/archives update
  if ! command -v xxd >/dev/null; then
    (cd /workspace/cache && apt-get -o Dir::Etc::sourcelist=/workspace/cache/apt/sources.list -o Dir::Etc::sourceparts=- -o Dir::State::lists=/workspace/cache/apt/lists download xxd)
    dpkg-deb -x /workspace/cache/xxd_*.deb /workspace/toolchains/xxd-package
    ln -sfn /workspace/toolchains/xxd-package/usr/bin/xxd /workspace/toolchains/xxd
  fi
  if [[ ! -x "$JAVA_HOME/bin/javac" ]]; then
    (cd /workspace/cache && apt-get -o Dir::Etc::sourcelist=/workspace/cache/apt/sources.list -o Dir::Etc::sourceparts=- -o Dir::State::lists=/workspace/cache/apt/lists download openjdk-21-jdk-headless openjdk-21-jre-headless)
    dpkg-deb -x /workspace/cache/openjdk-21-jre-headless_*.deb /workspace/toolchains/jdk-package
    dpkg-deb -x /workspace/cache/openjdk-21-jdk-headless_*.deb /workspace/toolchains/jdk-package
  fi
fi
mkdir -p /workspace/config
npm --prefix site ci --cache /workspace/cache/npm
cat > /workspace/toolchains/install-dos-tools.sh <<'DOSINSTALL'
#!/usr/bin/env bash
set -euo pipefail
source /workspace/toolchains/gameframe-env.sh
mkdir -p /workspace/cache/apt/lists/partial /workspace/cache/apt/archives/partial /workspace/toolchains/dos-deps
printf 'deb https://deb.debian.org/debian trixie main\n' > /workspace/cache/apt/sources.list
apt_options=(-o Dir::Etc::sourcelist=/workspace/cache/apt/sources.list -o Dir::Etc::sourceparts=- -o Dir::State::lists=/workspace/cache/apt/lists -o Dir::Cache::archives=/workspace/cache/apt/archives)
apt-get "${apt_options[@]}" update
apt-get "${apt_options[@]}" -o Debug::NoLocking=true --download-only --no-install-recommends -y install innoextract 7zip libtool autogen autoconf automake libncurses-dev nasm libsdl2-dev libsdl2-net-dev libpcap-dev libslirp-dev libfluidsynth-dev libavformat-dev libavcodec-dev libswscale-dev libfreetype-dev libxkbfile-dev libxrandr-dev
for package in /workspace/cache/apt/archives/*.deb; do dpkg-deb -x "$package" /workspace/toolchains/dos-deps; done
# Development-package symlinks can refer to runtime libraries already supplied by Debian.
python3 - <<'PYLIBS'
from pathlib import Path
root=Path('/workspace/toolchains/dos-deps/usr/lib/x86_64-linux-gnu')
for p in root.glob('*'):
    if p.is_symlink() and not p.exists():
        target=p.parent/p.readlink()
        system=Path('/usr/lib/x86_64-linux-gnu')/p.readlink().name
        if system.exists() and not target.exists() and target.parent==p.parent:
            target.symlink_to(system)
PYLIBS
ln -sfn /workspace/toolchains/dos-deps/usr/bin/innoextract /workspace/toolchains/innoextract
ln -sfn /workspace/toolchains/dos-deps/usr/bin/aclocal-1.17 /workspace/toolchains/aclocal
ln -sfn /workspace/toolchains/dos-deps/usr/bin/automake-1.17 /workspace/toolchains/automake
cat > /workspace/toolchains/7z <<'SEVENZIP'
#!/bin/sh
exec /workspace/toolchains/dos-deps/usr/lib/7zip/7z "$@"
SEVENZIP
cat > /workspace/toolchains/sdl2-config <<'SDLCONFIG'
#!/bin/sh
exec /workspace/toolchains/dos-deps/usr/bin/sdl2-config --prefix=/workspace/toolchains/dos-deps/usr "$@"
SDLCONFIG
chmod +x /workspace/toolchains/7z /workspace/toolchains/sdl2-config
sed 's|/usr/share/autoconf|/workspace/toolchains/dos-deps/usr/share/autoconf|g' /workspace/toolchains/dos-deps/usr/share/autoconf/autom4te.cfg > /workspace/toolchains/autom4te.cfg
cat > /workspace/toolchains/dos-build-env.sh <<'DOSBUILDENV'
export DOS_DEPS=/workspace/toolchains/dos-deps
export PATH=/workspace/toolchains:/workspace/toolchains/dos-deps/usr/bin:$PATH
export LD_LIBRARY_PATH=/workspace/toolchains/dos-deps/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
export PKG_CONFIG_PATH=/workspace/toolchains/dos-deps/usr/lib/x86_64-linux-gnu/pkgconfig:/workspace/toolchains/dos-deps/usr/share/pkgconfig
export PKG_CONFIG_SYSROOT_DIR=/workspace/toolchains/dos-deps
export autom4te_perllibdir=/workspace/toolchains/dos-deps/usr/share/autoconf
export AC_MACRODIR=/workspace/toolchains/dos-deps/usr/share/autoconf
export AUTOM4TE_CFG=/workspace/toolchains/dos-deps/usr/share/autoconf/autom4te.cfg
export AUTOM4TE=/workspace/toolchains/dos-deps/usr/bin/autom4te
export AUTOMAKE_LIBDIR=/workspace/toolchains/dos-deps/usr/share/automake-1.17
export ACLOCAL_PATH=/workspace/toolchains/dos-deps/usr/share/aclocal
export CFLAGS='-O2 -I/workspace/toolchains/dos-deps/usr/include -I/workspace/toolchains/dos-deps/usr/include/x86_64-linux-gnu'
export CXXFLAGS='-O2 -I/workspace/toolchains/dos-deps/usr/include -I/workspace/toolchains/dos-deps/usr/include/x86_64-linux-gnu'
export LDFLAGS='-L/workspace/toolchains/dos-deps/usr/lib/x86_64-linux-gnu -Wl,-rpath,/workspace/toolchains/dos-deps/usr/lib/x86_64-linux-gnu'
export PERL5LIB=/workspace/toolchains/dos-deps/usr/share/automake-1.17:/workspace/toolchains/dos-deps/usr/share/autoconf
export ACLOCAL_AUTOMAKE_DIR=/workspace/toolchains/dos-deps/usr/share/aclocal-1.17
export M4=/workspace/toolchains/dos-deps/usr/bin/m4
export AUTOM4TE_CFG=/workspace/toolchains/autom4te.cfg
export trailer_m4=/workspace/toolchains/dos-deps/usr/share/autoconf/autoconf/trailer.m4
DOSBUILDENV
rustup target add i686-pc-windows-gnu wasm32-unknown-unknown
source_root=/workspace/toolchains/dosbox-x-src
source_commit=f6b476e9c97e204bd9f057395388c8a8abd44d73
if [[ ! -d "$source_root" ]]; then
    git clone --no-checkout --depth 1 https://github.com/joncampbell123/dosbox-x.git "$source_root"
    git -C "$source_root" fetch --depth 1 origin "$source_commit"
    git -C "$source_root" checkout --detach "$source_commit"
fi
[[ "$(git -C "$source_root" rev-parse HEAD)" == "$source_commit" ]] || { echo 'Unexpected DOSBox-X source revision; preserve it and investigate.' >&2; exit 1; }
if [[ ! -x "$source_root/src/dosbox-x" ]]; then
    (source /workspace/toolchains/dos-build-env.sh; cd "$source_root"; ./build-debug-sdl2 --disable-x11 --disable-opengl)
fi
grep -q '^#define C_DEBUG 1' "$source_root/config.h"
grep -q '^#define C_HEAVY_DEBUG 1' "$source_root/config.h"
cat > /workspace/toolchains/dosbox-x <<'DOSBOXWRAPPER'
#!/bin/sh
export LD_LIBRARY_PATH=/workspace/toolchains/dos-deps/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
exec /workspace/toolchains/dosbox-x-src/src/dosbox-x "$@"
DOSBOXWRAPPER
chmod +x /workspace/toolchains/dosbox-x
innoextract --version
7z i
# DOSBox-X intentionally returns 1 for -version, even after a successful version report.
dos_version_rc=0
dos_version="$(dosbox-x -version 2>&1)" || dos_version_rc=$?
printf '%s\n' "$dos_version"
[[ $dos_version_rc == 0 || $dos_version_rc == 1 ]]
[[ "$dos_version" == *"DOSBox-X version 2026.10.01 "* ]]
DOSINSTALL
bash /workspace/toolchains/install-dos-tools.sh
# Registers the repository hook and writes ignored .env only when absent.
# All doctor dependencies, including DOSBox-X and extraction tools, are required here.
scripts/setup.sh
.venv/bin/python -c 'import unicorn, pefile, capstone, frida, anthropic'
zig version
java -version
rustc --version


"$JAVA_HOME/bin/javac" -version
java -cp "$GHIDRA_INSTALL_DIR/support/LaunchSupport.jar" LaunchSupport "$GHIDRA_INSTALL_DIR" -jdk_home -save
