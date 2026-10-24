"""The signs needs.py reads: imported libraries, section names, files and strings, each with the
need it shows. A need is `kind:value`. The kinds are cpu, os, api, lib, drm, store, media and
extender. tools/live/harness.py matches needs to modules by these keys, so keep them stable;
add a row when a game shows a new sign.

    from needs_signs import IMPORTS, SECTIONS, FILES, STRINGS
"""
import re

# Imported DLL (lower case, a regular expression on the whole name) -> need.
IMPORTS = [
    (r'ddraw\.dll', 'api:directdraw'), (r'd3dim\.dll|d3drm\.dll', 'api:direct3d-retained'),
    (r'd3d8\.dll', 'api:direct3d-8'), (r'd3d9\.dll', 'api:direct3d-9'),
    (r'd3d1[01](_1)?\.dll', 'api:direct3d-11'), (r'dxgi\.dll', 'api:dxgi'), (r'd3d12\.dll', 'api:direct3d-12'),
    (r'd3dx9_\d+\.dll', 'lib:d3dx9'), (r'd3dx1[01]_\d+\.dll', 'lib:d3dx11'), (r'd3dcompiler_\d+\.dll', 'lib:d3dcompiler'),
    (r'opengl32\.dll', 'api:opengl'), (r'vulkan-1\.dll', 'api:vulkan'),
    (r'glide\.dll', 'api:glide'), (r'glide2x\.dll', 'api:glide2'), (r'glide3x\.dll', 'api:glide3'),
    (r'ati3dcif\.dll', 'api:ati3dcif'), (r's3dtkw\.dll', 'api:s3d'), (r'sgl\.dll', 'api:powervr-sgl'),
    (r'wing32\.dll|wing\.dll', 'api:wing'),
    (r'dsound\.dll', 'api:directsound'), (r'xaudio2_\d+\.dll', 'api:xaudio2'), (r'x3daudio\d?_\d+\.dll', 'api:x3daudio'),
    (r'dinput8?\.dll', 'api:directinput'), (r'xinput\d?_?\d*\.dll', 'api:xinput'),
    (r'dplayx?\.dll', 'api:directplay'), (r'quartz\.dll', 'api:directshow'), (r'mfplat\.dll|mfreadwrite\.dll', 'api:media-foundation'),
    (r'wmvcore\.dll', 'api:windows-media'), (r'msacm32\.dll', 'api:acm'),
    (r'mss(32|16)\.dll', 'lib:miles'), (r'binkw(32|64)\.dll', 'lib:bink'), (r'smackw32\.dll', 'lib:smacker'),
    (r'fmod(ex)?(64)?\.dll|fmod_event.*\.dll', 'lib:fmod'), (r'bass\.dll', 'lib:bass'), (r'physxloader\.dll|physx.*\.dll', 'lib:physx'),
    (r'msvcr(\d+)\.dll|msvcp(\d+)\.dll|vcruntime\d+\.dll|ucrtbase\.dll|api-ms-win-crt-.*', 'lib:vc-runtime'),
    (r'msvbvm\d+\.dll', 'lib:vb-runtime'), (r'mscoree\.dll', 'lib:dotnet'),
    (r'steam_api(64)?\.dll', 'drm:steam'), (r'galaxy(64)?\.dll', 'store:gog-galaxy'),
    (r'eossdk-win(32|64)-shipping\.dll', 'store:epic'), (r'uplay_r1(_loader)?(64)?\.dll|upc_r\d.*\.dll', 'drm:ubisoft-connect'),
    (r'drm_dyndata.*|securom.*\.dll', 'drm:securom'), (r'clcd(16|32)\.dll', 'drm:safedisc'),
]

# PE section name -> need.
SECTIONS = {'.bind': 'drm:steamstub', '.securom': 'drm:securom', '.cms_t': 'drm:securom', '.cms_d': 'drm:securom',
            'stxt774': 'drm:safedisc', 'stxt371': 'drm:safedisc', '.vmp0': 'drm:vmprotect', '.vmp1': 'drm:vmprotect',
            '.themida': 'drm:themida', '.winlice': 'drm:themida', '.arxan': 'drm:arxan', 'UPX0': 'packer:upx'}

# A file anywhere in the game's folder or disc (lower case, a regular expression) -> need.
FILES = [
    (r'(^|/)secdrv\.sys$|(^|/)clcd(16|32)\.dll$|(^|/)dplayerx\.dll$|(^|/)00000001\.tmp$', 'drm:safedisc'),
    (r'(^|/)cms(16|_32)\.dll$|(^|/)paul\.dll$', 'drm:securom'),
    (r'(^|/)protect\.(dll|exe)$', 'drm:starforce'),
    (r'(^|/)steam_api(64)?\.dll$|(^|/)steam_appid\.txt$', 'drm:steam'),
    (r'(^|/)galaxy(64)?\.dll$|(^|/)goggame-\d+\.info$', 'store:gog-galaxy'),
    (r'(^|/)autorun\.inf$', 'media:cd'),
    (r'\.(cue|ccd|mds|nrg)$', 'media:cd-image'),
    (r'\.(wma|xwm|xwma)$', 'media:wma'), (r'\.bik$', 'lib:bink'), (r'\.smk$', 'lib:smacker'),
]

# Bytes in an executable -> need. Used for DOS programs and as a backup for Windows ones.
STRINGS = [
    (b'DOS/4G', 'extender:dos4gw'), (b'PMODE/W', 'extender:pmodew'), (b'Phar Lap', 'extender:pharlap'),
    (b'CWSDPMI', 'extender:cwsdpmi'), (b'go32', 'extender:djgpp'),
    (b'BLASTER', 'api:sound-blaster'), (b'ULTRASND', 'api:gravis-ultrasound'),
    (b'Miles Design', 'lib:miles'), (b'HMIDRV', 'lib:hmi-sos'), (b'sosDIG', 'lib:hmi-sos'),
    (b'GetVolumeInformationA', 'media:cd-check'),
]

# A CPU feature from an instruction's mnemonic or operands.
MMX = re.compile(r'\bmm[0-7]\b')
SSE = re.compile(r'\bxmm\d+\b')
AVX = re.compile(r'\b[yz]mm\d+\b')
NOW3D = re.compile(r'^(pf\w+|pi2f\w|femms|pavgusb|pmulhrw|pswapd)$')

# NE target operating system byte (offset 0x36 of the NE header).
NE_OS = {1: 'os:os2', 2: 'os:windows-3x', 3: 'os:dos4-european', 4: 'os:windows-386'}


def pe_os(major, minor):
    """The Windows a PE's subsystem version asks for at least."""
    if major < 4:
        return 'os:windows-nt3-or-win32s'
    if major == 4:
        return 'os:windows-9x'
    if major == 5:
        return 'os:windows-xp'
    return 'os:windows-vista-or-later'
