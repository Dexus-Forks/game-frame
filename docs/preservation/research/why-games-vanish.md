# Notes: why games disappear

## 1. The VGHF / SPN game availability study (2023) and follow-ups

### Takeaway
The July 2023 study by the Video Game History Foundation (VGHF) and Software Preservation Network (SPN) found that 87% of classic US-released games (pre-2010) are out of print, with only 13% commercially available. No later update to the study itself was found by October 2026. The figure is still the standard one used in policy fights, for example after the 2024 DMCA ruling.

### Cited Findings
- Published by VGHF and SPN. The explainer post is dated July 10, 2023, and was written by Phil Salvador. The full report is on Zenodo (doi 10.5281/zenodo.8161056). — [VGHF explainer](https://gamehistory.org/study-explainer)
- Headline: 87% of classic games released in the US are out of print, so 13% are in print (margin of error ±2.5%). No five-year period examined rose above 20% availability. — [VGHF explainer](https://gamehistory.org/study-explainer)
- A more precise figure is 13.27% available through commercial means. The authors call the rest "critically endangered". — [Exputer summary](https://exputer.com/news/games/87-classic-games-disappeared/); [GeekWire](https://www.geekwire.com/2023/researchers-find-87-of-u-s-classic-video-games-are-out-of-print-and-critically-endangered/)
- Scope: US-released games from before 2010, which the authors chose as roughly when digital distribution took off. The general sample was 1,500 random games from MobyGames. More than 4,000 games were studied in total once the platform samples are counted. Data is accurate as of April 15, 2023. — [VGHF explainer](https://gamehistory.org/study-explainer)
- Platform figures:
  - Commodore 64: 4.5% available, mostly only through Antstream Arcade. Without Antstream it would be 0.75%.
  - PlayStation 2: 12%.
  - Game Boy family: the whole library was studied, and 5.87% was in print as of April 2023. Another 6.5% had been in print only on the 3DS and Wii U eShops before they closed.
  - Games from before 1985: under 3%.
  — [VGHF explainer](https://gamehistory.org/study-explainer)
- Definition of "available": a yes or no answer. A re-release counted only if it was not "substantially different" from the original. Expensive collectibles, raw source code only, and sales only for defunct vintage hardware did not count. An average user had to be able to play the game. — [VGHF explainer](https://gamehistory.org/study-explainer)
- Method: volunteers from VGHF and the University of Washington Information School worked for about four months, and the results were double-checked. Penn State University Libraries' Data Learning Center and SPN's Law & Policy Working Group gave input. — [VGHF explainer](https://gamehistory.org/study-explainer)
- About 1,000 unique games fell out of release when the 3DS and Wii U eShops closed. — [VGHF explainer](https://gamehistory.org/study-explainer)
- Policy follow-up: in October 2024 the US Copyright Office denied SPN's DMCA exemption request (backed by VGHF), which would have let libraries give remote access to out-of-print games for research. The ruling is on p. 191 of the Register's Recommendation. VGHF's response cited the 87% figure. — [VGHF statement](https://gamehistory.org/dmca-2024-statement/); [VGC](https://videogameschronicle.com/news/us-copyright-office-refuses-to-allow-remote-access-to-out-of-print-games)
- Reasons the Office gave: the single-user and on-premises limits were unclear, proponents had not shown the use was non-infringing, and it agreed with the ESA that remote access could harm the market for re-releases. TechSpot reports that this was the fourth rejection of such a request since 2015. — [TechSpot](https://www.techspot.com/community/topics/us-copyright-office-denies-dmca-exemption-deals-blow-to-video-game-preservation-efforts.288839/)
- VGHF opened its digital library to the public on January 30, 2025. — [InfoDocket](https://www.infodocket.com/2025/01/24/report-video-game-history-foundations-long-awaited-digital-library-will-be-available-next-week/)
- At GDC 2026 (March 9–13), VGHF said it was helping the University of Washington iSchool with a study on game preservation and access in education, with initial findings presented on March 12. VGHF also called 2025 a hard year for funding in the field. — [VGHF GDC 2026](https://gamehistory.org/gdc-2026/)

### Inferences
- The 13% figure is a floor for "legally playable without piracy or vintage hardware". It is not a count of games that are lost. Many out-of-print games still exist on original media or in archives.
- Availability that rests on one service is fragile: C64 availability would fall from 4.5% to 0.75% if Antstream closed, and the Game Boy figures dropped when the eShops closed. A storefront is not a preservation strategy.

### Gaps
- No 2024–2026 re-run of the availability survey with new figures is known.
- The names of the full report's authors were not on the explainer page. They are in the Zenodo PDF.
- What the University of Washington 2026 education study found is not confirmed.

## 2. How legally owned games become unplayable

### Takeaway
Games break when the platform moves on (16-bit code on 64-bit Windows, 32-bit apps on macOS, DRM drivers, old graphics APIs). They also break or vanish when services shut down: online servers, licence revocation, store delistings and console store closures. Several of these changes landed between 2014 and 2026, and more are scheduled (Rosetta 2).

### Cited Findings
**Operating systems and hardware**
- Microsoft: x64 versions of Windows do not support 16-bit programs, processes or components. The fix Microsoft gives is a 32-bit Windows install (dual boot or VM). — [Microsoft KB 896458](https://support.microsoft.com/kb/896458)
- The technical reason is that x86-64 long mode has no virtual 8086 mode, so 64-bit Windows NT has no NTVDM. This comes from a secondary source. — [Microsoft Q&A](https://learn.microsoft.com/en-us/answers/a/4765604)
- Windows 11 ships only for 64-bit processors, so the 32-bit Windows route above is closing. — [Microsoft Windows 11 specifications](https://www.microsoft.com/windows/windows-11-specifications) (not confirmed)
- macOS Catalina (10.15, 2019) dropped 32-bit apps entirely. Warnings had started in High Sierra and Mojave. Apple's stated reason was that 64-bit is the norm and that Metal only works with 64-bit apps. — [MacPaw](https://macpaw.com/how-to/32-bit-apps-not-working-catalina); [MacRumors](https://www.macrumors.com/guide/32-bit-mac-apps/) (no Apple primary page was retrieved)
- Rosetta 2: at WWDC 2025 Apple said Rosetta 2 stays fully supported through macOS 27, and will be largely discontinued from macOS 28. Apple will keep a subset "aimed at supporting older unmaintained gaming titles" that rely on Intel frameworks. From macOS 26.4, apps that use Rosetta show a warning that they will stop working. macOS 27 drops Intel Macs. — [TechRepublic](https://www.techrepublic.com/article/news-apple-macos-27-drops-intel-mac-support-rosetta/); [MacRumors, Feb 16 2026](https://www.macrumors.com/2026/02/16/macos-tahoe-26-4-rosetta-2-warnings/); [AppleInsider, Jun 12 2026](https://appleinsider.com/articles/26/06/12/how-and-when-macos-will-finally-stop-support-for-intel-apps). Conflict: one AppleInsider piece suggests macOS 27 itself removes Rosetta, while other sources say macOS 27 is the last full version.
- DRM: Windows 10 (2015) does not run games protected by SafeDisc or older SecuROM. A Microsoft representative cited security holes from deeply embedded DRM. SafeDisc relies on the driver secdrv.sys, which Microsoft supported up to Windows 8.1. Games named in reports include GTA III, The Sims, Flight Simulator 2004 and Battlefield 1942 (roughly 2003–2008 releases). — [PC Gamer](https://www.pcgamer.com/au/report-games-with-securom-and-safedisc-drm-wont-run-under-windows-10); [Windows Central](https://windowscentral.com/windows-10-wont-run-games-safedisc-or-securom-drm)
- Graphics APIs: games written for 3dfx Glide need third-party wrappers on modern PCs. dgVoodoo2 covers Glide and DirectX 1–9 and outputs Direct3D 11. nGlide is Glide-only, outputs D3D9 or Vulkan, and is closed source. The open-source OpenGlide is "fairly dead". — [dgVoodoo2](https://sites.google.com/view/dgvoodoo2/); [Emulation General wiki](https://emulation.gametechwiki.com/index.php/Special:MobileDiff/41557) (community sources)

**Online services and licence revocation**
- The Crew (Ubisoft): delisted December 2023, servers closed March 31, 2024, and player licences revoked from April 2024, with no offline mode. — [Stop Killing Games wiki](https://stopkillinggames.wiki.gg/wiki/The_Crew)
- Lawsuits followed. A proposed US class action was filed on November 4, 2024 (E.D. California), and its status is unknown. On March 31, 2026 the French consumer group UFC-Que Choisir announced its own suit against Ubisoft. — [Gigazine](https://gigazine.net/gsc_news/en/20241112-ubisoft-lawsuit-the-crew-shutdown); [GamingOnLinux, Mar 2026](https://www.gamingonlinux.com/2026/03/french-consumer-group-ufc-que-choisir-sues-ubisoft-over-the-crew-shutdown/page=1)
- California's law requiring sellers to disclose that buyers get a licence, not ownership, was signed in September 2024 and took effect in 2025. — [Stop Killing Games wiki](https://stopkillinggames.wiki.gg/wiki/The_Crew)
- "Stop Destroying Videogames", the EU citizens' initiative started by Ross Scott, gathered 1,294,188 validated signatures and met the thresholds in 24 member states. The European Parliament held a hearing on April 16, 2026 (IMCO, JURI, PETI). The Commission adopted its reply on June 16, 2026. — [heise](https://heise.de/-11154579); [European Citizens' Initiative](https://citizens-initiative.europa.eu/news/european-commission-replies-stop-destroying-videogames-initiative-2026-06-16_cs); [LevelUp](https://www.levelup.com/noticias/stop-killing-games-la-union-europea-responde-a-la-iniciativa-tras-superar-el-millon-de-firmas-que-sigue-en-esta-lucha-contra-las-politicas-de-los-editores-de-videojuegos/)
- Wii online play ended in 2014 when the GameSpy agreement ended. — [Wii Shop Channel, Wikipedia](https://Www.Wikipedia.org/wiki/Wii_Point)

**Storefront delistings (licence expiry)**
- Rock Band 4 was delisted from PlayStation and Xbox in October 2025 because its core soundtrack licences expired, and its DLC was removed too. Owners can still download it. — [Game Informer](https://gameinformer.com/2025/10/01/rock-band-4-will-be-delisted-this-weekend)
- Spec Ops: The Line was delisted from Steam and other stores because "partnership licenses" expired. Project Gotham Racing 4 is reportedly gone because its car licences expired. — [KeenGamer](https://www.keengamer.com/articles/features/others/why-games-get-delisted-and-why-its-hurting-gaming/)
- Sony said: "we delist assets with licensed music when the licenses expire". — [VGC](https://www.videogameschronicle.com/news/sony-claims-it-delisted-the-kotor-remake-trailer-due-to-music-license-expiring)

**Console store closures**
- Wii Shop Channel: points sales ended March 26–27, 2018 (time zone dependent), and software purchases ended January 30–31, 2019. Nintendo said redownloads would also end later. — [Nintendo AU Q&A](https://www.nintendo.com/au/support/articles/q-and-a/); [Nintendo Life](https://www.nintendolife.com/news/2017/09/nintendo_has_announced_the_end_of_the_wii_shop_channel)
- 3DS and Wii U eShops: adding funds by card ended August 29, 2022, and all purchases (including free demos) ended March 27, 2023. Redownloads and online play continue "for the foreseeable future". — [VGC](https://www.videogameschronicle.com/news/reminder-the-wii-u-and-3ds-eshops-close-down-permanently-today); [Pocket Tactics](https://www.pockettactics.com/3ds-wii-u-eshop-closure)
- Nintendo's first FAQ said it had "no plans to offer classic content in other ways". That line was later removed. — [Gizmodo](https://gizmodo.com/3ds-wii-u-eshops-closing-when-shut-down-best-games-zeld-1850207476)

### Inferences
- There are two failure modes. In the first, the game still exists but cannot run on current platforms (OS, DRM, API). In the second, the game cannot be bought, or stops working, because a service ended. Reimplementation answers the first directly. It helps owners in the second only where the game files survive.
- Even Apple's own Rosetta plan names "older unmaintained gaming titles" as a category that needs special handling. That shows unmaintained games are a recognised problem.

### Gaps
- Microsoft's own wording on Windows 11 being 64-bit only, and Apple's own Catalina support page, are not confirmed. Both are cited from secondary sources.
- No primary figure is known for how many games depend on DirectDraw, Glide or D3D 1–7.
- The substance of the EU Commission's June 2026 reply is covered in the law notes.

## 3. Games still on sale but no longer updated (e.g. Banished)

### Takeaway
Banished is a clear example: a one-person studio's game, still sold, that has had no update for years. Its last listed build is 1.0.7. GOG's database dates that build to September 2017, so "last updated around 2016" should be corrected to 2017. No dataset measures how common abandoned-but-on-sale games are.

### Cited Findings
- Banished was released on Windows on February 18, 2014, developed and published by Shining Rock Software. Its designer is Luke Hodorowicz. — [Wikipedia](https://en.wikipedia.org/wiki/Banished_(video_game))
- Update 1.0.5 (May 2016) added a new modkit and raised the memory allowance to 1 GB, and announced a 1.0.6 multi-platform update. The developer had said Mac and Linux builds would come next. — [Shining Rock blog](https://shiningrocksoftware.com/?p=2987); [GamingOnLinux 2014](https://www.gamingonlinux.com/2014/10/citybuilding-game-banished-is-still-being-ported-to-linux-woo)
- GOG's build database lists Banished 1.0.7 for Windows, published September 14, 2017. — [GOGDB](https://www.gogdb.org/product/1207660783/build/50512346553854639)
- Apple's Rosetta 2 plan explicitly keeps support for "older unmaintained gaming titles". This is a platform holder acknowledging the category. — [TechRepublic](https://www.techrepublic.com/article/news-apple-macos-27-drops-intel-mac-support-rosetta/)
- What breaks over time, as in section 2: 32-bit binaries on macOS (Catalina, 2019), Intel-only binaries on Apple silicon (Rosetta wind-down from macOS 28), DRM drivers (Windows 10), and old graphics APIs that need wrappers. — sources in section 2

### Inferences
- A game that is still sold but frozen is only as playable as the next OS change allows. Nobody upstream will ship a fix, so a source-compatible reimplementation is the only route that does not depend on the original developer.

### Gaps
- No figure is known for how many Steam or GOG games have had no update in five or more years. Sources such as SteamDB might allow a count. No published study is known.
- It is not confirmed whether the Mac or Linux builds of Banished ever shipped, or whether Banished has a 64-bit build. GOGDB lists the build's bitness as "Any".

## 4. Bit rot, media decay and lost source code

### Takeaway
Recordable optical media last anywhere from about 5 years to more than 100 years, depending on dye, metal layer and storage. Magnetic media last about 10 years with moderate care. Publishers have lost final source code and assets: Konami for Silent Hill 2 and 3, Square Enix for Kingdom Hearts. Remasters of those games had to be rebuilt from incomplete code or retail discs.

### Cited Findings
- CCI Notes 19/1, "Longevity of Recordable CDs, DVDs and Blu-rays" (Joe Iraci, revised 2019, web page dated 2020-01-07) gives average longevity:
  - CD-R, phthalocyanine dye with gold layer: over 100 years.
  - CD-R, phthalocyanine with silver alloy: 50–100 years.
  - CD-R, cyanine or azo with silver alloy: 20–50 years.
  - DVD-R: 50–100 years with gold, 10–20 years with silver alloy.
  - CD-RW: 20–50 years.
  - Pressed CD: 50–100 years.
  - Pressed DVD or BD: 10–20 years.
  - BD-R: 5–20 years.
  - DVD-RW: 5–10 years.
  The ratings are "based on temperature and RH effects only". — [CCI Notes 19/1](https://www.canada.ca/en/conservation-institute/services/conservation-preservation-publications/canadian-conservation-institute-notes/longevity-recordable-cds-dvds.html)
- Some reports put disc lifetimes under 5 years. In Iraci's research, accelerated-ageing predictions held up for CD-R but not for DVD-R. — [Iraci, Restaurator (De Gruyter)](https://www.degruyterbrill.com/document/doi/10.1515/res-2016-0032/pdf)
- With moderate care, most magnetic media last about 10 years. Archival tape stored properly lasts 30 years or more. Floppy disks are "not recommended for archival storage". — [UMass OIT, mirrored by Software Preservation Society](https://softpres.org/cache/DigitalMediaLifeExpectancyAndCare.html)
- Silent Hill HD Collection (2012): the porting team worked from incomplete source code because the final code had not been archived. Producer Tomm Hulett said Konami no longer had the final versions of Silent Hill 2 and 3. — [Wikipedia](https://en.wikipedia.org/wiki/Silent_Hill_HD_Collection); [Push Square](https://pushsquare.com/news/2012/05/silent_hill_hd_collection_ported_from_incomplete_code)
- Kingdom Hearts HD remaster: director Tetsuya Nomura said the original data "was lost, so we had to research". Assets were recreated, reportedly by extracting data from retail copies. — [Game Revolution](https://www.gamerevolution.com/news/5796-square-lost-kingdom-hearts-assets-forced-to-recreate-hd-version-from-scratch); [IMDb trivia](https://imdb.com/title/tt4877728/trivia) (weak source)

### Inferences
- Pressed game discs and floppies outlive many recordable formats, but their working lifetime is set by the readers and platforms too. CCI notes that well-kept discs can outlast the drives that read them.
- The Silent Hill and Kingdom Hearts cases show that even rights holders sometimes depend on the shipped binary. That is the same starting point as a reverse-engineering project.

### Gaps
- No primary (CCI or Library of Congress) lifetime figure for floppy disks is known.
- No systematic count of games with lost source code is known. Other commonly cited cases are not confirmed.

## 5. Orphaned and abandoned games (unknown or defunct rights holders)

### Takeaway
When rights are split or undocumented, even a willing re-publisher cannot clear them. No One Lives Forever (2000) is the standard case: rights possibly split between Warner Bros., Activision and 20th Century Fox, a re-release attempt abandoned, and the game still not legally purchasable at its 25th anniversary.

### Cited Findings
- No One Lives Forever and its sequel may have their rights split across Warner Bros. (owner of the developer Monolith and the LithTech engine), Activision and 20th Century Fox. Asked, the companies reportedly did not know whether they held rights, but would sue if the game were made. Activision reportedly said its records predated digital storage. — [Techdirt](https://www.techdirt.com/company/night-dive-studios/)
- Night Dive Studios filed trademarks for the series around 2014, later stopped its re-release attempt, and planned to let the trademark lapse. — [Softpedia 2014](https://games.softpedia.com/blog/No-One-Lives-Forever-Trademarks-Filed-Games-Might-Get-Republished-440305.shtml); [Techdirt](https://www.techdirt.com/company/night-dive-studios/)
- At its 25th anniversary (2025), NOLF still could not be bought legally. — [Techdirt](https://www.techdirt.com/company/nightdive/)
- The legal route for institutions to share out-of-print games remotely stays closed in the US after the 2024 DMCA ruling. — [VGHF statement](https://gamehistory.org/dmca-2024-statement/)

### Inferences
- Orphaned rights mean the market cannot fix availability even when demand exists. The game survives only on original copies and the know-how needed to run them.

### Gaps
- No quantitative estimate of orphaned or defunct-owner games is known. The VGHF study measured availability, not rights status.
- The EU Orphan Works Directive and how it applies to games are not covered.
