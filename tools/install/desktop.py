"""A build as a folder on Linux or Windows: the programs and resources in GameFrame/NAME/, a start
script beside them, and a menu entry.

Linux: `start` (a shell script) and NAME.desktop in the folder; the menu entry is a copy of the
.desktop file in $XDG_DATA_HOME/applications. Windows: NAME.cmd in the folder, for a terminal,
and a Start menu shortcut (Programs/GameFrame/NAME.lnk) made with PowerShell, which starts the
program itself in the data folder with no console window.
"""
import os
import shutil
import subprocess
from pathlib import Path

import launch


def _copy(build, folder):
    folder.mkdir(parents=True)
    for prog in build['programs']:
        shutil.copy2(prog, folder / prog.name)
    for item in build['resources']:
        (shutil.copytree if item.is_dir() else shutil.copy2)(item, folder / item.name)
    if build.get('icon'):
        shutil.copy2(build['icon'], folder / ('icon' + build['icon'].suffix.lower()))


def desktop_entry(build, folder):
    icon = f'Icon={folder / ("icon" + build["icon"].suffix.lower())}\n' if build.get('icon') else ''
    return ('[Desktop Entry]\nType=Application\n'
            f'Name={build["name"]}\n'
            f'Exec="{folder / "start"}"\n'
            f'{icon}Categories=Game;\nTerminal=false\n')


def make_linux(build, staging, final):
    """Write the folder in `staging`; its paths name `final`, where it is moved afterwards."""
    _copy(build, staging)
    script = staging / 'start'
    script.write_text(launch.sh_script(build['name'], build['start'], build['args'], build['data'], '"$here"'))
    script.chmod(0o755)
    (staging / f'{build["slug"]}.desktop').write_text(desktop_entry(build, final))


def menu_linux(build, final, notes):
    apps = Path(os.environ.get('XDG_DATA_HOME') or '~/.local/share').expanduser() / 'applications'
    apps.mkdir(parents=True, exist_ok=True)
    entry = apps / f'gameframe-{build["slug"]}.desktop'
    shutil.copy2(final / f'{build["slug"]}.desktop', entry)
    notes.append(f'menu entry {entry}')


def make_windows(build, staging):
    _copy(build, staging)
    (staging / f'{build["name"]}.cmd').write_text(
        launch.cmd_script(build['name'], build['start'], build['args'], build['data']), newline='')


def menu_windows(build, final, notes):
    shell = shutil.which('powershell') or shutil.which('pwsh')
    if not shell:
        notes.append('no Start menu shortcut: PowerShell is missing')
        return
    menu = Path(os.environ.get('APPDATA') or '~/AppData/Roaming').expanduser() / \
        'Microsoft' / 'Windows' / 'Start Menu' / 'Programs' / 'GameFrame'
    menu.mkdir(parents=True, exist_ok=True)
    link = menu / f'{build["name"]}.lnk'
    args = subprocess.list2cmdline(launch.resolve(build['args'], build['data'], final))

    def q(text):
        return "'" + str(text).replace("'", "''") + "'"
    ps = (f'$s = (New-Object -ComObject WScript.Shell).CreateShortcut({q(link)}); '
          f'$s.TargetPath = {q(final / build["start"])}; $s.Arguments = {q(args)}; '
          f'$s.WorkingDirectory = {q(build["data"])}; $s.Save()')
    r = subprocess.run([shell, '-NoProfile', '-Command', ps], capture_output=True, text=True)
    notes.append(f'Start menu shortcut {link}' if r.returncode == 0
                 else 'no Start menu shortcut: ' + (r.stderr.strip() or 'PowerShell failed'))
