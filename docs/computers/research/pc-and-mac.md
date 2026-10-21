# Notes: the PC and the Macintosh

Covers IBM PC compatibles and the Apple Macintosh as game platforms, from 1981 to today: eras and executable formats, compilers, protection, Mac formats and emulators, Linux and modern engines. A finding marked "(not confirmed)" is widely documented, and the link is the place to confirm it. Treat its wording and dates as needing one more confirmation.

## PC eras as game platforms, and the executable formats of each era

### Takeaway
Each PC era has its own executable format, so its own loader and tool chain: COM and MZ for DOS, NE for Win16, LE/LX for DOS extenders and VxDs, PE32 for Win32 and PE32+ for x64. The main break for running old games for study is that 64-bit Windows cannot run 16-bit code at all. Windows 11 ships only in 64-bit form, so NE and DOS programs now need an emulator (DOSBox, Wine with winevdm/otvdm, a VM).

### Cited Findings

**IBM PC, XT, AT (1981–1984)**
- IBM PC model 5150 announced 12 August 1981, Intel 8088 at 4.77 MHz, with MDA or CGA video and a PC speaker driven by the 8253 PIT (not confirmed) — [IBM Personal Computer](https://en.wikipedia.org/wiki/IBM_Personal_Computer)
- IBM PC XT (5160) 1983 added a hard disk. IBM PC AT (5170) August 1984 used the 80286 (not confirmed) — [IBM PC XT](https://en.wikipedia.org/wiki/IBM_Personal_Computer_XT); [IBM PC AT](https://en.wikipedia.org/wiki/IBM_Personal_Computer_AT)

**PCjr and Tandy 1000 (1984–)**
- PCjr released March 1984. The Tandy 1000 series began in 1984 — [Tandy Graphics Adapter](https://en.wikipedia.org/wiki/Tandy_Graphics_Adapter)
- PCjr/Tandy graphics keep CGA modes (320×200 in 4 colours, 640×200 in 2, 40×25 and 80×25 text). They add 160×200×16, 320×200×16 and 640×200 in 4 of 16 colours. Tandy Video II / ETGA adds 640×200×16. Colours come from a 16-colour RGBI palette — [Tandy Graphics Adapter](https://en.wikipedia.org/wiki/Tandy_Graphics_Adapter)
- Some games detect Tandy hardware and draw Tandy graphics even when CGA is chosen. Others offer an explicit "Tandy" option — [Tandy Graphics Adapter](https://en.wikipedia.org/wiki/Tandy_Graphics_Adapter)
- The sound chip is the Texas Instruments SN76496 (SN76489 family): three square-wave tone voices plus one noise channel, at I/O port C0h (not confirmed; the Tandy Graphics Adapter page does not name the chip) — [Texas Instruments SN76489](https://en.wikipedia.org/wiki/Texas_Instruments_SN76489); [IBM PCjr](https://en.wikipedia.org/wiki/IBM_PCjr)
- Sierra's King's Quest (1984) was commissioned by IBM for the PCjr (not confirmed) — [King's Quest I](https://en.wikipedia.org/wiki/King%27s_Quest_I)
- PCjr was discontinued in 1985 (not confirmed) — [IBM PCjr](https://en.wikipedia.org/wiki/IBM_PCjr)

**DOS era formats (1981–late 1990s)**
- COM: a raw memory image with no header. It is loaded at offset 100h after the 256-byte PSP and must fit in one 64 KB segment. MZ (EXE): a header with "MZ" magic and a relocation table, for multi-segment programs (not confirmed) — [COM file](https://en.wikipedia.org/wiki/COM_file); [DOS MZ executable](https://en.wikipedia.org/wiki/DOS_MZ_executable)
- LE (Linear Executable) was used by Windows 3.x/9x VxDs and by DOS extenders such as DOS/4GW. LX is its OS/2 2.0 variant (not confirmed) — [Linear Executable](https://en.wikipedia.org/wiki/Linear_Executable)
- DJGPP programs are COFF images behind a small real-mode stub (go32 / CWSDPMI) (not confirmed) — [DJGPP](https://www.delorie.com/djgpp/)

**Windows 3.x / Win16 (1990–1995)**
- Windows 3.0 May 1990. Windows 3.1 April 1992. Win16 programs use the NE (New Executable) format, also used by OS/2 1.x (not confirmed) — [Windows 3.0](https://en.wikipedia.org/wiki/Windows_3.0); [New Executable](https://en.wikipedia.org/wiki/New_Executable)
- WinG shipped 21 September 1994 after being announced at the 1994 Game Developers Conference. It let Windows 3.x programs write device-independent bitmap memory directly (double buffering, dirty rectangles) and was meant to help port DOS games — [WinG](https://en.wikipedia.org/wiki/WinG)
- WinG was dropped as a separate API in Windows 98 SE in favour of DirectX. Windows NT 3.5 and Windows 95 added CreateDIBSection, which covers the same use — [WinG](https://en.wikipedia.org/wiki/WinG)
- Games listed as using WinG include Doom (the Win95 Doom95 build), SimCity 2000 (16-bit Windows), SimTower, Civilization II, Colonization, Heroes of Might and Magic II, Comix Zone and Titanic: Adventure Out of Time — [WinG](https://en.wikipedia.org/wiki/WinG)

**Windows 95 to ME / Win32 and DirectX (1995–2001)**
- Windows 95 released 24 August 1995. Win32 programs use PE32 (Portable Executable), introduced with Windows NT 3.1 in 1993 (not confirmed) — [Windows 95](https://en.wikipedia.org/wiki/Windows_95); [Portable Executable](https://en.wikipedia.org/wiki/Portable_Executable)
- DirectX 1.0 (the "Windows Games SDK") September 1995. Direct3D arrived with DirectX 2.0 (1996). DirectX 7 (1999) added hardware T&L, DirectX 8 (2000) programmable shaders, DirectX 9 (2002), DirectX 10 with Vista, DirectX 11 (2009) with Windows 7, DirectX 12 (2015) with Windows 10 (not confirmed) — [DirectX](https://en.wikipedia.org/wiki/DirectX)

**Windows XP to Windows 11 and 64-bit (2001–)**
- Windows XP 25 October 2001. AMD64 first shipped with the Opteron (April 2003) and Athlon 64 (September 2003). Windows XP Professional x64 Edition April 2005 (not confirmed) — [Windows XP](https://en.wikipedia.org/wiki/Windows_XP); [x86-64](https://en.wikipedia.org/wiki/X86-64); [Windows XP Professional x64 Edition](https://en.wikipedia.org/wiki/Windows_XP_Professional_x64_Edition)
- 64-bit programs use PE32+ (optional header magic 0x20B, against 0x10B for PE32) (not confirmed) — [PE Format, Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format)
- WOW64 runs 32-bit x86 programs on x64 Windows, and 32-bit x86 and ARM programs on ARM64 Windows. A 32-bit process cannot load a 64-bit DLL for execution, and the reverse also fails. DLLs loaded as data or resource files are exempt — [Running 32-bit Applications, Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/winprog64/running-32-bit-applications)
- "64-bit Windows does not support running 16-bit Windows-based applications", because handles have 32 significant bits. Attempts fail with ERROR_BAD_EXE_FORMAT — [Running 32-bit Applications, Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/winprog64/running-32-bit-applications)
- 32-bit Windows up to Windows 10 kept NTVDM for DOS and Win16 programs. Windows 11 (5 October 2021) has no 32-bit edition, so no 16-bit support remains in any current Windows (not confirmed) — [Virtual DOS machine](https://en.wikipedia.org/wiki/Virtual_DOS_machine); [Windows 11](https://en.wikipedia.org/wiki/Windows_11)

**Steam and GOG era (2003–)**
- Steam launched September 2003. Half-Life 2 (November 2004) required it (not confirmed) — [Steam (service)](https://en.wikipedia.org/wiki/Steam_(service))
- GOG.com launched in 2008 as Good Old Games and sells DRM-free builds, often older games repackaged with DOSBox or ScummVM (not confirmed) — [GOG.com](https://en.wikipedia.org/wiki/GOG.com)

**Tools that read each format**
- Ghidra ships a PEF loader (`PefLoader`) for classic Mac OS and BeOS PowerPC binaries — [Ghidra API: PefLoader](https://ghidradocs.com/11.4_PUBLIC/docs/GhidraAPI_javadoc/api/ghidra/app/util/opinion/PefLoader.html)
- Ghidra's built-in loaders include MZ, NE, PE and Mach-O. LE/LX needs a community extension (not confirmed; check the Ghidra loader list of the release in use) — [Ghidra file formats overview](https://www.mintlify.com/NationalSecurityAgency/ghidra/processors/file-formats)

### Inferences
- The executable format tells you the era and the loader before anything else. Checking the magic (MZ, then the e_lfanew header: NE, LE, LX or PE) is the first triage step. GameFrame's `tools/triage` and `tools/dos` already cover MZ and NE.
- A Tandy/PCjr code path is a separate rendering and sound path in the same binary. A port that aims for exact behaviour must choose which path is ground truth, and the emulator must report itself as Tandy (DOSBox `machine=tandy` or `pcjr`) to run it.
- Win16 games (WinG era) cannot be run under any 64-bit Windows. Wine (with its own Win16 support) or a 32-bit Windows VM is the practical way to observe them.

### Gaps
- Exact DirectX version dates are not confirmed.
- The Ghidra LE/LX loader status is not confirmed.

## Compilers and runtimes per era and how they show up in a binary

### Takeaway
The compiler leaves runtime-library code, copyright strings, startup code and calling conventions in the binary, and these identify the toolchain. Identifying it early tells you which library functions to label and skip, and which integer and floating-point semantics a port must copy.

### Cited Findings
- Turbo Pascal 1.0 November 1983. Turbo Pascal 7.0 / Borland Pascal 7.0 1992. TP programs on fast CPUs (around 200 MHz and up) fail with "Runtime error 200" (division by zero) in the CRT unit's delay-loop calibration (not confirmed) — [Turbo Pascal](https://en.wikipedia.org/wiki/Turbo_Pascal); [Runtime error 200](https://en.wikipedia.org/wiki/Turbo_Pascal#CRT_Delay_problem)
- Borland C++ binaries carry the string "Borland C++ - Copyright 1991 Borland Intl." and Borland's startup code (c0.asm). Borland debug info (TDInfo) may be appended to the MZ/NE file (not confirmed) — [Borland C++](https://en.wikipedia.org/wiki/Borland_C%2B%2B)
- Watcom C/C++ was bundled with the DOS/4GW extender. DOS/4GW prints a banner at start-up. Watcom's default 32-bit convention passes arguments in EAX, EDX, EBX, ECX ("watcall"). Doom (1993) was built with Watcom (not confirmed) — [Watcom C/C++](https://en.wikipedia.org/wiki/Watcom_C/C%2B%2B); [DOS/4G](https://en.wikipedia.org/wiki/DOS/4G)
- DJGPP (DJ Delorie, from 1989) is a GCC port for DOS. Quake (1996) was built with DJGPP (not confirmed) — [DJGPP](https://en.wikipedia.org/wiki/DJGPP)
- Microsoft Visual C++ 1.0 1993, 6.0 1998, Visual Studio .NET (7.0) 2002. MSVC-linked PE files carry an undocumented "Rich" header between the DOS stub and the PE header, listing tool and build IDs. Dynamically linked CRTs show as imports such as MSVCRT.DLL, MSVCR71.DLL or VCRUNTIME140.DLL (not confirmed) — [Microsoft Visual C++](https://en.wikipedia.org/wiki/Microsoft_Visual_C%2B%2B); [Portable Executable](https://en.wikipedia.org/wiki/Portable_Executable)
- Delphi 1.0 1995 (Win16), Delphi 2 1996 (Win32). Delphi binaries carry VCL class names, RTTI and DFM form resources in RCDATA (not confirmed) — [Delphi (software)](https://en.wikipedia.org/wiki/Delphi_(software))
- .NET Framework 1.0 2002. Mono 1.0 2004. .NET assemblies are PE files with a CLI header and IL metadata, which decompilers such as ILSpy and dnSpy turn back into C# (not confirmed) — [.NET Framework](https://en.wikipedia.org/wiki/.NET_Framework); [Mono](https://en.wikipedia.org/wiki/Mono_(software))
- Unity's IL2CPP is an ahead-of-time scripting back end. It "converts all managed assemblies into standard C++ code", which is compiled to native code and packaged as "either an executable file or a DLL". It was first made for platforms without JIT, such as iOS — [IL2CPP overview, Unity Manual](https://docs.unity3d.com/Manual/il2cpp-introduction.html)
- On Windows, IL2CPP output is GameAssembly.dll plus a global-metadata.dat file that keeps type and method names. Mono builds keep Assembly-CSharp.dll in a `GameName_Data/Managed` folder. Il2CppDumper rebuilds names from the metadata (not confirmed; the Unity manual page does not name these files) — [Il2CppDumper](https://github.com/Perfare/Il2CppDumper)

### Inferences
- Pascal, Borland C and Watcom each have their own calling conventions and real-number formats (Turbo Pascal's 6-byte Real, for example). A port that must match bit for bit has to reproduce the language's arithmetic rules, not the C rules.
- A Unity Mono game is decompiled to near-source C#. An IL2CPP game is native code whose names can be recovered. The two need different workflows, and the folder layout shows which one you have.

### Gaps
- The first linker version to write the Rich header is not confirmed.
- The Borland and Watcom convention details lack primary documentation (not confirmed).

## Copy protection and DRM across the PC eras

### Takeaway
Protection moved from things the user had (a manual, a code wheel, a key disk) to things the disc had (SafeDisc, SecuROM, StarForce), then to online checks (Steam, Denuvo). The later disc systems used kernel drivers that current Windows blocks. For study, that means many protected originals no longer run on a modern OS even when owned. A clean environment (a period OS in a VM, or the publisher's later unprotected re-release) is often the practical path, and the decision about which route to take stays with the user.

### Cited Findings
- Manual and feeler checks: games asked for a word from the manual or used code wheels, such as Monkey Island's "Dial-a-Pirate" (1990) (not confirmed) — [Copy protection](https://en.wikipedia.org/wiki/Copy_protection)
- Floppy-based protection used deliberately malformed tracks, sectors or weak bits that a normal copy could not reproduce (not confirmed) — [Copy protection](https://en.wikipedia.org/wiki/Copy_protection)
- SafeDisc first appeared in 1998, developed by Macrovision (1999–2007) and then Trymedia (2008–2009). Version 4 came out February 2005, the final release 4.90.010 on 2 May 2008, and it was discontinued in March 2009 — [SafeDisc](https://en.wikipedia.org/wiki/SafeDisc)
- SafeDisc's driver is secdrv.sys. Microsoft disclosed an elevation-of-privilege flaw in it on 7 November 2007 and patched it on 11 December 2007. Shortly after Windows 10's release in 2015, Microsoft said SafeDisc games would not run on it. Update 3086255 (2015) withdrew support on earlier versions — [SafeDisc](https://en.wikipedia.org/wiki/SafeDisc)
- SecuROM over time overtook SafeDisc. Windows 10 would not run SecuROM-protected games either — [SafeDisc](https://en.wikipedia.org/wiki/SafeDisc)
- StarForce (StarForce Technologies, Moscow, formerly Protection Technology). StarForce 3.0 installed a device driver that was generally not removed on uninstall. StarForce 4.0 added a service that removes it. In 2006, Heroes of Might and Magic V and GTR2 replaced StarForce 3.0 with SecuROM after a Ubisoft forum poll. A driverless approach appeared around May 2014 — [StarForce](https://en.wikipedia.org/wiki/StarForce)
- The Steam DRM wrapper "verifies game ownership and ensures that Steamworks features work properly by launching Steam before launching the game". Valve says it "by itself is not an anti-piracy solution" and that its obfuscation "is easily removed by a motivated attacker" — [Steam DRM, Steamworks documentation](https://partner.steamgames.com/doc/features/drm)
- Steam Custom Executable Generation (CEG) built a per-customer executable. The Steamworks DRM page does not mention it (not confirmed) — [Steam (service)](https://en.wikipedia.org/wiki/Steam_(service))
- Denuvo Software Solutions GmbH (Salzburg) was formed 23 September 2014 by a management buyout of DigitalWorks, the developer of SecuROM. FIFA 15 (September 2014) was the first Denuvo Anti-Tamper game. Irdeto acquired it in January 2018 — [Denuvo](https://en.wikipedia.org/wiki/Denuvo)
- Denuvo needs online activation and binds a token to factors such as the user's hardware. Its code is integrated with the game's code. Publishers have removed it by patch, for example Rage 2's Steam version two days after release — [Denuvo](https://en.wikipedia.org/wiki/Denuvo)

### Inferences
- Code that is protected or virtualised (Denuvo, VMProtect, SecuROM's later versions) is hard to decompile in those regions. Emulating single functions with Unicorn works best on the game's own unprotected logic, not on the protection layer.
- Later DRM-free releases (GOG, patched Steam builds) are often the cleanest owned copy to study. The legal position of bypassing protection varies by country (see `docs/LEGAL.md`), and the guide leaves that choice to the user.

### Gaps
- Exact first-release years of SecuROM and StarForce are not confirmed.
- No primary Valve source on CEG is known.

## Apple Macintosh: hardware, OS eras and executable formats

### Takeaway
The Mac has had four CPU families: 68k (1984–1996), PowerPC (1994–2006), Intel x86/x86-64 (2006–2020/23) and ARM64 (2020–). Each change came with a translator: the 68k emulator in PowerPC Mac OS, Rosetta, then Rosetta 2. Each format also has its own wrapper: CODE resources in the resource fork, PEF/CFM, Mach-O, fat binaries. Catalina (2019) ended all 32-bit apps, and Rosetta 2 is being cut back from macOS 28 (2027). Most Mac games before 2019 therefore need an emulator or old hardware to run.

### Cited Findings

**68k Macs and Classic Mac OS**
- Macintosh 128K 24 January 1984, Motorola 68000 at about 8 MHz. Mac II March 1987 (68020, colour). Quadra 700/900 October 1991 (68040) (not confirmed) — [Macintosh 128K](https://en.wikipedia.org/wiki/Macintosh_128K); [Macintosh II](https://en.wikipedia.org/wiki/Macintosh_II); [Macintosh Quadra](https://en.wikipedia.org/wiki/Macintosh_Quadra)
- Mac OS 8.1 (January 1998) was the last 680x0-compatible release. Mac OS 8.5 was PowerPC-only — [Basilisk II](https://en.wikipedia.org/wiki/Basilisk_II)
- 68k application code is stored as CODE resources in the resource fork. CODE 0 holds the jump table. Globals and the jump table sit around register A5 (the "A5 world"), and the Segment Loader loads segments on demand. Toolbox calls are A-line traps (opcodes $Axxx) (not confirmed) — [Inside Macintosh: Mac OS Runtime Architectures (archive)](https://developer.apple.com/library/archive/documentation/mac/runtimehtml/RTArch-110.html)
- Ghidra does not understand resource forks. On macOS you can open a fork by adding `..namedfork/rsrc` to the path. Users import extracted CODE resources as raw binaries, set the header up by hand, and have added A-trap names to the 68000 spec without parameter knowledge. RetroGhidra has a basic Classic Mac loader (community forum source) — [A-Traps in Ghidra, 68kMLA](https://68kmla.org/bb/goto/post?id=466137); [68kmla.org thread](https://68kmla.org/bb/goto/post?id=510430) (search snippet)
- Forks are lost when files are copied to non-Mac file systems, so archives use MacBinary, BinHex, AppleSingle/AppleDouble or HFS disk images (DiskCopy 4.2) (not confirmed) — [Resource fork](https://en.wikipedia.org/wiki/Resource_fork)

**PowerPC Macs (1994–2006)**
- Power Macintosh 6100/7100/8100 launched 14 March 1994 with the PowerPC 601. A 68k emulator built into the ROM ran older code (not confirmed) — [Power Macintosh](https://en.wikipedia.org/wiki/Power_Macintosh)
- PEF (Preferred Executable Format) is Apple's executable format for Classic Mac OS, "optimized for RISC processors". It is loaded by the Code Fragment Manager (CFM files). It stayed supported on PowerPC Mac OS X for Carbon apps, so one binary could run on both OSes. BeOS for PowerPC also used it — [Preferred Executable Format](https://en.wikipedia.org/wiki/Preferred_Executable_Format)
- "Fat" Classic apps held both 68k CODE resources and a PowerPC PEF fragment in the data fork (not confirmed) — [Fat binary](https://en.wikipedia.org/wiki/Fat_binary)
- Mac OS X 10.0 March 2001. The Classic environment ran Mac OS 9 apps on PowerPC Macs up to 10.4 Tiger and never on Intel (not confirmed) — [Classic environment](https://en.wikipedia.org/wiki/Classic_Environment)

**Intel Macs (2006–2020) and Rosetta**
- Intel transition announced at WWDC June 2005. First Intel Macs January 2006 (not confirmed) — [Mac transition to Intel processors](https://en.wikipedia.org/wiki/Mac_transition_to_Intel_processors)
- Rosetta (PowerPC to Intel) arrived in 2006 with Mac OS X 10.4.4. It was optional on Snow Leopard and removed in Lion (10.7) in 2011. It ran only user-level code and did not translate G5 instructions — [Rosetta (software)](https://en.wikipedia.org/wiki/Rosetta_(software))
- Mach-O magic is 0xfeedface (32-bit) and 0xfeedfacf (64-bit). Multi-architecture ("universal"/fat) files use 0xcafebabe big-endian, with a header listing CPU type, subtype, offset, size and alignment of each image. A file is a header, load commands, then segments of up to 255 sections each — [Mach-O](https://en.wikipedia.org/wiki/Mach-O)
- macOS Catalina (7 October 2019) was the first macOS to run only 64-bit apps. 32-bit apps, Carbon software and QuickTime 7 apps no longer run — [macOS Catalina](https://en.wikipedia.org/wiki/MacOS_Catalina)

**Apple silicon (2020–) and Rosetta 2**
- Rosetta 2 (Intel to ARM64) was announced in 2020 and shipped with macOS Big Sur (11) — [Rosetta (software)](https://en.wikipedia.org/wiki/Rosetta_(software))
- At WWDC 2025 Apple said most Rosetta 2 features will be removed in macOS 28 (2027). Support will remain only for older, unmaintained games. The Wikipedia infobox and body disagree on 27 against 28 — [Rosetta (software)](https://en.wikipedia.org/wiki/Rosetta_(software))
- Rosetta 2 does not translate kernel extensions, x86_64 virtual machines, or AVX/AVX2/AVX-512 instructions (not confirmed) — [About the Rosetta translation environment, Apple](https://developer.apple.com/documentation/apple-silicon/about-the-rosetta-translation-environment)

**Mac game history highlights**
- Marathon (Bungie, December 1994, Mac) (not confirmed). Marathon 2: Durandal: Mac 24 November 1995, Windows 20 September 1996 — [Marathon 2: Durandal](https://en.wikipedia.org/wiki/Marathon_2:_Durandal); [Marathon (video game)](https://en.wikipedia.org/wiki/Marathon_(video_game))
- Myst (Cyan, 1993) was first released on the Mac and built with HyperCard. Windows followed in 1994 (not confirmed) — [Myst](https://en.wikipedia.org/wiki/Myst)

### Inferences
- A 68k Mac binary needs three steps before analysis: extract the resource fork, split the CODE resources, and rebuild the A5-relative jump table so calls between segments resolve. No mainstream loader does all three, so expect to build or reuse a small extractor.
- Big-endian CPUs (68k, PowerPC) change data-file decoding. Ports of Mac games to PC (and the reverse) often kept big-endian data files, which helps when comparing versions.
- A fat or universal file can carry two compiled versions of the same game. Comparing them (68k against PPC, or x86_64 against arm64) gives a second view of each function.

### Gaps
- Exact Apple dates for the final 68k Mac and the end of the Apple silicon transition (Mac Pro 2023) are not confirmed.
- The Rosetta 2 limits are not confirmed against Apple's page.

## Emulators and tools for Mac study, and Mac source releases or reimplementations

### Takeaway
Open emulators cover every Classic Mac era: Mini vMac and MAME for early 68000 Macs, Basilisk II for 68020–68040 up to Mac OS 8.1, SheepShaver for PowerPC Mac OS 7.5.2–9.0.4, and QEMU for PowerPC Mac OS 9 and early OS X. All need a ROM image or install media from the user's own machine. Ghidra handles 68000, PowerPC, PEF and Mach-O, but not resource forks. Marathon's source release (Aleph One) is the main example of an official Mac game source release.

### Cited Findings
- Basilisk II: by Christian Bauer, first released March 1999, GPL, source on GitHub. It emulates 68000-series Macs and runs up to Mac OS 8.1 — [Basilisk II](https://en.wikipedia.org/wiki/Basilisk_II)
- SheepShaver: by Christian Bauer and Gwenolé Beauchesne, a GPL PowerPC Mac emulator. It was commercial from 1998 and open-sourced in 2002. The final release is 2.5 (13 September 2022). It runs Mac OS 7.5.2 to 9.0.4. Mac OS 8.1 and earlier need an Old World ROM. It has no MMU emulation, so Mac OS 9.1–9.2.2 do not run — [SheepShaver](https://en.wikipedia.org/wiki/SheepShaver)
- Mini vMac (Paul C. Pratt) emulates the Mac Plus and other early 68000 Macs, with a Mac II variant (not confirmed) — [Mini vMac](https://www.gryphel.com/c/minivmac/)
- QEMU's PowerPC machines (g3beige, mac99) run Mac OS 9 and Mac OS X up to 10.5 (not confirmed) — [QEMU PowerPC Mac99 / g3beige documentation](https://www.qemu.org/docs/master/system/ppc/powermac.html)
- MAME has Macintosh drivers (mac128k, macplus, macse, macii and others) (not confirmed) — [MAME](https://www.mamedev.org/)
- Executor (ARDI) reimplemented the 68k Mac OS so Mac programs ran without Apple ROMs. It was later open-sourced. Advanced Mac Substitute is a newer project with the same goal (not confirmed) — [Executor](https://en.wikipedia.org/wiki/Executor_(software)); [Advanced Mac Substitute](https://www.v68k.org/advanced-mac-substitute/)
- resource_dasm (Martin Michelsen) decodes many classic Mac resource types and disassembles 68k and PPC code resources (not confirmed) — [resource_dasm](https://github.com/fuzziqersoftware/resource_dasm)
- Ghidra has a PEF loader. Users import 68k CODE resources raw and add A-trap names by hand — [Ghidra API: PefLoader](https://ghidradocs.com/12.1.3_PUBLIC/docs/GhidraAPI_javadoc/api/ghidra/app/util/opinion/PefLoader.html); [68kMLA thread](https://68kmla.org/bb/goto/post?id=466137)
- Unicorn supports M68K and PowerPC CPUs. PPC was added in Unicorn 2.0 (not confirmed) — [Unicorn Engine](https://www.unicorn-engine.org/)
- Bungie released the Marathon 2 source code in January 2000, shortly before Microsoft acquired it. Aleph One, the engine of the Marathon Open Source Project, grew from it. The trilogy's data was released as freeware in 2005 — [Marathon 2: Durandal](https://en.wikipedia.org/wiki/Marathon_2:_Durandal)
- The source was released under the GNU GPL. Aleph One runs on Windows, macOS and Linux (not confirmed; the Marathon 2 page does not name the licence) — [Aleph One on GitHub](https://github.com/Aleph-One-Marathon/alephone)
- "Classic Marathon 2" was re-released on Steam on Aleph One. The infobox gives 12 July 2024 but the lead says 2025 — [Marathon 2: Durandal](https://en.wikipedia.org/wiki/Marathon_2:_Durandal)

### Inferences
- For ground truth on a 68k Mac game, GameFrame's Unicorn approach can run single CODE-segment functions if the A5 world and needed traps are stubbed, much as `tools/emu/dos_env` stubs DOS. This is a natural future extension.
- Aleph One is a reference for "official source plus fan engine" as a model, but its engine is not the original's binary. Exact-match work should still use the shipped binary.

### Gaps
- Mini vMac, QEMU and MAME details are not confirmed.
- No reliable source covers other official Mac-only source releases.

## Linux ports, and how modern engines change reverse engineering

### Takeaway
Linux was a port target (Loki Software, id Software's own ports) and is now mostly reached through Proton (Wine) on Steam Deck and SteamOS. Modern engine games move the game's logic into managed code (Unity Mono), generated native code with metadata (IL2CPP), engine reflection and packaged assets (Unreal .pak, Godot .pck). Anti-cheat and anti-tamper can block debuggers and emulation.

### Cited Findings
- Loki Software ported Windows games to Linux from 1998 until it closed in 2002 (not confirmed) — [Loki Entertainment Software](https://en.wikipedia.org/wiki/Loki_Entertainment_Software)
- Steam for Linux launched February 2013. Proton (Wine-based) was released August 2018. Steam Deck shipped February 2022 (not confirmed) — [Steam (service)](https://en.wikipedia.org/wiki/Steam_(service)); [Proton (software)](https://en.wikipedia.org/wiki/Proton_(software)); [Steam Deck](https://en.wikipedia.org/wiki/Steam_Deck)
- Unity IL2CPP converts all managed assemblies to C++ and compiles them ahead of time. It trades build size and time for performance — [IL2CPP overview, Unity Manual](https://docs.unity3d.com/Manual/il2cpp-introduction.html)
- Unreal Engine 4 (2014) and Unreal Engine 5 (April 2022) package assets in .pak files, which may be AES-encrypted. UObject reflection keeps class and property names in the binary (not confirmed) — [Unreal Engine](https://en.wikipedia.org/wiki/Unreal_Engine)
- Godot (MIT licence) packs games into .pck files and GDScript. Tools such as gdsdecomp recover projects (not confirmed) — [Godot](https://godotengine.org/); [gdsdecomp](https://github.com/bruvzg/gdsdecomp)
- Kernel-level anti-cheat (Easy Anti-Cheat, BattlEye, Riot Vanguard) runs alongside games and can block debuggers (not confirmed) — [Anti-cheat software](https://en.wikipedia.org/wiki/Anti-cheat_software)

### Inferences
- In engine-based games, the engine is generic and the game logic is a small part. Reverse engineering then targets scripts, IL or metadata, and the assets, not the whole binary.
- Linux builds (where they exist) sometimes keep more symbols than Windows builds, which makes them a useful second binary.

### Gaps
- The Unreal, Godot and anti-cheat facts lack primary sources.
