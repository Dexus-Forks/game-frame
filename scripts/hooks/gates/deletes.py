"""The delete gate: a shell command that would delete or move a project's game folder.

reason(cwd, command) returns why the command is blocked, or None. A game folder is a GameFrame
project's target/ (game_folder.is_game_folder): the user's own copy of the game, which git does
not keep. The gate blocks:

- rm, rmdir, unlink, trash, gio trash and srm of it or a folder above it;
- mv of it, or of anything that holds it into a Trash folder (renaming a project passes);
- rm of everything in it (target/*, target/.);
- git clean -x or -X with -f, which deletes ignored files, over a tree that holds one;
- find with -delete or -exec rm/mv/trash and a target name, over a tree that holds one;
- cargo clean whose target dir is the game folder (--target-dir, CARGO_TARGET_DIR, or a
  project without .cargo/config.toml);
- build-folder cleaners (kondo, cargo sweep, cargo clean-all, npkill) over a tree that holds one;
- scripts that name it next to rmtree, send2trash, ~/.Trash or a Finder delete.

It follows `cd DIR` inside the command and expands ~ and $HOME. Other variables, globs other
than target/*, and paths built at run time are not resolved. A command that starts with
GAMEFRAME_DELETE_GAME=1 passes: the user decides to delete the game folder, and says so.
It is a safety net for mistakes, not a security boundary.
"""
import os
import re
import shlex

try:
    from .game_folder import is_game_folder
except ImportError:
    from game_folder import is_game_folder

ALLOW = 'GAMEFRAME_DELETE_GAME=1'
REMOVERS = {'rm', 'rmdir', 'unlink', 'trash', 'srm', 'trash-put', 'gtrash'}
SWEEPERS = {'kondo', 'npkill', 'cargo-sweep', 'cargo-clean-all'}
SCRIPTED = re.compile(r'rmtree|send2trash|\.Trash|move_to_trash|recycleItems|trashItem|'
                      r'removeItem|delete\s+(?:POSIX\s+)?file', re.I)
SEPARATORS = re.compile(r'&&|\|\||[;|\n&()]')
# A quick look before parsing: commands without any of these words pass at once.
MAY_DELETE = re.compile(r'\b(rm|rmdir|unlink|trash|srm|mv|find|cargo|kondo|npkill|clean)\b|'
                        + SCRIPTED.pattern, re.I)
DEPTH = 3  # how far below a deleted folder or a swept tree to look for game folders


def _expand(word, cwd):
    word = word.replace('${HOME}', '~').replace('$HOME', '~')
    return os.path.normpath(os.path.join(cwd, os.path.expanduser(word)))


def _holds(path, depth=DEPTH):
    """The first game folder at or below path, at most depth levels down, or None."""
    if not os.path.isdir(path) or os.path.islink(path):
        return None
    if is_game_folder(path):
        return path
    if depth == 0:
        return None
    try:
        entries = sorted(os.scandir(path), key=lambda e: e.name)
    except OSError:
        return None
    for e in entries:
        if e.is_dir(follow_symlinks=False) and not e.name.startswith('.'):
            found = _holds(e.path, depth - 1)
            if found:
                return found
    return None


def _target(word, cwd):
    """The game folder that removing word would take: itself, all of it, or one below it."""
    every = re.sub(r'/(\*|\.)$', '', word.rstrip('/')) if re.search(r'/(\*|\.)$', word) else None
    if every and is_game_folder(_expand(every, cwd)):
        return _expand(every, cwd)
    if any(c in word for c in '*?['):
        return None
    path = _expand(word, cwd)
    # rm -rf link/ follows a link to the main checkout's target/ (a worktree's link).
    return _holds(os.path.realpath(path) if word.endswith('/') else path)


def _words(segment):
    try:
        words = shlex.split(segment, comments=True)
    except ValueError:
        words = segment.split()
    env = {}
    while words and re.match(r'^[A-Za-z_]\w*=', words[0]):
        k, _, v = words.pop(0).partition('=')
        env[k] = v
    while words and words[0] in ('sudo', 'command', 'nohup', 'time', 'xargs', 'env'):
        words.pop(0)
    return env, words


def _args(words):
    return [w for w in words[1:] if not w.startswith('-')]


