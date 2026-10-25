# The sound card makers' APIs: Maxi Sound 64, A3D, EAX, SoundFonts, QSound

From 1996 to 1999 most Windows games played their sound through DirectSound or WinMM, which
any card could serve. Some went further and called a card maker's own library for what
DirectSound did not have: 3D positional sound, reverb, instrument banks in the card's memory, or
more outputs. These libraries came with the card's drivers, and none of them ships with
Windows or Wine today. A game that imports one at load time will not start without it. A game
that looks for it at run time starts, but plays less than it was made for.

This page lists those libraries, how a game finds each one, what Wine offers, and whether a
stand-in can replace it. Only one row is *measured* here: the Maxi Sound 64, from POD Gold.
The other rows come from the libraries' own documentation and SDKs as remembered, and are not
yet checked against a game. Check a row on your game before you rely on it. For the broader
device-by-era inventory and GameFrame's next reusable chip and card models, see
[PC audio coverage](audio-coverage.md).
[Sound](sound.md) has the devices and DirectSound. [Replacement libraries](replacement-libraries.md)
has the method for writing a stand-in. `wrappers/` holds the stand-ins written so far.

## The libraries

| Card or technology | Library | API | How a game finds it |
|---|---|---|---|
| Guillemot Maxi Sound 64, Home Studio 64 (Dream SAM9407) | `94DLL32.dll` (a thunk to `94DLL16.DLL`) | 37 exports: MPU-401 commands, the card's memory, wave stream volumes, sample banks, the chip's voices | Imported at load time, by ordinal (POD Gold's `PODMAX64.EXE`, `PODFXM64.EXE`, *observed*) |
| Aureal A3D 1.x and 2.0 (Vortex chips; Diamond Monster Sound, Turtle Beach Montego) | `a3d.dll` | COM: `IA3d`, `IA3dListener`, `IA3dSource` and later interfaces; A3D 2.0 adds geometry for reflections and occlusion | `CoCreateInstance` of A3D's class ID, registered under `HKEY_CLASSES_ROOT\CLSID` by the driver. Some games call A3D's DirectSound 3D path instead |
| Creative EAX 1.0 and 2.0 (Sound Blaster Live!, AWE64 with later drivers) | none of its own | Property sets on DirectSound 3D buffers and the listener (`IKsPropertySet`) | `QueryInterface` for `IKsPropertySet` on a 3D buffer, then `QuerySupport` for the EAX property set's GUID |
| Creative AWE32 and AWE64 SoundFonts (EMU8000) | `sfman32.dll` | The SoundFont Manager: load and free instrument banks in the card's memory | `LoadLibrary` and `GetProcAddress` at run time. The game then plays MIDI through `midiOut` to the card's synthesiser |
| Sensaura (on ESS, Yamaha, Crystal and other chips) | none of its own | DirectSound 3D, with the chip maker's own property sets for extras | `QuerySupport` on the property set, as for EAX |
| QSound (QSound Labs) | `qmixer.dll` (QMixer) | A mixing library with QSound's 3D placement, on top of WinMM or DirectSound | Imported at load time, or installed with the game. Some chips also offered QSound through DirectSound 3D |
| Miles Sound System 3D providers (RAD Game Tools) | `mss32.dll` with `.m3d` provider files | Middleware: the game calls Miles, and Miles picks a provider (DirectSound 3D, A3D, EAX) | Miles lists the `.m3d` files beside the game. The game or the player picks one |

What each needs is different. A library a game imports at load time must exist, or the game
does not start. A COM class or a property set the game asks for at run time can be missing.
The game then falls back to plain DirectSound or to no 3D sound, if it was written to.

## What Wine offers

| Need | Wine |
|---|---|
| WinMM wave output (`waveOut`) | Yes, through the host's audio |
| WinMM MIDI output (`midiOut`) | Yes. On macOS it plays through Core Audio's own synthesiser. On Linux it goes to an ALSA sequencer port, so a software synthesiser (FluidSynth, TiMidity++) must be running |
| The WinMM mixer | A destination line for the speakers. No CD source line. Its wave source line's volume read 0 to POD (*measured*, Wine 11 on macOS) |
| DirectSound and DirectSound 3D | Yes, mixed in software |
| EAX property sets | Not in Wine itself. Wine Staging has had an EAX patch set (not checked here). DSOAL, a DirectSound on top of OpenAL Soft, offers EAX through OpenAL Soft's EAX support |
| A3D (`a3d.dll`) | No |
| SoundFont Manager (`sfman32.dll`) | No |
| `94DLL32.dll`, `qmixer.dll` | No. A game that ships `qmixer.dll` beside it brings its own |

## Which can be stood in for

| Library | Stand-in | Effect lost or kept |
|---|---|---|
| `94DLL32.dll` | **Done** (`wrappers/maxi94`). The card's wave streams become Wine's wave outputs, and the game's mixer lines are answered for | POD's sound effects play through a whole race (*measured*, emulated PC, recorded). Front and rear outputs are mixed into two speakers |
| `sfman32.dll` | Possible. Accept the banks the game loads, and give them to a software synthesiser that the game's MIDI goes to (FluidSynth reads SoundFont 2 files; the AWE's own `.sbk` banks are older) | The game's own instruments heard, as on the AWE |
| `a3d.dll` | Possible as a COM server. Map sources and the listener onto OpenAL Soft (HRTF), or onto DirectSound 3D. A3D 2.0's reflections need its geometry traced into an effect | 3D placement kept. A3D 2.0's wavetracing must be emulated, or it is lost |
| EAX | Needs no stand-in of its own. Use DSOAL with OpenAL Soft in place of Wine's DirectSound | Reverb kept, through OpenAL Soft's EAX |
| Sensaura | Nothing to replace: plain DirectSound 3D. Its extra property sets answer "not supported" | The chip's own extras lost |
| `qmixer.dll` | The game's own copy usually runs, since it mixes in software on WinMM or DirectSound | Kept |
| Miles providers | Use the DirectSound 3D provider, or EAX through DSOAL | As for EAX |

A silent stand-in is the first step for any of them. It answers every call as a present card
that does nothing, and the game starts. The second step reads what the game sends the library
and decides where it goes. The Maxi Sound 64 shows why that second step needs the game's code
first. The card's library suggests MIDI and sample banks, but POD sends neither: it mixes its
own sound and asks the card only which stream each wave output got. A stand-in that answers
"no streams" made the game turn its sound off.

## Downloads this would need

None of these libraries was downloaded for this page. A stand-in for A3D, EAX or the SoundFont
Manager is best checked against the original library on a real card or in a 1990s Windows in an
emulator, together with the SDK's headers for the exact interfaces. Ask the project's owner
before fetching any of them, and name the file, its source and its size.

## Checklist

1. List the game's imports (`./gameframe run peinfo GAME.EXE --imports`) and its strings for
   DLL names, COM class IDs and property-set GUIDs.
2. Find the code that calls the library, and write down what it sends: commands, banks,
   streams, positions, volumes. Name each function.
3. Find what the game does when the library is missing or says no. That is often the cause of
   silence under Wine.
4. Write the silent stand-in first, then route each kind of data to the modern equivalent.
5. Log the first buffer that is not silence, or the first note, to show the sound reaches the
   output.
