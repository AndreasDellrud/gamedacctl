---
title: Legal and publication considerations
type: synthesis
status: mixed
updated: 2026-09-05
sources:
  - docs/research-process.md
  - docs/protocol.md
---

# Legal and publication considerations

## Status and scope

This page records research findings and risk controls for a Sweden-based personal or open-source interoperability project. It is not legal advice and does not establish how a court would decide a particular dispute. Obtain advice from a Swedish intellectual-property lawyer before commercialization, paid support, accepting investment, or responding to a legal demand.

The present method observes USB messages produced during ordinary use of lawfully installed SteelSeries GG with an owned GameDAC. It does not copy GG source code, decompile object code, bypass authentication or encryption, modify firmware, defeat firmware signing, access a cloud service, or redistribute SteelSeries software.

## Copyright and interoperability

[Swedish Copyright Act (1960:729) §26g](https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/lag-1960729-om-upphovsratt-till-litterara-och_sfs-1960-729/) permits a person entitled to use a computer program to observe, investigate, or test its functioning to determine underlying ideas and principles while performing authorized loading, display, execution, transmission, or storage. The same section says contractual terms restricting that observation right are invalid.

Section 26h permits reproduction or translation of code when necessary to obtain otherwise not readily available information required for interoperability with another program. The acts must be performed by or for a lawful user, limited to necessary portions, and the information may not be used beyond interoperability, unnecessarily disclosed, or used to create a program with substantially similar expression. Contractual terms restricting §26h are invalid.

[Directive 2009/24/EC](https://eur-lex.europa.eu/legal-content/en/TXT/?uri=CELEX:32009L0024) provides the corresponding EU framework. Article 1(2) excludes ideas and principles underlying interfaces from program copyright protection; Article 5(3) protects observation, study, and testing by a lawful user; Article 6 provides the interoperability exception for necessary decompilation.

In [SAS Institute v World Programming, C-406/10](https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX%3A62010CA0406), the Court of Justice of the European Union held that program functionality, programming languages, and data-file formats used to exploit functions are not themselves protected forms of program expression. A USB lighting protocol is not identical to that case's facts, but the distinction between functionality/interface information and copied expression is relevant.

The current packet-observation method fits more naturally under observation and testing than decompilation. Future work should not decompile GG unless it is genuinely necessary for interoperability, the information is not readily available, and the narrower §26h conditions have been reviewed.

## Trade-secret considerations

[Directive (EU) 2016/943, Article 3](https://eur-lex.europa.eu/eli/dir/2016/943/oj/eng) treats independent discovery and observation, study, disassembly, or testing of a publicly available or lawfully possessed product as lawful acquisition when the acquirer is free from a legally valid duty limiting acquisition. It also recognizes acquisition, use, or disclosure allowed by EU or national law.

The Swedish government's [preparatory work for the 2018 Trade Secrets Act](https://www.riksdagen.se/sv/dokument-och-lagar/dokument/proposition/en-ny-lag-om-foretagshemligheter_h503200/html/) states that a person who obtains a trade secret by reverse engineering a lawfully marketed product may freely use or disclose it. Contractual duties and the precise facts can still matter.

This project acquired information from an ordinary retail product and normal device traffic, without employment, NDA, leaked material, unauthorized account access, or intrusion into SteelSeries systems. That materially lowers trade-secret risk.

## Contract concerns

SteelSeries publishes [Game On terms](https://steelseries.com/game-on/terms-and-conditions) containing broad restrictions on reverse engineering the promotional site or software used for it. The page's scope is not clearly the same as a current GG desktop end-user license, so it should neither be ignored nor treated as conclusively controlling this project.

Swedish §§26g and 26h expressly invalidate contract terms that restrict their covered rights. That does not answer every possible contract, governing-law, remedy, or jurisdiction question. Preserve a copy or version reference for any GG terms actually accepted before public release, and obtain counsel if the project becomes commercial.

## Other legal and practical concerns

### Trademark and passing off

Use “SteelSeries,” “Arctis Pro,” and “GameDAC” only as necessary to describe compatibility. Choose an original application name and artwork, avoid SteelSeries logos and trade dress, and state prominently that the project is independent and not affiliated with or endorsed by SteelSeries.

### Copied expression and assets

Do not copy GG source or object code, UI artwork, icons, screenshots as application assets, sounds, documentation prose, firmware, fonts, or installers. Implement the documented behavior independently. Release binaries should generate supported packets rather than bundle GG components.

### Raw captures

Filtered captures are valuable provenance but need not ship with the application. Before publishing a capture, verify that it contains only the target device address and no credentials, serial numbers, unrelated USB traffic, proprietary executables, or firmware. Prefer publishing the derived specification, source code, and deterministic fixtures needed for interoperability.

### Patents

Copyright permission does not determine patent liability. No patent search has been performed. Ordinary RGB configuration appears low risk, but this is an unverified assumption rather than a legal conclusion.

### Warranty, safety, and support

Sending observed lighting reports is not a firmware modification, but unsupported software can still malfunction or be blamed for device problems. Limit supported devices precisely, reject unknown firmware/device identities where necessary, avoid firmware commands, provide static rollback, and include a no-warranty/use-at-own-risk notice consistent with the chosen open-source license.

## Practical risk assessment

| Activity | Working risk assessment | Controls |
| --- | --- | --- |
| Private observation and replay on owned hardware | Low | Normal GG operation, filtered captures, known commands only. |
| Publishing protocol notes and independent source | Low to moderate | Separate facts from inference; publish no proprietary code or assets. |
| Publishing a free Linux controller under original branding | Low to moderate | Compatibility-only marks, disclaimer, safe device scope, independent implementation. |
| Publishing unfiltered captures or bundled GG/firmware files | High and unnecessary | Prohibited by project policy. |
| Firmware probing, circumvention, or cloud/API access | High and outside scope | Do not implement. |
| Commercial product, paid support, or vendor-facing claims | Moderate and fact-dependent | Obtain Swedish/EU legal review before launch. |

“Low” does not mean SteelSeries cannot object or send a demand. It means the current independently written interoperability implementation and Swedish/EU authorities provide a materially stronger position than copying, circumvention, or misleading branding would.

## Publication safeguards

A public release should have an original name, original visual design, open-source license, compatibility matrix, independent-project disclaimer, no proprietary dependencies, no firmware functionality, precise experimental warnings, security-reviewed udev access, and a documented process for correcting protocol claims. Legal correspondence should be preserved and reviewed rather than answered casually in an issue tracker.