def _cargo_target(cwd, env, words):
    """The folder `cargo clean` would empty."""
    for i, w in enumerate(words):
        if w == '--target-dir' and i + 1 < len(words):
            return _expand(words[i + 1], cwd)
        if w.startswith('--target-dir='):
            return _expand(w.split('=', 1)[1], cwd)
    if env.get('CARGO_TARGET_DIR') or os.environ.get('CARGO_TARGET_DIR'):
        return _expand(env.get('CARGO_TARGET_DIR') or os.environ['CARGO_TARGET_DIR'], cwd)
    d, manifest = cwd, None
    while True:
        config = os.path.join(d, '.cargo', 'config.toml')
        if os.path.isfile(config):
            m = re.search(r'^\s*target-dir\s*=\s*["\']([^"\']+)["\']', open(config).read(), re.M)
            if m:
                return _expand(m.group(1), d)
        if manifest is None and os.path.isfile(os.path.join(d, 'Cargo.toml')):
            manifest = d
        if os.path.dirname(d) == d:
            break
        d = os.path.dirname(d)
    return os.path.join(manifest or cwd, 'target')


def _segment_reason(cwd, env, words, text):
    if not words:
        return None
    verb = os.path.basename(words[0])
    if verb in REMOVERS or (verb == 'gio' and words[1:2] == ['trash']):
        for w in _args(words[1:] if verb == 'gio' else words):
            hit = _target(w, cwd)
            if hit:
                return hit
    if verb == 'mv':
        # Renaming a project moves its game along; moving the folder itself, or anything to the
        # Trash, is how a cleaner loses it.
        args = _args(words)
        trash = bool(args) and re.search(r'(^|/)\.?Trash(es)?(/|$)', args[-1])
        for w in args[:-1]:
            hit = _target(w, cwd)
            if hit and (trash or os.path.realpath(_expand(w.rstrip('/*.'), cwd)) == os.path.realpath(hit)):
                return hit
    if verb == 'git' and 'clean' in words[1:4] and any(
            re.match(r'^-[a-zA-Z]*[xX]', w) for w in words) and any(
            re.match(r'^-[a-zA-Z]*f', w) or w == '--force' for w in words):
        # git clean -x deletes ignored files, and target/ is ignored.
        paths = [w for w in words[words.index('clean') + 1:] if not w.startswith('-')]
        for w in paths or ['.']:
            hit = _holds(_expand(w, cwd))
            if hit:
                return hit
    if verb == 'find' and ('-delete' in words or any(
            w == '-exec' and i + 1 < len(words) and os.path.basename(words[i + 1]) in REMOVERS | {'mv'}
            for i, w in enumerate(words))) and 'target' in text:
        for w in words[1:]:
            if w.startswith('-'):
                break
            hit = _holds(_expand(w, cwd))
            if hit:
                return hit
    if verb == 'cargo' and 'clean' in words[1:3] and 'clean-all' not in words:
        folder = _cargo_target(cwd, env, words)
        if is_game_folder(folder):
            return folder
    if verb in SWEEPERS or (verb == 'cargo' and words[1:2] in (['sweep'], ['clean-all'])):
        for w in _args(words[1:] if verb == 'cargo' else words) or ['.']:
            hit = _holds(_expand(w, cwd))
            if hit:
                return hit
    return None


def reason(cwd, command):
    """Why this shell command is blocked, or None."""
    if not command or command.lstrip().startswith(ALLOW) or not MAY_DELETE.search(command):
        return None
    hit = None
    for segment in SEPARATORS.split(command):
        env, words = _words(segment)
        if words[:1] == ['cd']:
            cwd = _expand(words[1] if len(words) > 1 else '~', cwd)
            continue
        hit = _segment_reason(cwd, env, words, segment)
        if hit:
            break
    if not hit and SCRIPTED.search(command):
        for word in re.findall(r'[\w~$/.{}-]*target\b', command):
            folder = _expand(word, cwd)
            if is_game_folder(folder):
                hit = folder
                break
    if not hit:
        return None
    return (f'{hit} is a GameFrame project\'s game folder: the user\'s own copy of the game, '
            'which git does not keep. It is not build output (Cargo builds into build/; a real '
            'Cargo target/ holds a CACHEDIR.TAG). This command would delete or move it.\n'
            'Leave it in place. If the user wants it gone, they say so, and the command starts '
            f'with {ALLOW}. See docs/knowhow/game-folder-deleted-as-build-output.md.')
