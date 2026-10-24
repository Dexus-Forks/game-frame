#!/usr/bin/env python3
"""Write the files that run the original in an emulated x86-64 PC: QEMU, a Debian cloud image,
cloud-init, Wine with 32-bit libraries, Xvfb and a window manager, and the game's disc as a
CD-ROM drive.

Usage: emupc.py init DIR --image QCOW2 --key PUBKEY [--disc ISO] [--memory 4G] [--cpus 4]
                        [--ssh-port 2222] [--disk 20G] [--name gf-vm-PROJECT]
       emupc.py image                 print where to download the Debian cloud image

  DIR        the VM's folder, which git ignores (derived/vm)
  --image    the Debian cloud image (a .qcow2 you downloaded; see `emupc.py image`). It is
             never changed: the VM writes to an overlay, DIR/disk.qcow2
  --key      your SSH public key (~/.ssh/id_ed25519.pub), for the VM's user `gf`
  --disc     the game's disc as an ISO (tools/disc/discinfo.py IMAGE --iso FILE cuts the data
             track out of a BIN/CUE), attached as a CD-ROM drive
  --ssh-port the port on this machine that reaches the VM's SSH (default 2222). Each game has
             its own VM, so each needs its own port: init refuses a port that something else
             already listens on
  --name     the VM's host name and SSH alias (default gf-vm-PROJECT, from the project folder
             in GAMEFRAME_PROJECT, which ./gameframe sets; gf-vm without one)

It writes:

  DIR/cloud-init/user-data   the user, the key, and the first-boot steps: the i386
                             architecture, Wine, Xvfb, openbox, xdotool, rsync, 32-bit Mesa,
                             and a udev rule that makes the CD-ROM readable by everyone
  DIR/cloud-init/meta-data   the instance id and host name
  DIR/ssh_config             `ssh -F DIR/ssh_config NAME`, with IdentitiesOnly
  DIR/start.sh               makes the cloud-init seed, the overlay disk, then starts QEMU in the
                             background (serial log in DIR/serial.log, pid in DIR/qemu.pid)
  DIR/stop.sh                stops it

Why: when Wine on the host fails in ways no patch fixes (Wine under Rosetta on Apple silicon,
docs/knowhow/wine-wow64-rosetta-fault.md), an emulated x86-64 Linux with Wine runs the game
as Wine on a PC would. Each choice here fixed a failure (docs/knowhow/context-emulated-pc.md).
Files go over rsync and SSH because the cloud kernel has no 9p. The disc is a real CD-ROM
device because Wine on Linux reads the drive type and label from the device. Xvfb runs with a
window manager because without one no window becomes the foreground. Then DirectInput reads
no keys.

One VM per game: give each game project its own DIR (its derived/vm), port and name, and
never run one game in another game's VM. A shared VM ties a game to another project's
machine, its load and its changes, and puts one game's files inside the other's.

It does not download anything, install QEMU or copy the game. Copy the install folder in with
rsync over SSH. It needs qemu-system-x86_64 and qemu-img, and one of cloud-localds,
genisoimage, mkisofs, xorriso or hdiutil to make the seed. Standard library only.
"""
import os
import socket
import shlex
import stat
import sys

IMAGE_URL = 'https://cloud.debian.org/images/cloud/trixie/latest/debian-13-genericcloud-amd64.qcow2'
PACKAGES = ['wine', 'wine32:i386', 'wine64', 'xvfb', 'openbox', 'xdotool', 'x11-apps', 'imagemagick',
            'rsync', 'libgl1:i386', 'libglx-mesa0:i386', 'libgl1-mesa-dri:i386']
UDEV = 'KERNEL=="sr[0-9]*", MODE="0644"'


def user_data(key, name):
    pkgs = ' '.join(PACKAGES)
    return f"""#cloud-config
hostname: {name}
users:
  - name: gf
    shell: /bin/bash
    sudo: ALL=(ALL) NOPASSWD:ALL
    ssh_authorized_keys:
      - {key}
write_files:
  - path: /etc/udev/rules.d/60-cdrom-readable.rules
    content: '{UDEV}'
runcmd:
  - dpkg --add-architecture i386
  - apt-get update
  - DEBIAN_FRONTEND=noninteractive apt-get install -y {pkgs}
  - udevadm control --reload-rules
  - udevadm trigger --subsystem-match=block
  - touch /var/lib/gameframe-ready
"""


def start_sh(d, a):
    q = shlex.quote
    disc = f'  -drive file={q(os.path.abspath(a["disc"]))},media=cdrom,readonly=on \\\n' if a.get('disc') else ''
    return f"""#!/usr/bin/env bash
# Start the emulated PC that emupc.py wrote. Written by tools/live/emupc.py; run it again any time.
set -euo pipefail
cd "$(dirname "$0")"
image={q(os.path.abspath(a['image']))}
[[ -f "$image" ]] || {{ echo "no cloud image at $image (see: emupc.py image)" >&2; exit 1; }}
if [[ ! -f seed.iso ]]; then
  if command -v cloud-localds >/dev/null; then cloud-localds seed.iso cloud-init/user-data cloud-init/meta-data
  elif command -v genisoimage >/dev/null; then genisoimage -quiet -output seed.iso -volid cidata -joliet -rock cloud-init/user-data cloud-init/meta-data
  elif command -v mkisofs >/dev/null; then mkisofs -quiet -output seed.iso -volid cidata -joliet -rock cloud-init/user-data cloud-init/meta-data
  elif command -v xorriso >/dev/null; then xorriso -as mkisofs -quiet -output seed.iso -volid cidata -joliet -rock cloud-init/user-data cloud-init/meta-data
  elif command -v hdiutil >/dev/null; then hdiutil makehybrid -quiet -o seed.iso -iso -joliet -default-volume-name cidata cloud-init
  else echo "need cloud-localds, genisoimage, mkisofs, xorriso or hdiutil to make the seed" >&2; exit 1; fi
fi
[[ -f disk.qcow2 ]] || qemu-img create -q -f qcow2 -b "$image" -F qcow2 disk.qcow2 {a['disk']}
accel=tcg
[[ -w /dev/kvm ]] && accel=kvm
qemu-system-x86_64 -accel "$accel" -cpu max -m {a['memory']} -smp {a['cpus']} \\
  -drive file=disk.qcow2,if=virtio \\
  -drive file=seed.iso,media=cdrom,readonly=on \\
{disc}  -netdev user,id=n0,hostfwd=tcp:127.0.0.1:{a['ssh_port']}-:22 -device virtio-net-pci,netdev=n0 \\
  -display none -serial file:serial.log -daemonize -pidfile qemu.pid
echo "started; first boot installs Wine (10 to 30 minutes under TCG). Then: ssh -F $(pwd)/ssh_config {a['name']}"
echo "ready when this prints: ssh -F $(pwd)/ssh_config {a['name']} ls /var/lib/gameframe-ready"
"""


