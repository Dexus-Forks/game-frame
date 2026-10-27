"""A build as a macOS application bundle: NAME.app with the programs in Contents/MacOS, a start
script as the bundle's executable, the resources and the icon in Contents/Resources, and an
Info.plist. The bundle is signed ad hoc (codesign -s -), which a build made on this Mac needs
to open with no warning; it is not a Developer ID signature, so another Mac asks first.
"""
import plistlib
import shutil
import subprocess
import tempfile
from pathlib import Path

import launch

START = 'gameframe-start'


def _icns(icon, out):
    """Copy an .icns, or make one from a PNG with sips and iconutil. False when that fails."""
    if icon.suffix.lower() == '.icns':
        shutil.copy2(icon, out)
        return True
    if not (shutil.which('sips') and shutil.which('iconutil')):
        return False
    with tempfile.TemporaryDirectory() as tmp:
        iconset = Path(tmp) / 'AppIcon.iconset'
        iconset.mkdir()
        for size in (16, 32, 128, 256, 512):
            for scale in (1, 2):
                name = f'icon_{size}x{size}{"@2x" if scale == 2 else ""}.png'
                px = str(size * scale)
                r = subprocess.run(['sips', '-z', px, px, str(icon), '--out', str(iconset / name)],
                                   capture_output=True)
                if r.returncode:
                    return False
        return subprocess.run(['iconutil', '-c', 'icns', str(iconset), '-o', str(out)],
                              capture_output=True).returncode == 0


def plist(build):
    info = {
        'CFBundleName': build['name'],
        'CFBundleDisplayName': build['name'],
        'CFBundleIdentifier': build['id'],
        'CFBundleVersion': build['version'],
        'CFBundleShortVersionString': build['version'],
        'CFBundleExecutable': START,
        'CFBundlePackageType': 'APPL',
        'CFBundleInfoDictionaryVersion': '6.0',
        'LSApplicationCategoryType': 'public.app-category.games',
        'LSMinimumSystemVersion': build.get('macos_minimum', '11.0'),
        'NSHighResolutionCapable': True,
    }
    return info


def make(build, folder, notes):
    """Write NAME.app in `folder` (a new, empty place). Returns the bundle's path."""
    app = folder / f'{build["name"]}.app'
    macos, res = app / 'Contents' / 'MacOS', app / 'Contents' / 'Resources'
    macos.mkdir(parents=True)
    res.mkdir()
    for prog in build['programs']:
        shutil.copy2(prog, macos / prog.name)
    for item in build['resources']:
        (shutil.copytree if item.is_dir() else shutil.copy2)(item, res / item.name)
    info = plist(build)
    if build.get('icon'):
        if _icns(build['icon'], res / 'AppIcon.icns'):
            info['CFBundleIconFile'] = 'AppIcon'
        else:
            notes.append(f'no icon: could not make an .icns from {build["icon"]}')
    with open(app / 'Contents' / 'Info.plist', 'wb') as f:
        plistlib.dump(info, f)
    script = macos / START
    script.write_text(launch.sh_script(build['name'], build['start'], build['args'], build['data'],
                                       '"$(cd "$here/../Resources" && pwd)"'))
    script.chmod(0o755)
    return app


def sign(app, notes):
    if not shutil.which('codesign'):
        notes.append('not signed: codesign is missing')
        return
    r = subprocess.run(['codesign', '--force', '--deep', '--sign', '-', str(app)],
                       capture_output=True, text=True)
    if r.returncode:
        notes.append('not signed: ' + (r.stderr.strip().splitlines() or ['codesign failed'])[-1])
