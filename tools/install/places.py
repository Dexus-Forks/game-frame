"""Where an installed build goes on each system, and where its player's data goes.

The programs go in a GameFrame folder among the system's applications. The data (settings,
library, saves) goes in the user's own data folder, so a new install never touches it.

| System  | Programs                                   | Data                                          |
|---------|--------------------------------------------|-----------------------------------------------|
| macos   | /Applications/GameFrame/NAME.app (else ~/Applications/GameFrame) | ~/Library/Application Support/GameFrame/NAME |
| windows | %LOCALAPPDATA%\\Programs\\GameFrame\\NAME    | %APPDATA%\\GameFrame\\NAME                      |
| linux   | $XDG_DATA_HOME/GameFrame/NAME (~/.local/share) | $XDG_CONFIG_HOME/GameFrame/NAME (~/.config) |

Windows and Linux install per user, with no administrator rights. On macOS /Applications is
used when this user can write to it, as an administrator can.
"""
import os
import sys
from pathlib import Path


def system():
    if sys.platform == 'darwin':
        return 'macos'
    if sys.platform.startswith('win'):
        return 'windows'
    return 'linux'


def _env(name, default):
    value = os.environ.get(name)
    return Path(value) if value else Path(default).expanduser()


def apps_folder(kind):
    """The GameFrame folder that holds the installed programs."""
    if kind == 'macos':
        shared = Path('/Applications')
        if os.access(shared, os.W_OK):
            return shared / 'GameFrame'
        return Path('~/Applications/GameFrame').expanduser()
    if kind == 'windows':
        return _env('LOCALAPPDATA', '~/AppData/Local') / 'Programs' / 'GameFrame'
    return _env('XDG_DATA_HOME', '~/.local/share') / 'GameFrame'


def data_folder(kind):
    """The GameFrame folder that holds each build's settings, library and saves."""
    if kind == 'macos':
        return Path('~/Library/Application Support/GameFrame').expanduser()
    if kind == 'windows':
        return _env('APPDATA', '~/AppData/Roaming') / 'GameFrame'
    return _env('XDG_CONFIG_HOME', '~/.config') / 'GameFrame'
