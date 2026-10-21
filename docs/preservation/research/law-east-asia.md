# Notes: East Asia

Scope: Japan, mainland China, South Korea, Taiwan and Hong Kong. This is not legal advice.

## 1. Statutory exceptions for reverse engineering, decompiling, and studying or testing software

### Takeaway
Korea has the clearest statute: a decompilation exception limited to interoperability (Art. 101-4) plus a separate "examine, research or test" exception. China lets a lawful user study a program's design ideas by running it (Software Regulations Art. 17) and treats reverse engineering of a lawfully obtained product as a defence to trade-secret claims. Taiwan has no decompilation article and relies on general fair use (Art. 65) plus the owner's right to adapt a program for use on a machine (Art. 59). Japan relies on the 2018 "flexible" exceptions (Art. 30-4 and 47-4), which the government says cover program analysis. Hong Kong's legislature deleted a proposed decompilation clause in 1997 and left it to fair dealing.

### Cited Findings

**Mainland China**
- Regulations on the Protection of Computer Software (State Council; 2001, revised 2011 and 2013, current text in force from 30 Jan 2013), Art. 17: a person who uses software by installing, displaying, transmitting or storing it, in order to study and research the design ideas and principles it contains, needs no permission from the copyright owner and pays no fee. Art. 6: protection does not extend to ideas, processes, methods of operation or mathematical concepts used to develop software. — [gov.cn State Council text, 2013-02-08](https://www.gov.cn/zhengce/2013-02/08/content_2602615.htm); [fadada summary of current version](https://www.fadada.com/article/detail-18344)
- Art. 17 permits use "by installing, displaying, transmitting or storing". It does not expressly permit decompilation or reproduction of code beyond that. — same sources (wording); the narrow reading is an inference, see Inferences.
- Supreme People's Court, Provisions on Several Issues Concerning the Application of Law in Trial of Civil Cases of Infringement of Trade Secrets (2020): getting information through independent research or reverse engineering is not trade-secret infringement. Reverse engineering means analysing, by technical means, a product obtained from public channels. The defence fails if the defendant first obtained the secret by improper means. — [chinaipr.com, 12 Sep 2020](https://chinaipr.com/2020/09/12/supreme-peoples-court-releases-trade-secret-ji/); [China IP Law Update](https://www.chinaiplawupdate.com/2020/09/chinas-supreme-peoples-court-releases-provisions-on-several-issues-concerning-the-application-of-law-in-trial-of-civil-cases-of-infringement-of-trade-secrets/)
- Supreme People's Court case (2022) No. 1592 is indexed on WIPO Lex under reverse engineering, computer software, former employees and punitive damages (facts not covered). — [WIPO Lex judgment 3397](https://www.wipo.int/wipolex/en/judgments/details/3397)

**South Korea**
- Copyright Act Art. 101-4 (Program Code Reverse Analysis / Decompilation of Program Codes), para 1: a person who lawfully uses a program, or a person they authorise, may decompile the program without the owner's permission, only for the parts needed for interoperability, and only where the information needed for interoperability cannot easily be obtained and obtaining it is unavoidable. Para 2: the information obtained may not be used for other purposes, or given to third parties, or used (per the commonly cited third item) to develop a substantially similar program. — [ITWiki quoting the article](https://itwiki.kr/w/%EB%A6%AC%EB%B2%84%EC%8A%A4_%EC%97%94%EC%A7%80%EB%8B%88%EC%96%B4%EB%A7%81); [KLRI article listing (Korean)](https://elaw.klri.re.kr/kor_service/lawViewTitle.do?hseq=42726)
- Korea has allowed reverse analysis in principle since 2000, following Art. 6 of the EU Software Directive. Decompilation is defined in Art. 2(34) as reproduction and translation, which Art. 101-4 exempts. — [ITWiki](https://itwiki.kr/w/%EB%A6%AC%EB%B2%84%EC%8A%A4_%EC%97%94%EC%A7%80%EB%8B%88%EC%96%B4%EB%A7%81); [Korea Copyright Commission paper 231](https://copyright.or.kr/eng/doc/quarterly_pdf/231%20Law%20and%20Policy%20Study%20on%20Reverse%20Engineering%20Provisions%20under%20the%20Copyright%20Act.pdf)
- A separate limitation on program rights, Art. 101-3(1)(6), lets a lawful user examine, research or test a program to understand its underlying ideas and principles. A 2009 Copyright Commission paper argues analysis should be allowed as widely as possible under this item and fair use, except to build a competing program. — [Korea Copyright Commission paper 045](https://www.copyright.or.kr/eng/doc/quarterly_pdf/045%20A%20New%20Analysis%20of%20Reverse%20Engineering%20Regulations%20for%20the%20Prevention%20of%20Hacking.pdf) (uses older numbering "101c")
- The Ministry of Culture, Sports and Tourism has studied revising or deleting Art. 101-4 since 2016. — [Korea Copyright Commission paper 231](https://copyright.or.kr/eng/doc/quarterly_pdf/231%20Law%20and%20Policy%20Study%20on%20Reverse%20Engineering%20Provisions%20under%20the%20Copyright%20Act.pdf)
- Korea also has a general fair-use clause, Art. 35-5 (added 2011). (not confirmed)

**Taiwan**
- Copyright Act (last amended 15 June 2022), Art. 59: the owner of a lawful copy of a computer program may modify it where necessary for use on a machine. The owner may also make backup copies, for personal use only. If the owner loses ownership other than by destruction, the modified and backup copies must be destroyed unless the rights holder agrees otherwise. — [Laws & Regulations Database of the ROC, Copyright Act](https://law.moj.gov.tw/ENG/LawClass/LawAll.aspx?pcode=J0070017)
- Art. 65: fair use is not infringement; four factors, including purpose and commercial nature and the effect on the work's current and potential market value. — [same](https://law.moj.gov.tw/ENG/LawClass/LawAll.aspx?pcode=J0070017)
- Taiwan's IP Office analysed decompilation of programs as a fair-use question under Art. 65 (article 89-19, 2000, Chinese). — [TIPO PDF](https://www.tipo.gov.tw/public/AttachmentORG/電腦程式還原工程合理使用之界線(89-19).pdf)
- Taiwan's anti-circumvention article lists "reverse engineering" as an exception (see section 2), which signals the legislature treats it as legitimate. — [Copyright Act Art. 80-2(3)(8)](https://law.moj.gov.tw/ENG/LawClass/LawAll.aspx?pcode=J0070017)

**Hong Kong**
- The 1997 Copyright Bill had a clause 60 letting lawful users decompile programs for interoperability. The Administration proposed deleting it and widening the fair-dealing clause (clause 37, now s.38 research and private study) instead. — [LegCo Bills Committee minutes, PLC Paper CB(1)39/97-98](https://legco.gov.hk/yr96-97/english/bc/bc14/minutes/bc143004.htm); [BSA submission](https://legco.gov.hk/yr96-97/english/bc/bc14/papers/b141804a.htm)
- The enacted Ordinance (Cap. 528) s.60 covers back-up copies of computer programs. A 2001-02 bill amended ss.60-61 so that a person with a contractual right to use a program is treated as having that right for s.60(2) despite licence restrictions. — [LegCo bill c056-e](https://www.legco.gov.hk/yr01-02/english/bills/c056-e.pdf)

**Japan**
- The 2018 amendment (in force 1 January 2019) introduced Art. 30-4, which permits use of a work that is not for enjoying the thoughts or sentiments expressed in it, unless it unreasonably harms the copyright owner's interests. — [PrivacyWorld commentary](https://www.privacyworld.blog/?p=10720); [Korea Copyright Commission paper 285 on Art. 30-4](https://www.copyright.or.kr/eng/doc/quarterly_pdf/285%20Implications%20of%20%E2%80%98Smooth%20Rights%20Restriction%20Provisions%E2%80%99%20under%20the%20Japanese%20Copyright%20Act%20as%20Applied%20to%20Digital%20Works%20%20Focusing%20on%20explaining%20Article%2030%204.pdf); [CRIC English translation](https://www.cric.or.jp/english/clj/cloj7.html)
- The Korean commentary notes Art. 30-4 treats digital and program works distinctly and frames protection around "enjoyment". — [Korea Copyright Commission paper 285](https://www.copyright.or.kr/eng/doc/quarterly_pdf/285%20Implications%20of%20%E2%80%98Smooth%20Rights%20Restriction%20Provisions%E2%80%99%20under%20the%20Japanese%20Copyright%20Act%20as%20Applied%20to%20Digital%20Works%20%20Focusing%20on%20explaining%20Article%2030%204.pdf)
- Art. 47-3: the owner of a copy of a program may reproduce or adapt it to the extent needed to use it on a computer (backup, adaptation). Art. 47-4: incidental copies made in computer use (cache, maintenance). (not confirmed against the official text)
- The Agency for Cultural Affairs' 2019 explanatory guidance on the flexible exceptions lists program analysis (reverse engineering) as an example of non-enjoyment use under Art. 30-4. (not confirmed)

### Inferences
- For GameFrame's use case (study a game you own, record behaviour, write new code), China Art. 17 and Korea Art. 101-3(1)(6) cover observing and running the program. Decompilation beyond interoperability is less clearly covered in China and Korea. Taiwan and Hong Kong rely on general fair use or fair dealing, so outcomes depend on facts.
- Korea's Art. 101-4 and China's trade-secret rule both protect the information gained, but neither permits sharing decompiled code. This fits a framework that never distributes code.
- Japan's Art. 30-4 is the broadest in theory, but it is tied to "non-enjoyment"; playing the game is enjoyment, analysing its code is not.

### Gaps
- Exact current Japanese text of Arts. 30-4, 47-3 and 47-4 and the ACA guidance on reverse engineering: not confirmed against the official text.
- Current wording of Hong Kong s.60 and s.61, and whether any later decompilation rule exists: not confirmed against the official text.
- Korea Art. 101-4(2) item list and Art. 101-3(1)(6) wording: not confirmed against the official KLRI text.
- No Chinese court case is known that applies Art. 17 to games.

## 2. Anti-circumvention of technological protection measures (TPMs), and exceptions

### Takeaway
All five jurisdictions ban dealing in circumvention tools, and most ban the act of circumventing access controls. Every one except Japan has a written exception for reverse engineering or interoperability (China, Taiwan, Korea, Hong Kong). Japan has two overlapping regimes (Copyright Act and the Unfair Competition Prevention Act, UCPA). Its 2018 UCPA amendment made selling save-data editing services a crime, which led to arrests from 2021.

### Cited Findings

**Japan**
- UCPA 2018 amendment, enacted 23 May 2018: widened "technological restriction measures" to include activation and licence authentication; made offering codes that disable such measures (e.g. illegal product keys) unfair competition; and added providing a circumvention service. METI lists enforcement in November 2018 and July 2019 without mapping each change to a date. — [METI English overview slides, 2018 revision](https://www.meti.go.jp/english/policy/economy/chizai/chiteki/pdf/english_2018rev.pdf); [METI UCPA page](https://www.meti.go.jp/english/policy/economy/chizai/chiteki/index.html)
- Press reporting calls this "the 2019" UCPA change. Input Magazine places it in late 2018. — [Inverse/Input](https://www.inverse.com/input/gaming/japanese-police-arrest-man-over-selling-mods-for-zelda-botw); [Nintendo Wire](https://nintendowire.com/news/2021/07/09/breath-of-the-wild-hacker-arrested-in-japan-for-selling-modified-save-games/)
- Penalties reported by ACCS: up to 5 years' imprisonment, a fine of up to 5 million yen, or both. — [Siliconera](https://siliconera.com/pokemon-sword-and-shield-hacker-arrested-in-japan) (citing ACCS)
- Japan's 2011 UCPA amendment (in force December 2011) added criminal penalties and removed the "only function" requirement, which let Japan stop magicon imports at customs from 2012. — [Gigazine, 22 Nov 2012, magicon import ban](https://wbgsv0a.gigazine.net/gsc_news/en/20121122-magicom-ban-import) (date details not confirmed)
- Copyright Act: "technological protection measures" (Art. 2(1)(xx)); the 2012 amendment brought encryption-based access protection in; the 2018 TPP-related amendment (in force 30 December 2018) added "technological utilisation restriction measures" (Art. 2(1)(xxi)) and treats circumventing them as infringement (Art. 113). Art. 120-2 penalises dealing in circumvention devices. (not confirmed against the official text)
- No interoperability or preservation exception to Japan's TPM rules is known.

**Mainland China**
- Copyright Law (amended 11 Nov 2020, in force 1 June 2021), Art. 49: no organisation or individual may intentionally circumvent or destroy technological measures without permission, except as laws or regulations allow. Art. 49 also bans making, importing or providing circumvention devices or services. — [China Justice Observer, Copyright Law 2020](https://chinajusticeobserver.com/law/x/copyright-law-of-china-20201111)
- Art. 50 allows circumvention, provided no tools are given to others and no other rights are infringed, for: (1) giving a small amount of published works to teachers or researchers for teaching or research, where not available through normal channels; (2) accessible formats for print-disabled people, not for profit; (3) official duties of State organs; (4) testing the security of computers, systems or networks; (5) encryption research or reverse engineering research of computer software. — [same](https://chinajusticeobserver.com/law/x/copyright-law-of-china-20201111)
- Software Regulations Art. 24 also prohibits intentionally circumventing or destroying measures protecting software copyright. (not confirmed)

**South Korea**
- Copyright Act Art. 104-2 (added by the 2011 KORUS FTA amendment) bans circumventing access-control measures. It has statutory exceptions, including encryption research and program code reverse analysis, plus exceptions set by Ministry notice that last only three years and must be remade every three years. — [ITWiki summary](https://itwiki.kr/w/%EB%A6%AC%EB%B2%84%EC%8A%A4_%EC%97%94%EC%A7%80%EB%8B%88%EC%96%B4%EB%A7%81); [Korea Copyright Commission paper 254, 2018 anti-circumvention rules](https://www.copyright.or.kr/eng/doc/quarterly_pdf/254%202018%20Anti-circumvention%20Rules%20of%20Korea%20Copyright%20Act.pdf); [paper 320 on exemptions notice](https://copyright.or.kr/eng/doc/quarterly_pdf/320%20A%20Study%20to%20prepare%20exemptions%20notice%20draft%20for%20the%20prohibition%20on%20circumvention%20of%20technological%20protection%20measures.pdf)
- Full numbered item list of Art. 104-2(1) and the current exemptions notice: not covered.

**Taiwan**
- Copyright Act Art. 80-2 (added 2004). Para 3 exceptions: national security; government agencies; archives, educational institutions or public libraries assessing whether to acquire; protecting minors; protecting personal data; security testing of computers or networks; encryption research; **reverse engineering**; uses under Arts. 44-63 and 65 (all other exceptions and fair use); other cases set by the competent authority. — [Laws & Regulations Database, Copyright Act](https://law.moj.gov.tw/ENG/LawClass/LawAll.aspx?pcode=J0070017); [Wikibooks annotated Art. 80-2](https://en.wikibooks.org/wiki/Annotated_Republic_of_China_Laws/Copyright_Act/Article_80-2)
- Art. 96-1: making, importing or offering circumvention tools (Art. 80-2(2)) is punishable by up to one year's imprisonment or detention and/or a fine of NT$20,000 to NT$250,000. — [same](https://law.moj.gov.tw/ENG/LawClass/LawAll.aspx?pcode=J0070017)

**Hong Kong**
- Copyright (Amendment) Ordinance 2007 added criminal offences for making, importing, exporting or dealing in circumvention devices, or providing commercial circumvention services; in force 25 April (2008 per the Customs release context). Before that, s.273 gave only civil remedies. — [HK Government press release, 27 Sep 2008](https://www.info.gov.hk/gia/general/200809/27/P200809270122_print.htm); [Sydney University Press chapter](https://open.sydneyuniversitypress.com.au/9781920898724/cldc-new-hope-for-consumers-of-digital-copyright-material-in-hong-kong.html)
- Statutory exceptions (s.273D) let a person circumvent, subject to conditions, for purposes including interoperability of independently created programs and research into cryptography; s.273H lets the Secretary add further exceptions. — [HK Government (CEDB) note for proponents](https://www.cedb.gov.hk/assets/resources/citb/consultations-and-publications/Note_for_Proponents_Circumvention_Eng.pdf); [CEDB consultation paper on exclusion list](https://www.cedb.gov.hk/assets/resources/citb/consultations-and-publications/Consultation_Paper_exclusion_list_Eng_full.pdf); [IAM](https://www.iam-media.com/article/consultation-launched-further-exemptions-circumventing-tpms)
- IIPA (rights-holder group) supported confining any added exceptions to the act of circumvention, not to dealing in devices. — [IIPA comments](https://iipa.org/files/uploads/2018/01/HongKongCommentsonTPMConsultationPapers.pdf)

### Inferences
- GameFrame reads the owner's own files and does not ship circumvention tools. Where a game's data is encrypted, China (Art. 50(5)), Korea (Art. 104-2 statutory exceptions), Taiwan (Art. 80-2(3)(8)) and Hong Kong (s.273D interoperability) all have a reverse engineering or interoperability path for the act itself. None permits giving tools to others.
- Japan is the riskiest place for tools or services that change game data or bypass console protections, because the UCPA covers services and is criminally enforced.

### Gaps
- Japan Copyright Act article numbers for TPM and access-control rules (2(1)(xx), 2(1)(xxi), 113, 120-2) not confirmed against text.
- Exact UCPA effective dates for each 2018 change (Nov 2018 vs July 2019).
- Hong Kong: exact 2007 commencement date and current s.273D wording.
- Korea: current exemptions notice under Art. 104-2 and whether it mentions games or preservation.

## 3. Preservation exceptions for libraries, archives and museums

### Takeaway
Each jurisdiction lets libraries copy items for preservation. China names museums and archives explicitly. Taiwan names archives and libraries. None is known to have a game-specific or software-specific preservation exception, and none lets a preservation copy override a TPM except Taiwan's narrow acquisition-assessment item.

### Cited Findings
- **China**, Copyright Law 2020 Art. 24(1)(8): a library, archive, memorial hall, museum or art gallery (and cultural centre) may reproduce a work in its collection for display or to preserve a copy. — [China Justice Observer](https://chinajusticeobserver.com/law/x/copyright-law-of-china-20201111)
- **Taiwan**, Copyright Act Art. 48: archives and libraries may reproduce collection works for a patron's personal research (part of a published work), where preservation is needed to avoid loss or damage, for out-of-print or hard-to-buy works requested by a similar institution, and for restoration during a digital-collection licence period. — [Laws & Regulations Database](https://law.moj.gov.tw/ENG/LawClass/LawAll.aspx?pcode=J0070017)
- **Korea**, Copyright Act Art. 31(1)(2): libraries may reproduce books etc. where needed to preserve them (2006 text). The proviso bars digital-format reproduction for items 1 and 3. — [WIPO Lex, Korean Copyright Act as amended to Act No. 8101 of 2006](https://www.wipo.int/wipolex/en/text/188273) (old version; current text not confirmed)
- **Japan**: Copyright Act Art. 31 (libraries); the National Diet Library is Japan's only legal deposit library, and since a 2000 revision deposit covers packaged electronic publications such as video games. The NDL holds over 9,600 game titles, mostly physical releases since October 2000; it said Switch 2 Game-Key Cards are not eligible because only physical media containing the content qualifies. — [Automaton, NDL and Game-Key Cards](https://automaton-media.com/en/news/nintendo-switch-2-game-key-cards-not-eligible-for-preservation-by-japans-national-diet-library-officials-say/); [VGC](https://www.videogameschronicle.com/news/japans-national-library-says-it-wont-be-preserving-switch-2-game-key-cards)
- The NDL put its game collection at about 7,200 titles as of December 2022. — [Automaton, Akamatsu](https://automaton-media.com/en/news/20230606-19332/)
- Ken Akamatsu (manga artist, House of Councillors member) has pushed since 2022 for a legal framework to preserve games and keep them playable, naming the NDL as the right body. — [Automaton, July 2022](https://automaton-media.com/en/news/20220714-14016/); [Automaton, June 2023](https://automaton-media.com/en/news/20230606-19332/)
- **Hong Kong**: Cap. 528 ss.46-53 are the library and archive provisions (s.51 replacement copies). (not confirmed)

### Inferences
- Deposit libraries (NDL) preserve physical media but not necessarily playability. No jurisdiction gives private individuals a preservation exception; GameFrame's model (owner studies own copy) sits under the software and fair-use exceptions in section 1, not under library rules.

### Gaps
- Current Korean Art. 31 text (amended several times since 2006).
- Hong Kong library sections: not confirmed against the official text.
- Whether the NDL may circumvent TPMs to preserve or make games playable: no source known.

## 4. Copyright term for software and games

### Takeaway
Japan and Korea are on 70 years; China, Taiwan and Hong Kong are on 50. Most commercial games are corporate works, so the clock usually runs from publication.

### Cited Findings
- **China**, Copyright Law 2020 Art. 23: works of legal persons (and works for hire held by them) are protected for 50 years, ending 31 December of the 50th year after first publication; if unpublished within 50 years of creation, no longer protected. — [China Justice Observer](https://chinajusticeobserver.com/law/x/copyright-law-of-china-20201111)
- **Taiwan**, Art. 30: life plus 50 years. Art. 33: works of juristic persons, 50 years from public release, or 50 years from completion if not released within 50 years. — [Laws & Regulations Database](https://law.moj.gov.tw/ENG/LawClass/LawAll.aspx?pcode=J0070017)
- **Korea**, 2006 text Art. 39: life plus 50 years. — [WIPO Lex 2006 text](https://www.wipo.int/wipolex/en/text/188273). The 2011 amendment extended this to life plus 70 (and 70 years from publication for works made for hire) from 1 July 2013. (not confirmed)
- **Japan**: the TPP-related amendment extended terms from 50 to 70 years (life plus 70; 70 years from publication for corporate works, Art. 53), in force 30 December 2018, not reviving expired works. (not confirmed)
- **Hong Kong**: life plus 50; computer-generated works 50 years from creation. (not confirmed)

### Inferences
- A corporate Japanese game published in 1980 is protected to the end of 2050 (70 years after publication). A Chinese or Taiwanese corporate game from 1980 would be protected to the end of 2030. GameFrame should not assume any 1980s commercial game is out of copyright.

### Gaps
- Japan, Korea and Hong Kong term articles: not confirmed against the official text.

## 5. Orphan works schemes

### Takeaway
Japan and Korea have government licence schemes for works whose owners cannot be found. Japan added a second, wider scheme for "unmanaged" published works, in force 26 May 2026. China, Taiwan and Hong Kong have no known general orphan-works scheme.

### Cited Findings
- **Japan**, long-standing Art. 67 ruling system: the Commissioner of the Agency for Cultural Affairs can authorise use when the owner cannot be found after diligent search. (not confirmed against the official text)
- **Japan**, Act No. 33 of 2023 created a new ruling system for published works that are not managed and whose owner's intentions cannot be confirmed: a user may get a time-limited licence from the ACA Commissioner after depositing compensation set at an ordinary royalty rate; designated private bodies handle procedures; the owner can ask the Commissioner to revoke the licence. Remaining provisions in force 26 May 2026. — [Digital Policy Alert, implementation](https://digitalpolicyalert.org/event/30237-compulsory-license-system-in-act-no-33-of-2023-partially-amending-the-copyright-act-was-implemented); [ACA, 2023 amendments](https://www.bunka.go.jp/english/policy/copyright/amendments_2023/); [Japanese Law Translation outline](https://www.japaneselawtranslation.go.jp/en/outlines/view/86); [MLex](https://www.mlex.com/mlex/articles/2343694)
- A May 2025 draft cabinet order proposed a JPY 13,800 application fee. Final adoption not confirmed. — [Digital Policy Alert, consultation](https://digitalpolicyalert.org/event/30199-agency-for-cultural-affairs-opened-consultation-on-the-draft-cabinet-order-partially-amending-the-enforcement-order-of-the-copyright-act)
- **Korea**, Copyright Act Art. 50: where a person, after considerable effort, cannot identify the owner of economic rights in a published work, they may use it with the approval of the Minister of Culture, Sports and Tourism by depositing compensation. — [WIPO Lex 2006 text](https://www.wipo.int/wipolex/en/text/188273) (old version)
- Comparative overview of obscure-copyright regimes in Asia. — [Asia IP](https://asiaiplaw.com/index.php/article/obscure-copyrights-in-asia-and-what-they-mean-for-vintage-films)

### Inferences
- For a defunct game studio whose rights holder is unknown, Japan's 2026 scheme is the most practical official route to a re-release or public archive copy. It is a licence to use, not a licence to break TPMs.

### Gaps
- Whether any game has been licensed under Japan's or Korea's scheme.
- Confirmation that China, Taiwan and Hong Kong lack any scheme (absence not proven).

## 6. Notable cases

### Takeaway
Japan has the densest case law: the R4 flash-cart cases (2009 injunction; 2013 damages of 95.625 million yen, final in 2016) and criminal arrests from 2021 for selling edited save data. Hong Kong had the Lik Sang mod-chip cases (2002) and customs raids. Korea punishes private servers under the Game Industry Promotion Act, including a 2024 Supreme Court fine for a private server of a single-player game. No Chinese emulator or game reverse-engineering case was confirmed.

### Cited Findings

**Japan**
- 27 Feb 2009: Tokyo District Court granted Nintendo and 54 software makers an injunction under the UCPA against importers and sellers of "R4 Revolution for DS". — [Capcom press release, 27 Feb 2009](https://www.capcom.co.jp/ir/english/news/pdf/e090227.pdf)
- 5 Oct 2009: a second suit against four importers and sellers, case 2009 (Wa) No. 35335, seeking injunction and damages, because sales continued. — [Capcom press release, 5 Oct 2009](https://www.capcom.co.jp/ir/english/news/pdf/e091005.pdf); [Gigazine](https://gigazine.net/gsc_news/en/20091016_majikon)
- July 2013: Tokyo District Court granted an injunction on import and sale and ordered damages totalling 95,625,000 yen (Nintendo plus 49 software makers). The Supreme Court (Third Petty Bench) refused appeal on 12 Jan 2016, making the judgment final. — [Nintendo release, 9 Jul 2013](https://www.nintendo.co.jp/corporate/release/2013/130709.html); [Game*Spark](https://www.gamespark.jp/article/2013/07/10/41983.html); [ScanNetSecurity, 20 Jan 2016](https://scan.netsecurity.ne.jp/article/2016/01/20/37973.html)
- February 2021: Nagoya police arrested a man (aged 23) for selling generated Pokémon for Sword and Shield made with a modified Switch; reported as the first UCPA arrest for altering Switch game data. — [Nintendo Wire, 5 Feb 2021](https://nintendowire.com/news/2021/02/05/japanese-police-arrest-pokemon-sword-and-shield-hacker-who-sold-generated-monsters-online/); [Automaton](https://automaton-media.com/en/news/hacked-pokemon-seller-arrested-by-japanese-police/)
- July 2021: Niigata police arrested a 27-year-old man for selling modified Breath of the Wild save files; he admitted earning about $90,000 in 18 months. — [Nintendo Wire, 9 Jul 2021](https://nintendowire.com/news/2021/07/09/breath-of-the-wild-hacker-arrested-in-japan-for-selling-modified-save-games/)
- April 2023: a 36-year-old man arrested for modifying Pokémon Scarlet and Violet save data and selling results. — [Automaton](https://automaton-media.com/en/?p=41046)
- October 2025: further Nintendo Switch piracy litigation reported in Japan. — [Automaton JP, 6 Oct 2025](https://automaton-media.com/articles/newsjp/nintendo-switch-piracy-lawsuit-20251006-360534/) (details not confirmed)
- "Magicon" (マジコン) is the Japanese term for flash carts and similar devices; the save-data arrests are not "magicon" cases as such.

**Hong Kong**
- 2002: Nintendo, Sony and Microsoft sued Lik Sang in the High Court; injunctions issued and Lik Sang stopped selling mod chips. Sony Computer Entertainment v Lik Sang International was read broadly in the rights holders' favour under the old s.273. — [The Register, 30 Oct 2002](https://www.theregister.com/2002/10/30/attack_of_the_mod_squads/); [Sydney University Press](https://open.sydneyuniversitypress.com.au/9781920898724/cldc-new-hope-for-consumers-of-digital-copyright-material-in-hong-kong.html)
- September 2008: Customs operation "Intruder" against shops modifying consoles: 29 consoles with circumvention devices seized from seven shops; arrests including three shop owners. — [HK Government press release, 27 Sep 2008](https://www.info.gov.hk/gia/general/200809/27/P200809270122_print.htm)

**South Korea**
- 2024: the Supreme Court upheld a 5 million won fine for a person who ran a private multiplayer server for GTA San Andreas and took donations. The first instance gave a suspended prison sentence; the appeal court cut it to a fine. — [Inven Global](https://www.invenglobal.com/articles/23798/democratic-party-rep-jung-joon-ho-proposes-bill-to-strengthen-penalties-for-private-servers) (search snippet; its account of the lower courts is partly unclear)
- NCSoft obtained a Seoul Central District Court provisional attachment of bank accounts of operators of four Lineage private servers; earlier, police and the Game Rating and Administration Committee arrested six people under the Game Industry Promotion Act. — [Inven Global](https://www.invenglobal.com/articles/20315/ncsoft-pursues-legal-action-including-provisional-attachment-of-accounts-against-operators-of-illegal-lineage-private-servers); [Digital Today](https://www.digitaltoday.co.kr/en/view/40815/ncsoft-seeks-asset-seizure-and-damages-over-lineage-private-servers)
- A bill by Rep. Jung Joon-ho would raise private-server penalties to IP-infringement levels, add punitive damages and an emergency blocking system. Status not known. — [Inven Global](https://www.invenglobal.com/articles/23798/democratic-party-rep-jung-joon-ho-proposes-bill-to-strengthen-penalties-for-private-servers)
- The private-server offence sits in the Game Industry Promotion Act (Art. 32(1), unauthorised provision or operation of game servers). — [KLRI listing](https://elaw.klri.re.kr/kor_service/lawViewTitle.do?hseq=49070) (article text not confirmed)

**Mainland China**
- A Shanghai Pudong court treated game character imagery, skill effects and skill data of unannounced content as trade secrets. — [IAM 2026 trade secrets hub](https://www.iam-media.com/hub/trade-secrets-hub/2026/article/china-trade-secrets-litigation-policy-and-latest-developments)

### Inferences
- Japanese enforcement targets commercial sellers of devices, services and edited data, not individuals who study their own copies. The Korean GTA case shows private servers can be criminal even for single-player games, which matters if a reimplementation adds online play.

### Gaps
- No confirmed Chinese case is known on game cloning by reverse engineering, emulators, or cheat ("外挂") prosecutions.
- No Taiwan game-specific case is known.
- No emulator or ROM case is known in any of the five jurisdictions.

## 7. Game-specific regulation, consumer protection and shutdowns

### Takeaway
China requires a game publishing licence (ISBN) from the National Press and Publication Administration (NPPA) before a game can be released or monetised; a re-release or new platform needs approval or a recorded change. Korea is the only known jurisdiction with consumer rules aimed at games, including refunds and compensation when games close (KFTC standard terms, in force 22 March 2024) and mandatory loot-box odds disclosure.

### Cited Findings
- **China**: any game for official release or monetisation in mainland China (mobile, PC or console) needs an NPPA game publishing ISBN. Foreign firms cannot apply and must partner with a local publisher. Timeframes are not published; approval can take months to over a year. The NPPA publishes monthly lists of domestic, imported and changed approvals. — [NZ MFAT market report](https://www.mfat.govt.nz/cn/trade/mfat-market-reports/entering-chinas-mobile-gaming-market-a-guide-to-licensing-and-regulatory-approval)
- 2025: the NPPA licensed 1,771 games (1,676 domestic, 95 imported); imported approvals normalised at about 9-10 a month from October 2024. — [Niko Partners](https://substack.nikopartners.com/p/chinas-nppa-approved-1771-video-games)
- **Korea**: KFTC revised standard terms for online and mobile games, in force 22 March 2024: in-game purchase refunds for at least 30 days, including for games no longer active, and a route to compensation for discontinued games without suing. Loot-box types and odds must be published (Game Industry Promotion Act, effective March 2024). — [Korea Times, Nov 2024](https://koreatimes.co.kr/www/nation/2024/11/419_387275.html); [Vixio](https://www.vixio.com/insights/gc-south-korean-reforms-impose-loot-box-transparency-refunds)
- KFTC fined Nexon a record 11.64 billion won in 2024 over MapleStory probability items; later fines for Gravity (Ragnarok Online), WEMADE (Night Crows) and KOG (Grand Chase Classic). — [Korea JoongAng Daily](https://www.koreajoongangdaily.com/business/nexon-to-refund-maplestory-game-users-after-item-probability-error/12321081); [Automaton](https://automaton-media.com/en/news/ragnarok-online-dev-fined-by-korean-government-for-manipulating-pull-rates-agrees-to-reimburse-affected-players/)
- **Korea** also requires classification by the Game Rating and Administration Committee before distribution. (not confirmed)
- **Taiwan**: Game Software Rating Management Regulations require age ratings. (not confirmed)

### Inferences
- China's licensing makes a lawful re-release slow and dependent on a local publisher; this pushes preservation in China toward private study. A framework that redistributes nothing avoids publication licensing.
- Korea's shutdown rules protect money spent, not continued access; none of the five gives a right to keep playing a closed online game.

### Gaps
- Japan, China, Taiwan, Hong Kong: no consumer-protection position on online game shutdowns is known.
- Korea: text of the KFTC standard terms and the exact shutdown-compensation rules not confirmed.

## 8. Who does game preservation

### Takeaway
Japan has the most active sector: the National Diet Library (legal deposit), the Game Preservation Society, the Ritsumeikan Center for Game Studies, and the Institute of Game Culture Conservation. Korea has the Nexon Computer Museum in Jeju.

### Cited Findings
- Japan: Game Preservation Society, Ritsumeikan Center for Game Studies and Institute of Game Culture Conservation are named preservation bodies. — [Automaton, July 2022](https://automaton-media.com/en/news/20220714-14016/)
- NDL: legal deposit covers physical game releases since October 2000; over 9,600 titles. — [Automaton](https://automaton-media.com/en/news/nintendo-switch-2-game-key-cards-not-eligible-for-preservation-by-japans-national-diet-library-officials-say/)
- A 2023 Japanese project to rescue rare game artefacts was reported. — [Tech Times, June 2023](https://www.techtimes.com/articles/292489/20230611/japan-launches-rescue-mission-preserve-rare-video-game-artifacts.htm)
- Korea: Nexon Computer Museum, Jeju, opened 2013. (not confirmed)

### Inferences
- Japanese institutions preserve hardware and media; legal reform for playable preservation is still a political proposal (Akamatsu).

### Gaps
- Preservation bodies in China, Taiwan and Hong Kong are not covered.
- Game Preservation Society founding date (believed 2011) and Nexon Computer Museum opening date not confirmed.