STOP = """#!/usr/bin/env bash
# Stop the emulated PC that start.sh started. Written by tools/live/emupc.py.
cd "$(dirname "$0")"
[[ -f qemu.pid ]] && kill "$(cat qemu.pid)" 2>/dev/null && rm -f qemu.pid && echo stopped || echo "not running"
"""


def ssh_config(a):
    return f"""Host {a['name']}
  HostName 127.0.0.1
  Port {a['ssh_port']}
  User gf
  IdentityFile {os.path.abspath(os.path.splitext(a['key'])[0])}
  IdentitiesOnly yes
  StrictHostKeyChecking accept-new
  UserKnownHostsFile {os.path.abspath(os.path.join(a['dir'], 'known_hosts'))}
"""


def write(path, text, executable=False):
    with open(path, 'w', encoding='utf-8') as f:
        f.write(text)
    if executable:
        os.chmod(path, os.stat(path).st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def default_name():
    project = os.environ.get('GAMEFRAME_PROJECT')
    if not project:
        return 'gf-vm'
    base = ''.join(c if c.isalnum() else '-' for c in os.path.basename(project.rstrip('/')).lower())
    return f'gf-vm-{base.strip("-") or "project"}'


def port_in_use(port, d):
    """True when something listens on 127.0.0.1:port and it is not this DIR's own running VM."""
    try:
        pid = int(open(os.path.join(d, 'qemu.pid')).read().strip())
        os.kill(pid, 0)
        return False
    except (OSError, ValueError):
        pass
    with socket.socket() as s:
        s.settimeout(0.5)
        return s.connect_ex(('127.0.0.1', int(port))) == 0


def init(argv):
    opt = lambda k, d=None: argv[argv.index(k) + 1] if k in argv and argv.index(k) + 1 < len(argv) else d
    args = [x for i, x in enumerate(argv) if not x.startswith('--') and (i == 0 or not argv[i - 1].startswith('--'))]
    if not args or not opt('--image') or not opt('--key'):
        sys.exit('usage: emupc.py init DIR --image QCOW2 --key PUBKEY [--disc ISO] ...')
    a = {'dir': args[0], 'image': opt('--image'), 'key': opt('--key'), 'disc': opt('--disc'),
         'memory': opt('--memory', '4G'), 'cpus': opt('--cpus', '4'), 'ssh_port': opt('--ssh-port', '2222'),
         'disk': opt('--disk', '20G'), 'name': opt('--name', default_name())}
    if port_in_use(a['ssh_port'], a['dir']):
        sys.exit(f'emupc.py: port {a["ssh_port"]} is in use, probably by another game\'s VM. '
                 'Give each game its own VM and port: --ssh-port N')
    key = open(a['key'], encoding='utf-8').read().strip()
    if not key.startswith(('ssh-', 'ecdsa-', 'sk-')):
        sys.exit(f'emupc.py: {a["key"]} is not an SSH public key (want the .pub file)')
    for f in ('image', 'disc'):
        if a[f] and not os.path.isfile(a[f]):
            sys.exit(f'emupc.py: no file {a[f]}')
    d = a['dir']
    os.makedirs(os.path.join(d, 'cloud-init'), exist_ok=True)
    write(os.path.join(d, 'cloud-init', 'user-data'), user_data(key, a['name']))
    write(os.path.join(d, 'cloud-init', 'meta-data'), f'instance-id: {a["name"]}\nlocal-hostname: {a["name"]}\n')
    write(os.path.join(d, 'ssh_config'), ssh_config(a))
    write(os.path.join(d, 'start.sh'), start_sh(d, a), executable=True)
    write(os.path.join(d, 'stop.sh'), STOP, executable=True)
    print(f'wrote {d}: cloud-init/, ssh_config, start.sh, stop.sh')
    print(f'next: {os.path.join(d, "start.sh")}, then rsync -e "ssh -F {os.path.join(d, "ssh_config")}" -a '
          f'GAME_FOLDER/ {a["name"]}:game/')
    return 0


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__.rstrip())
        return 0 if argv else 2
    if argv[0] == 'image':
        print(f'Download the Debian 13 cloud image (about 400 MB) yourself, for example:\n  curl -LO {IMAGE_URL}')
        return 0
    if argv[0] == 'init':
        return init(argv[1:])
    print(f'emupc.py: unknown command {argv[0]!r}; see --help', file=sys.stderr)
    return 2


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
