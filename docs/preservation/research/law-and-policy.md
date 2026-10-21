# Notes: law and policy

Not legal advice. Notes for a preservation-focused site. Research done 21 October 2026. Some primary pages (Federal Register, Justia, EUR-Lex, CURIA, the ECI register, the 2027 petitions page) blocked automated fetches; where that happened the finding cites the best reachable source and says so.

## United States: DMCA Section 1201, the 1201(f) interoperability exemption, and reverse-engineering case law

### Takeaway
The current 1201 exemptions (October 2024 to October 2027) let individuals restore server-dead games for personal, local play and let libraries, archives and museums preserve games in playable form, but only on their physical premises; the 2024 request for remote access was denied. A tenth triennial proceeding (the 2027 cycle) opened in June 2026. Separately, 17 U.S.C. 1201(f) and the Ninth Circuit's Sega and Connectix decisions are the legal footing for reverse engineering to build interoperable, independently written code.

### Cited Findings
**Current exemptions (37 C.F.R. 201.40, from the ninth triennial rulemaking, final rule published 28 October 2024)**
- The video game exemption is 37 C.F.R. 201.40(b)(19). It covers complete, lawfully acquired games where the copyright owner "has ceased to provide access to an external computer server." — [eCFR 201.40](https://www.ecfr.gov/current/title-37/chapter-II/subchapter-A/part-201/section-201.40)
- (b)(19)(i)(A): circumvention, copying and modification are allowed "to restore access to the game for personal, local gameplay" on a personal computer or console. — [eCFR 201.40](https://www.ecfr.gov/current/title-37/chapter-II/subchapter-A/part-201/section-201.40)
- (b)(19)(i)(B): an eligible library, archives or museum may circumvent to preserve the game "in a playable form", provided the game "is not distributed or made available outside of the physical premises" of the institution. — [eCFR 201.40](https://www.ecfr.gov/current/title-37/chapter-II/subchapter-A/part-201/section-201.40)
- (b)(19)(ii): extends the institutional preservation exemption to games that never needed a server and are "no longer reasonably available in the commercial marketplace", with the same premises limit. — [eCFR 201.40](https://www.ecfr.gov/current/title-37/chapter-II/subchapter-A/part-201/section-201.40)
- (b)(19)(iv)(C): "ceased to provide access" means server support has been discontinued for at least six months and not restored. — [eCFR 201.40](https://www.ecfr.gov/current/title-37/chapter-II/subchapter-A/part-201/section-201.40)
- (b)(20) covers preservation of computer programs other than video games. It limits use outside the premises. 2024 changes clarified that one lawfully owned, preserved copy may serve as many users at once as there are lawfully owned copies (no extra simultaneous copies). — [eCFR 201.40](https://www.ecfr.gov/current/title-37/chapter-II/subchapter-A/part-201/section-201.40); [Finnegan summary of the final rule](https://www.finnegan.com/en/insights/ip-updates/final-rule-issued-in-the-us-copyright-offices-ninth-triennial-section-1201-proceeding.html)
- The Software Preservation Network (SPN) and Library Copyright Alliance (LCA) petitioned to renew the abandoned-video-games exemption; the Register found a continuing need. — [SPN/LCA renewal petition](https://copyright.gov/1201/2024/petitions/renewal/Renewal%20Pet.%20-%20Abandoned%20Video%20Games%20-%20Software%20Preservation%20Network%20and%20Library%20Copyright%20Alliance.pdf); [Register's Recommendation, October 2024](https://www.copyright.gov/1201/2024/2024_Section_1201_Registers_Recommendation.pdf)
- The ninth proceeding began with a notice of inquiry on 8 June 2023. — [Copyright Office bulletin](https://content.govdelivery.com/accounts/USLOCCOPYRIGHT/bulletins/35be9b1)
- The Federal Register final rule is document 2024-24563, 28 October 2024 (page blocks automated fetch; not read directly). — [Federal Register](https://www.federalregister.gov/documents/2024/10/28/2024-24563/exemption-to-prohibition-on-circumvention-of-copyright-protection-systems-for-access-control)

**2024 denial of remote access (proposed Class 6(b))**
- SPN and LCA, supported by the Video Game History Foundation (VGHF), asked for libraries and archives to give researchers remote (off-premises) access to preserved games. The proposal was filed as Class 6(b). — [SPN/LCA Class 6(b) initial comments](https://www.copyright.gov/1201/2024/comments/Class%206(b)%20-%20Initial%20Comments%20-%20%20Software%20Preservation%20Network%20and%20Library%20Copyright%20Alliance.pdf); [SPN/LCA reply](https://www.copyright.gov/1201/2024/comments/reply/Class%206(b)%20-%20Reply%20-%20Software%20Preservation%20Network%20and%20Library%20Copyright%20Alliance.pdf)
- The Entertainment Software Association (ESA) opposed it. — [ESA opposition comments](https://www.copyright.gov/1201/2024/comments/opposition/Class%206(b)%20-%20Opp'n%20-%20Entertainment%20Software%20Association.pdf)
- The Register recommended against it and the Librarian adopted that; it was reported in late October 2024. Reported reasons: the proponents had not shown the off-site use would be fair use, there were not enough safeguards to limit use to research, and there was a "significant risk" that preserved games would be used for recreation. The denial is around page 191 of the Recommendation. — [VGC](https://videogameschronicle.com/news/us-copyright-office-refuses-to-allow-remote-access-to-out-of-print-games); [Register's Recommendation](https://www.copyright.gov/1201/2024/2024_Section_1201_Registers_Recommendation.pdf)
- VGHF said the ESA stated at the hearing that it would not support remote research access under any conditions. VGHF said it is "not done fighting". It cites its own study finding about 87% of games released in the US before 2010 are out of print. — [VGHF statement](https://gamehistory.org/dmca-2024-statement/)

**The 2027 cycle (tenth triennial)**
- The Copyright Office published the notice of inquiry and request for petitions on 9 June 2026. Renewal and new-exemption petitions were due 24 August 2026. Comments on renewal petitions were due 28 September 2026. Docket COLC-2026-0100 on regulations.gov. Exemptions from this round would run from October 2027 to October 2030. — [Copyright Office 1201 page for 2027](https://www.copyright.gov/1201/2027/)

**17 U.S.C. 1201(f)**
- (f)(1): a person who has lawfully obtained the right to use a program may circumvent an access control to identify and analyse the elements needed for interoperability of an independently created program, where those elements "have not previously been readily available" to them and the analysis does not infringe. — [17 U.S.C. 1201, Cornell LII](https://www.law.cornell.edu/uscode/text/17/1201)
- (f)(2): they may develop and use circumvention tools for that purpose. (f)(3): the information and tools may be shared "solely for the purpose of enabling interoperability of an independently created computer program with other programs", without infringement. (f)(4): "interoperability" is "the ability of computer programs to exchange information" and use it. — [17 U.S.C. 1201, Cornell LII](https://www.law.cornell.edu/uscode/text/17/1201)

**Case law**
- *Sega Enterprises v. Accolade*, 977 F.2d 1510 (9th Cir. 1992): Accolade bought Sega games and disassembled them to learn the Genesis interface. Disassembly was fair use where it was the only way to reach unprotected functional elements and there was a legitimate reason for that access. — [Copyright Office fair use summary](https://www.copyright.gov/fair-use/summaries/segaenters-accolade-9thcir1992.pdf)
- *Sony Computer Entertainment v. Connectix*, 203 F.3d 596 (9th Cir. 2000): intermediate copies made while reverse engineering the PlayStation BIOS to write the Virtual Game Station emulator were fair use. The emulator was found modestly transformative. — [Copyright Office fair use summary](https://www.copyright.gov/fair-use/summaries/sony-connectix-9thcir2000.pdf); [opinion text](https://law.resource.org/pub/us/case/reporter/F3/203/203.F3d.596.99-15852.html)
- Nintendo's case against the Yuzu Switch emulator (Tropic Haze) relied on DMCA anti-circumvention and anti-trafficking claims, and settled in March 2024 with a $2.4 million judgment and a permanent injunction. — [ChannelNews](https://www.channelnews.com.au/nintendo-settles-lawsuit-with-tropic-haze/)

### Inferences
- The exemptions cover circumvention, not distribution. Nothing in them lets anyone share game files with the public. The premises limit is the main friction for preservation institutions.
- For reimplementation, 1201(f) and Sega/Connectix protect the analysis step and the shipping of independent code. They do not protect copying protected expression (code, art, audio, text) into the result. Yuzu shows that the circumvention side (keys, decryption) is where emulators get hit, separate from the reverse-engineering question.

### Gaps
- I could not reach the 2027 petitions page (HTTP 403). Whether SPN/LCA filed to renew the game exemption, or refiled for remote access, in August 2026 is unconfirmed. The site should check regulations.gov docket COLC-2026-0100.
- Exact wording of the Register's 2024 reasoning was taken from press reports, not read directly from the PDF.
- Other US case law (e.g. *Atari v. Nintendo* 1992 on 10NES copying; *Sony v. Bleem* 2000; *Oracle v. Google* 2021 on APIs as fair use; *Lewis Galoob v. Nintendo* 1992) was not verified in this session.

## European Union: Software Directive, CJEU cases, and Stop Killing Games

### Takeaway
Directive 2009/24/EC lets a lawful user observe, study and test a program (Art. 5(3)), decompile it only as needed for interoperability of an independently created program (Art. 6), and, per Top System (2021), decompile to correct errors (Art. 5(1)); SAS v WPL (2012) says functionality, programming languages and data file formats are not protected expression. The Stop Killing Games European Citizens' Initiative gathered about 1.29 million verified signatures; the Commission answered on 16 June 2026 with no new law, offering a voluntary code of conduct instead.

### Cited Findings
**Software Directive 2009/24/EC** (statutory text; EUR-Lex returned an empty page to the fetcher, so the summary is from the case law and commentary below)
- Art. 5(3): a person with a right to use a copy may, without authorisation, "observe, study or test" how the program works to find the ideas and principles behind it, while loading, displaying, running, transmitting or storing it as they are entitled to. — [Directive 2009/24/EC, EUR-Lex](https://eur-lex.europa.eu/eli/dir/2009/24/oj); applied in [SAS v WPL summary, SCL](https://www.scl.org/2451-sas-institute-inc-v-world-programming-ltd-report-and-analysis/)
- Art. 6: decompilation (reproduction and translation of code) is allowed without authorisation where indispensable to get the information needed for interoperability of an independently created program, by a licensee, where the information is not already readily available, and confined to the parts needed. The information may not be used for other goals, given to others except where needed for interoperability, or used to make a program substantially similar in expression. Art. 8 makes contract terms contrary to Art. 6 and 5(2)-(3) null and void. — [Directive 2009/24/EC, EUR-Lex](https://eur-lex.europa.eu/eli/dir/2009/24/oj) (not fetched; matches the published text as I know it, verify before quoting)

**SAS Institute v World Programming (C-406/10, CJEU, 2 May 2012)**
- Neither the functionality of a program, nor the programming language, nor the format of data files is a form of expression protected by copyright under the Directive. — [SCL report](https://www.scl.org/2451-sas-institute-inc-v-world-programming-ltd-report-and-analysis/); [Wikipedia case summary](https://en.wikipedia.org/wiki/SAS_Institute_Inc_v_World_Programming_Ltd)
- A licensed user may observe, study and test the program to determine its ideas and principles without the owner's authorisation, provided they do not infringe the exclusive rights; a licence term cannot stop this. — [SCL](https://www.scl.org/2436-sas-v-wpl-no-copyright-says-ecj/)
- Manuals can be protected where they are their author's own intellectual creation; keywords, syntax and commands alone are not enough, though their selection and arrangement might be. — [SCL report](https://www.scl.org/2451-sas-institute-inc-v-world-programming-ltd-report-and-analysis/)

**Top System v Belgian State (C-13/20, CJEU, 6 October 2021)**
- A lawful acquirer may decompile all or part of a program to correct errors affecting its operation, under Art. 5(1). The Art. 6 conditions do not have to be met. — [Cuatrecasas analysis](https://www.cuatrecasas.com/en/spain/article/decompilation-of-software-for-error-correction-cjeu-judgment-in-top-system)
- The dispute involved Selor, the Belgian federal selection agency, decompiling Top System's framework. — [Cuatrecasas](https://www.cuatrecasas.com/en/spain/article/decompilation-of-software-for-error-correction-cjeu-judgment-in-top-system)

**Stop Killing Games ("Stop Destroying Videogames", ECI(2024)000007)**
- Submitted to the Commission on 26 January 2026 with 1,294,188 verified statements of support. Organisers met Commission officials on 23 February 2026. Parliament held a plenary debate on 21 May 2026. — [Lewis Silkin, 24 June 2026](https://www.lewissilkin.com/insights/2026/06/24/european-commission-stops-short-of-imposing-mandatory-requirements-following-citi-102n4yy); [Commission meeting page](https://commission.europa.eu/european-citizens-initiative/meeting-organisers-european-citizens-initiative-stop-destroying-videogames-european-commission-2026-02-23-0_en)
- The Commission's Communication was published 16 June 2026 (press release IP/26/1369). It found a mandatory duty to keep games playable after commercial support ends would not be proportionate at this stage. It cited copyright (rights holders decide "whether and how" works are used), publishers' costs and security. — [European Audiovisual Observatory, IRIS 2026-7](https://merlin.obs.coe.int/article/10576); [Lewis Silkin](https://www.lewissilkin.com/insights/2026/06/24/european-commission-stops-short-of-imposing-mandatory-requirements-following-citi-102n4yy)
- Commitments: "explore with the industry the possibility of elaborating a self-regulatory code of conduct" on sunsetting games, and work with consumer authorities to raise awareness of existing rights (Unfair Commercial Practices Directive 2005/29/EC, Unfair Contract Terms Directive 93/13/EEC, Digital Content Directive 2019/770). — [IRIS 2026-7](https://merlin.obs.coe.int/article/10576)
- Industry talks were expected in late 2026. — [CDE Universidad de Almería](https://www.cde.ual.es/en/la-comision-colaborara-con-la-industria-sobre-la-desactivacion-de-videojuegos-por-parte-de-los-editores/)
- Organisers plan to push for amendments in the coming Digital Fairness Act. — [Gaming Amigos](https://www.gamingamigos.com/post/eu-commission-declines-new-laws-for-the-stop-killing-games)
- In France, UFC-Que Choisir began legal action over game shutdowns on 31 March 2026 (context: Ubisoft's *The Crew*, taken offline April 2024). — [IRIS 2026-7](https://merlin.obs.coe.int/article/10576)

### Inferences
- The EU gives a clearer statutory base for reimplementation than the US. Observing a running game and decompiling to learn file formats and interfaces fits Art. 5(3) and Art. 6. SAS v WPL means reimplementing behaviour and reading the original data formats is not copying expression. Art. 6 forbids using the information to make a program substantially similar in expression, so the reimplementation must be new code.
- Stop Killing Games is a consumer-law question about publishers. It changes nothing about copyright, so it gives no new right to copy or distribute game files.

### Gaps
- I could not fetch the Commission Communication or its C(2026) reference number; quotes come via the Audiovisual Observatory.
- One secondary site reported 45 MEPs backing a call for legislation and a lawsuit against Sony; unverified, left out.
- The Digital Fairness Act proposal's timing and content as of October 2026 was not checked.
- Whether the voluntary code of conduct talks have started was not confirmed.

## United Kingdom: CDPA s.50B and the Stop Killing Games petition

### Takeaway
UK law mirrors the EU directive (CDPA 1988 s.50A-50C, with s.296A voiding contrary terms). The UK petition got 189,887 signatures and a Westminster Hall debate on 3 November 2025; the government said it has no plans to change consumer law.

### Cited Findings
- CDPA 1988 s.50B permits decompilation by a lawful user where needed to get information for an independent program to interoperate, with conditions like those in Art. 6; s.50BA permits observing, studying and testing. — [legislation.gov.uk, s.50B](https://www.legislation.gov.uk/ukpga/1988/48/section/50B); [s.50BA](https://www.legislation.gov.uk/ukpga/1988/48/section/50BA) (not fetched this session; verify wording)
- Petition 702074, "Prohibit publishers irrevocably disabling video games they have already sold". Opened 14 January 2025, closed 14 July 2025, 189,887 signatures. — [UK Parliament petition](https://petition.parliament.uk/petitions/702074)
- Government response 3 February 2025: "There are no plans to amend UK consumer law on disabling video games." It pointed to the Consumer Rights Act 2015. — [UK Parliament petition](https://petition.parliament.uk/petitions/702074)
- The Petitions Committee asked for a revised response, saying the first did not answer the question. — [Overclock3D](https://overclock3d.net/news/software/mps-seek-a-revised-response-to-the-uks-stop-killing-games-petition/)
- Debate held 3 November 2025, led by Ben Goldsborough MP. Minister Stephanie Peacock said there were no plans to change the law, but companies may need to communicate better. — [Hansard](https://hansard.parliament.uk/commons/2025-11-03/debates/C603C07C-1BB5-41EF-A3C1-1AE1FEBBB95D/VideoGamesConsumerLaw); [Push Square](https://www.pushsquare.com/news/2025/11/the-uk-government-shoots-down-stop-killing-games-campaign-in-official-debate)

### Inferences
- UK and EU are close on reverse engineering. The UK has no DMCA-style triennial rulemaking; CDPA s.296ZA-296ZF handles anti-circumvention with a complaint route to the Secretary of State (not researched here).

### Gaps
- Whether a revised government response was ever published is unconfirmed.
- UK anti-circumvention (s.296ZA onward) and its preservation exceptions (s.42ZA, s.43 for libraries) were not researched.

## The "abandonware" myth

### Takeaway
Copyright does not lapse when a game leaves sale or its publisher closes. Sharing game files without permission is infringement in the US, EU and UK, whatever the motive.

### Cited Findings
- "Abandonware" is not a legal category. Out-of-print games stay under copyright, and sharing them is infringement unless the rights holder allows it. Some sites distribute with permission (e.g. World of Spectrum). — [Wikipedia, Abandonware](https://en.wikipedia.org/wiki/Abandonware); [How-To Geek](https://www.howtogeek.com/829307/what-is-abandonware-and-is-it-legal/)
- The US 1201 exemptions limit even accredited institutions to on-premises access; remote access was denied because of the risk of recreational use. — [eCFR 201.40](https://www.ecfr.gov/current/title-37/chapter-II/subchapter-A/part-201/section-201.40); [VGHF](https://gamehistory.org/dmca-2024-statement/)
- About 87% of US games released before 2010 are out of print. — [VGHF statement](https://gamehistory.org/dmca-2024-statement/)

### Inferences
- Copyright terms: US works made for hire run 95 years from publication (17 U.S.C. 302(c)); EU/UK run 70 years after the author's death. So a 1990s game stays protected until about the 2080s-2090s in the US. These terms are standard law but were not fetched in this session.
- If even libraries may not share preserved games off-premises, a public download site cannot claim to be preservation.

### Gaps
- No primary statement from ESA or VGHF on the word "abandonware" itself was found.

## How reimplementation projects stay legal, and why some were taken down

### Takeaway
Projects that last ship only new code and make the player supply their own copy of the original data. Takedowns hit projects that shipped original assets or executables built from them (Mario 64 PC port), whose code came from decompiling the original line by line and therefore arguably copied it (re3/reVC), or that focused on circumvention (Yuzu).

### Cited Findings
- DevilutionX needs the player's own DIABDAT.MPQ and suggests buying Diablo from GOG or Battle.net; the shareware spawn.mpq runs the demo part. It states Diablo is Blizzard's copyright and the project is not endorsed. — [DevilutionX README](https://github.com/diasurgical/devilutionX)
- Take-Two filed a DMCA notice with GitHub (19 February 2021) against GTAmodding/re3 and about 200 forks (Switch, Vita, Wii U, PSP and other ports), saying the content was unlicensed Take-Two material. — [GitHub DMCA repository](https://github.com/github/dmca/blob/master/2021/02/2021-02-19-take-two.md)
- Take-Two sued the developers in California in September 2021. The defendants argued fair use. The parties reported a settlement in principle in early 2023; the terms were not published. — [VGC](https://videogameschronicle.com/news/take-two-is-suing-the-creators-of-a-gta-3-and-vice-city-reverse-engineering-projects); [NME](https://www.nme.com/news/take-two-and-grand-theft-auto-modders-will-settle-legal-battle-3394677)
- re3/reVC needed the player's own game data, yet were still targeted. The code was a decompilation of the original executables. — [VGC](https://videogameschronicle.com/news/take-two-is-suing-the-creators-of-a-gta-3-and-vice-city-reverse-engineering-projects); [GamingOnLinux](https://gamingonlinux.com/2021/11/gta-modders-behind-re3-and-revc-fire-back-in-court/page=4)
- Nintendo's lawyers sent DMCA notices against the Super Mario 64 PC port, which grew from the 2019 decompilation project. The notices targeted sites hosting the compiled executable, which contained the game's assets. — [VGC](https://www.videogameschronicle.com/news/nintendo-takes-action-against-mario-64-pc-port/); [PCGamesN](https://www.pcgamesn.com/super-mario-64/pc-port-copyright)
- Yuzu: DMCA anti-circumvention and anti-trafficking claims; $2.4m settlement and injunction in March 2024; later notices hit forks. — [ChannelNews](https://www.channelnews.com.au/nintendo-settles-lawsuit-with-tropic-haze/)

### Inferences
- The safe pattern: new code, no original assets in the repository or builds, the player brings their own data, keys are never shipped, and documented independent work (a clean room or at least written specs, not pasted decompiled code). OpenRCT2 (needs RollerCoaster Tycoon 2 data), ScummVM (needs the original game files) and OpenTTD (original-data option, or free OpenGFX/OpenSFX/OpenMSX replacements) follow this pattern. These project details come from general knowledge and were not fetched in this session.
- re3 shows "bring your own data" is necessary but not enough. A matching decompilation that copies the original's structure line by line is a derivative of the code and can be attacked even without assets. Clean-room separation and Art. 6's ban on "substantially similar" expression point the same way.
- Several matching decompilations (e.g. Ocarina of Time, Mario 64) have not been sued directly. Rights holders have gone after shipped binaries and ports. This is enforcement practice, not a legal safe harbour.

### Gaps
- OpenRCT2, ScummVM and OpenTTD READMEs were not fetched; confirm their data requirements before citing.
- The re3 settlement terms, and whether the projects came back, are not public in the sources found.
- Other takedowns (e.g. AM2R, Pokemon Uranium, Nintendo's 2024-2025 actions against emulators and ports) were not checked this session.

## Rights holders who support preservation

### Takeaway
Some rights holders release engine source code under the GPL without the assets. Others keep old games on sale and patched. Both show the model a reimplementation follows: code is open, game data stays a purchase.

### Cited Findings
- EA released source for the Tiberian Dawn and Red Alert DLLs under GPL v3 with the Remastered Collection in 2020. It had no assets and no engine. — [Neowin](https://www.neowin.net/amp/tiberian-dawn-and-red-alert-will-be-partially-open-sourced-alongside-remaster-launch/); [VGC](https://videogameschronicle.com/news/ea-says-it-will-release-the-command-conquer-source-code-in-a-key-moment-for-the-industry)
- On 27 February 2025 EA released GPL source for Command & Conquer Tiberian Dawn, Red Alert, Renegade, and Generals/Zero Hour on GitHub. Players still need to own the games for the assets. — [Phoronix](https://phoronix.com/news/EA-Open-Source-CnC-Red-Alert)
- GOG launched its Preservation Program with about 100 games, maintaining compatibility with modern systems. Each has a stamp and a Preservation Log. Examples include Heroes of Might and Magic 3 Complete, Dungeon Keeper 2, Theme Hospital, Diablo + Hellfire and System Shock 2. The date conflicts: GOG's press page says 13 October 2024, while other coverage dates it 13 November 2024. — [GOG pressroom](https://www.gog.com/pressroom/gog-re-releases-100-improved-classic-games-as-part-of-the-new-gog-preservation-program); [GOG blog](https://www.gog.com/blog/the-gog-preservation-program/); [Gematsu](https://gematsu.com/2024/11/gog-preservation-program-announced-preserving-old-games-for-current-and-future-pc-setups)

### Inferences
- id Software's GPL source releases (Doom, Quake, Quake II/III, Wolfenstein 3D) set the "code free, data bought" model that source ports follow. Nightdive Studios licenses and remasters old games (System Shock, Quake, Doom with id/Bethesda). These are well known but were not checked in this session; source them before publishing.

### Gaps
- id Software release dates and licences were not verified (from memory: Doom source 1997, relicensed GPL 1999; Quake 1999; Quake III 2005).
- Nightdive's projects and stance on preservation were not researched.
- GOG launch date conflict (October vs November 2024) is unresolved. Most coverage, and Gematsu's November 2024 article, suggests November 13, 2024; GOG's press page date as reported may be wrong.
