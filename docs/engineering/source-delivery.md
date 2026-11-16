# Corresponding source and rebuild materials

**Release boundary, 2026-11-05:** By owner direction, this collector is for local review and rebuild verification only. Corresponding-source archives are no longer provided as GitHub Release or Actions artifacts. Historical public Release assets and still-downloadable release Actions artifacts containing these archives have been withdrawn, and future release workflows reject their generation or upload. This decision does not change component licenses or establish that corresponding-source, relinking or other obligations are satisfied. GitHub's automatically generated source snapshots for tags in the public repository are outside these Release attachments.

Routine release and acceptance must not run this collector or generate corresponding-source archives locally. Run it only when the user explicitly requests source collection; the upload restriction still applies. License review must identify actual components and obligations rather than treating generic machine-generated review flags as established defects.

scripts/prepare-source-delivery.py collects project source from a clean commit, complete Rust vendor inputs corresponding to Cargo.lock, pinned Qt 6.8.2 module source archives, and rebuild/replacement instructions. All outputs are under build/. A download address is not delivered source, and the collector does not automatically approve an open-source license branch.

```sh
python3 scripts/prepare-source-delivery.py --output build/review/source-delivery \
  --qt-prefix build/toolchain/Qt/6.8.2/macos
python3 scripts/verify-source-delivery.py --delivery build/review/source-delivery \
  --qt-prefix build/toolchain/Qt/6.8.2/macos --output build/review/source-rebuild
```

Qt runtime module archives are qtbase, qtdeclarative, qtsvg and qttranslations; build-required qtshadertools, qttools and pinned Rust standard-library source components are also delivered, from the [official Qt 6.8.2 archive](https://download.qt.io/archive/qt/6.8/6.8.2/submodules/). Official .sha256 values are pinned in scripts/corresponding-sources.json. Cache and downloads are reverified; downloads first go to partial files and are committed after validation. Only archive directories are inspected; third-party paths are not automatically extracted. Cargo vendor uses --locked, retains registry checksums and build dependencies, and does not execute third-party build scripts.

Source delivery also includes the pinned editable Public Suffix List and its MPL-2.0 text. Collection verifies the input, Qt archive and generator identities, then regenerates and compares both the C++ data and binary test data byte for byte. It stops on a mismatch and does not establish full SDK equivalence or approve release licensing.

Source delivery retains the pinned Apache Tika input, upstream NOTICE and OGL-UK-3.0 text for Qt Core MIME data. It verifies the Qt archive attribution and regenerates its MIME data using the pinned archive's conversion script, allowing only native Windows line endings. The complete Tika NOTICE covers the broader upstream distribution and does not imply its other dependencies are shipped by xdzip.

The delivery also contains the CLDR 46 `common` source and upstream license used by Qt 6.8.2 locale generation, not the rest of the CLDR source tree. Collection verifies the CLDR version in two generated Qt source files, the pinned upstream archive hash and selected inputs. Qt's second generation step additionally requires an ISO 639-3 table that is not included; `qt-cldr-sources.json` records the tested candidate's source, version and digest, and its [upstream download page and terms](https://iso639-3.sil.org/code_tables/download_tables) require separate review. Reproducing generated data with this candidate does not establish the precise historical input or the original SDK's complete build identity.

Omitting the original ISO tables does not remove all related identifiers from other CLDR data. CLDR 46 `common/validity/language.xml` retains compact language codes and `common/supplemental/supplementalMetadata.xml` contains aliases. In the [local comparison](https://github.com/xiaodaozhi/xdzip/issues/138#issuecomment-6588994510), a three-letter candidate set constructed from these delivered files covered 7,919 of 7,920 `Id` values in the CLDR 46 candidate ISO table and had 338 extra values. Applying the same method to the CLDR 43.1 subset against that later table covered 7,896 values and had 338 extras. The candidate set is not the original table and does not contain its complete fields; matching identifiers alone does not settle provenance, authorization or applicable terms. Excluding the five ISO-named files does not establish that other related data is absent.

The CLDR 43.1 subset also retains a `language-subtag-registry` tool input dated 2023-02-14 in its header. Of its 8,049 three-letter language subtags, 7,713 match identifiers in the later candidate ISO table above; that cross-version difference is not attributed here. Unicode's update instructions identify this file as the language-subtag registry obtained from IANA. IANA's CC0 statement covers only rights it owns or may sublicense and preserves possible third-party rights. See [#140](https://github.com/xiaodaozhi/xdzip/issues/140#issuecomment-6589292716) for the comparison and limits.

The CLDR 43.1 subset also includes two Noto fonts, four Glyphicons files, Bootstrap and Autosize scripts, and emoji images used by its web tool. `cldr43-webapp-notices/` supplies provenance and licenses for those fonts and scripts and 908 images matching two pinned Noto Emoji versions, after verifying each pinned file. For the other 963 original images whose provenance and redistribution terms have not been independently established, the collector verifies each original hash and substitutes 698 pinned Noto images and 265 pinned Twemoji images. The manifest records both hashes, paths and sources; this does not claim infringement. These files are not part of the xdzip interface. The notices also identify three web-tool source files with IBM copyright notices and treat them under Unicode-DFS-2016 based on that release's root README, preserving copyright notices and license. Another 27 web-root images match a pinned 2026 CLDR commit byte for byte and are treated under that commit's Unicode-3.0 license with the complete notice and individual hashes retained. This assessment does not extend to trademarks, unlisted assets or external dependencies.

Two complete text files in the CLDR 43.1 `docs/rfc` subset, `draft-davis-t-langtag-ext.txt` and `draft-davis-u-langtag-ext-00.txt`, match the IETF's [2011 T extension draft](https://www.ietf.org/archive/id/draft-davis-t-langtag-ext-07.txt) and [2010 U extension draft](https://www.ietf.org/archive/id/draft-davis-u-langtag-ext-00.txt) byte for byte. Collection verifies hashes and retains copyright and notices. Both refer to terms effective on publication; section 3.c of [TLP 4.0](https://trustee.ietf.org/wp-content/uploads/IETF-TLP-4.pdf) permits distribution outside the standards process in full and without modification to the extent the Trust may grant those rights, with required notices preserved. This assessment covers these two texts only, not other HTML, XML, assets or third-party rights.

For languages already in Qt source, extract a limited generator input from original `qlocale_data_p.h`. For example, after extracting QtBase to the current directory, run `python3 scripts/derive-qt-locale-code-input.py --qtbase qtbase-everywhere-src-6.8.2 --output build/qt-locale-codes.tsv`. This covers only existing lookup keys and is neither the complete ISO 639-3 table nor the historical upstream input. Adding languages may still require separately obtained codes and review of their terms. Regeneration of eight files with pinned Qt 6.8.2 source, CLDR 46 data and this derived input was checked allowing only generation-date and line-ending differences. This does not settle ISO table licensing or establish complete corresponding source.

For a specific additional language, extract the delivered CLDR 43.1 subset and use `tools/cldr-code/src/main/resources/org/unicode/cldr/util/data/language-subtag-registry` to check the language subtag and explicitly supply four code fields. The Choctaw example's ISO 639-2 fields can be checked against the [Library of Congress list](https://www.loc.gov/standards/iso639-2/php/code_list.php), and ISO 639-3 against [registration authority data](https://iso639-3.sil.org/code_tables/download_tables). The final empty field indicates no two-letter code:

```sh
python3 scripts/derive-qt-locale-code-input.py --qtbase path/to/qtbase \
  --iana path/to/cldr43/tools/cldr-code/src/main/resources/org/unicode/cldr/util/data/language-subtag-registry \
  --add-language 'cho,cho,cho,' --output build/qt-locale-codes-with-cho.tsv
```

`--add-language` can be repeated. Fields are three-letter code, ISO 639-2/B, ISO 639-2/T and two-letter code; missing fields are empty, and the Part 2 fields must be supplied together or both left empty. The tool checks that at least one three- or two-letter code is a registered IANA language subtag, validates syntax and rejects Qt lookup-key conflicts. It does **not** infer or verify that fields refer to the same language or establish ISO mappings. The recipient must review each mapping and source. Adding a Qt language also requires updating `enumdata.py`, supplying CLDR locale data and regenerating and rebuilding Qt; the row alone does not add a language. This remains a limited derived table, not the complete ISO table or proof of historical input identity or complete corresponding source.

In an isolated test, manually adding a private-use test code to the Qt tool's enum and derived table generated a new enum and code entry. A rebuilt Linux QtCore passed forward and reverse lookup, and an isolated original alpha.8 copy completed offscreen browsing, cancellation and extraction. This added no real language or CLDR locale-format data, does not replace review of real inputs and terms, and does not establish all corresponding-source obligations. See [#138](https://github.com/xiaodaozhi/xdzip/issues/138#issuecomment-6611169790) for steps and the initial probe failure.

Another option is independently obtaining `iso-codes` ISO 639-2 and 639-3 JSON data and converting it to a Qt input. For example, on Linux with the data package installed:

```sh
python3 scripts/convert-iso-codes-for-qt.py \
  --iso-639-2 /usr/share/iso-codes/json/iso_639-2.json \
  --iso-639-3 /usr/share/iso-codes/json/iso_639-3.json \
  --output build/qt-iso-codes.tsv
```

The converter extracts code mappings without names; complete input still produces a complete code set. It rejects invalid codes, duplicates and Qt lookup-key conflicts, skips the ISO 639-2 reserved local-use range and reports input and output SHA-256. See [#138](https://github.com/xiaodaozhi/xdzip/issues/138) for the Qt 6.8.2 comparison with Ubuntu `iso-codes` 4.16.0-1; check other versions separately. The data is not included. Review its stated license, provenance and redistribution terms for generated tables independently; an alternative table does not establish Qt's historical input or complete source compliance.

To reproduce the comparison, obtain the pinned [Ubuntu `iso-codes` 4.16.0-1 package](https://archive.ubuntu.com/ubuntu/pool/main/i/iso-codes/iso-codes_4.16.0-1_all.deb) independently. The following Linux example requires `curl`, `dpkg-deb` and Python 3; it verifies the package hash before local extraction and conversion without changing system installation:

```sh
mkdir -p build && mkdir build/qt-iso-peer &&
curl --fail --location --silent --show-error --connect-timeout 15 --max-time 120 https://archive.ubuntu.com/ubuntu/pool/main/i/iso-codes/iso-codes_4.16.0-1_all.deb -o build/qt-iso-peer/iso-codes.deb &&
python3 -c "import hashlib,pathlib,sys; p=pathlib.Path('build/qt-iso-peer/iso-codes.deb'); h=hashlib.sha256(p.read_bytes()).hexdigest(); sys.exit(0 if h=='cafca0a3a1f07b44fbba53ab7a1f1d9755e6b80cc041173afe2b52e53bed97a7' else 'Package SHA-256 mismatch')" &&
dpkg-deb -x build/qt-iso-peer/iso-codes.deb build/qt-iso-peer/data &&
python3 scripts/convert-iso-codes-for-qt.py \
  --iso-639-2 build/qt-iso-peer/data/usr/share/iso-codes/json/iso_639-2.json \
  --iso-639-3 build/qt-iso-peer/data/usr/share/iso-codes/json/iso_639-3.json \
  --output build/qt-iso-peer/qt-locale-codes.tsv
```

The verified input SHA-256 values are `fa83810fdb59f9d84b4d58486d5e5e48e807d82a98d6a39ef0ba4fc57c2a9327` and `9636ce5266053867627140ce5ada1f9aa897ca07a7501302c1b14b8d1147cdda`; converted output is `85457a36e013e053b70dea19d500ebec2ae6bedf5dad9af0035b973c8db6a60a`. Ubuntu's `usr/share/doc/iso-codes/copyright` declares LGPL-2.1-or-later, but that package-level statement alone does not establish the historical SIL table grant or redistribution rights for a complete code set. Neither the package nor complete converted table is included here; review provenance, terms and subsequent redistribution separately.

Alternatively obtain the [IANA language-subtag registry](https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry) and [Library of Congress ISO 639-2 list](https://loc.gov/standards/iso639-2/ISO-639-2_utf-8.txt) independently and generate another input. The tool extracts codes without names, reports unmatched codes and input/output hashes, and rejects invalid or ambiguous records. The Linux/macOS shell example uses a new directory and verifies both raw files from this comparison before converting:

```sh
(
  set -eu
  mkdir -p build
  mkdir build/qt-iana-loc-inputs
  curl --fail --location --output build/qt-iana-loc-inputs/iana.txt \
    https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry
  curl --fail --location --output build/qt-iana-loc-inputs/loc.txt \
    https://loc.gov/standards/iso639-2/ISO-639-2_utf-8.txt
  python3 -c 'import hashlib,pathlib,sys; p=sys.argv[1:]; sys.exit(0 if all(hashlib.sha256(pathlib.Path(p[i]).read_bytes()).hexdigest()==p[i+1] for i in (0,2)) else "Input SHA-256 mismatch")' \
    build/qt-iana-loc-inputs/iana.txt 755fad43283be7b41ebe3c89ad054b6eaf928f404f9c0edb74799e0eab74beb1 \
    build/qt-iana-loc-inputs/loc.txt 42b71885e4dc885559fda5ad059fd81838cf4782cedb5fd08cc3431f7d067371
  python3 scripts/convert-iana-loc-for-qt.py \
    --iana build/qt-iana-loc-inputs/iana.txt \
    --loc build/qt-iana-loc-inputs/loc.txt \
    --output build/qt-iana-loc-inputs/qt-codes.tsv
)
```

The comparison used an IANA file dated 2026-09-17 with SHA-256 `755fad43283be7b41ebe3c89ad054b6eaf928f404f9c0edb74799e0eab74beb1`. The UTF-8 LoC file obtained directly on 2026-11-03 was 16,010 bytes, SHA-256 `42b71885e4dc885559fda5ad059fd81838cf4782cedb5fd08cc3431f7d067371`. Its 487 records' first three code fields individually matched the earlier web reconstruction. The converter's LF output SHA-256 was `6a75bebfa694258a1a4125fc30512d8ae7994080701377b41306db665ad371c6`. Together these inputs produced 8268 rows matching all 346 distinct existing Qt lookup keys. All eight regenerated files matched after normalizing generation dates and line endings; a negative control modifying one input matched seven. Seven IANA two-letter subtags `bh`, `in`, `iw`, `ji`, `jw`, `mo`, `sh` had no LoC record and the tool reports them. Verify other versions and added languages separately. Upstream addresses may change; investigate a hash mismatch rather than bypassing checks. See [#138](https://github.com/xiaodaozhi/xdzip/issues/138#issuecomment-6588040592).

[IANA/IETF's statement](https://www.iana.org/help/licensing-terms) applies CC0 to protocol-registry rights they hold or can sublicense, preserving possible third-party rights; it does not automatically establish all rights to the original SIL table. Neither the 2026-09-17 IANA snapshot, LoC input nor complete converted comparison table is included here. The CLDR 43.1 subset separately contains the earlier registry described above. Recipients must separately review provenance, applicable terms and subsequent redistribution conditions. This alternative establishes neither Qt's historical input nor complete corresponding-source compliance.

Independent comparison here means obtaining and converting files separately, not independent standards provenance. [RFC 5646 section 2.2.1](https://www.rfc-editor.org/rfc/rfc5646#section-2.2.1) identifies ISO 639-2, 639-3 and 639-5 as standards sources for IANA three-letter language subtags. Matching values do not establish independent authorship or a complete sublicense.

Output includes xdzip-source.tar for the actual pinned commit, rust-vendor.tar.gz, offline Cargo configuration, original Qt archives, REBUILD.md, and per-file SHA-256/size manifest. The manifest is explicitly collected-not-compliance-verified; it does not replace checks of SDK build features/patches, platform runtimes, embedded resources or license compliance. Reconfigured project source may have unknown identity and must be bound to the original commit through the external delivery manifest, without fabricating a clean Git identity.

The separate `rar-wire-mappings/` directory preserves bounded finite-mapping reference provenance, explanations and the exact original UnRAR 7.30 beta1 notice, all included in the hashed delivery. The pinned download filename is `unrarsrc-7.3.1.tar.gz`; its numeric filename does not establish a separate upstream release version. As a conservative notice measure, delivery requires the complete original second license paragraph in the related Rust source comments and stops if missing or changed. The RAR crate conservatively declares `Apache-2.0 AND UnRAR`; the non-OSI-approved `UnRAR` identifier appears in SPDX 3.28 and later and may not be recognized by older tools. These materials include no official engine source and do not relicense reference-derived facts as Apache-2.0. Applicable terms and actual redistribution obligations require candidate-specific review.

The pinned complete ICU 73.2 license, official ICU4C 73.2 release source archive, pinned editable ICU source, CLDR 43.1 source subset and ICU 73.1 bootstrap source are included for Linux ICU and enter the overall manifest. The CLDR subset excludes five ISO 639-3 files, UTS #35 text and associated files under `docs/ldml`, and two documentation screenshots, and substitutes 963 web-tool images. The delivery record retains upstream archive hash and each exclusion and substitution. [Unicode-published UTS #35 version 43.1](https://www.unicode.org/reports/tr35/tr35-69/tr35.html) is separately available under its owner's terms; whether additional transferable permission exists remains to be established. The collector also rejects bootstrap archive files with `iso-639-3` in their names; a filename check does not rule out data in another form. Collection checks archive hashes, versions, embedded licenses and selected source files without extracting third-party paths. The [ICU data rebuild guide](icu-cldr-rebuild.md) records tool builds, generation and editing scope. Matching upstream versions do not prove a supplier built its binaries from these inputs without patches. Current Windows developer packages omit software OpenGL and no longer collect dedicated Mesa/LLVM materials; historical deliveries retain original materials and obligations. See [native runtime sources and notices](native-runtime-materials.md).

The application dynamically links Qt and provides source-rebuild and compatible-library replacement instructions. [macOS deployment verification](macos-deployment.md) covers modified-library loading in an isolated copy without claiming a full Qt source rebuild. Actual Qt source builds and product loading have a separate process in the [automated verification contract](wp12-automation.md). SDK byte-level build equivalence, exact static-link attribution and licensing choices remain part of [release licensing review](https://github.com/xiaodaozhi/xdzip/issues/4), not newly reopened development work on completed WP-12.

verify-source-delivery.py first reverifies delivery file hashes, safely extracts owned project and vendor inputs into a new build/ directory, builds the real app using an empty Cargo cache, CARGO_NET_OFFLINE=true and the existing pinned Qt SDK, then runs startup/shutdown smoke. This verifies offline use of project and Rust inputs, not a Qt module source build or byte-reproducible builds. Generated source directories have unknown Git identity, bound externally by pinned commit and delivery manifest.

Source delivery invokes Cargo from the output directory with a relative `vendor` destination and explicit project manifest. Only currently supported crates.io source mapping and relative `vendor` configuration are accepted. Absolute directories, path escapes or unexpected mappings fail rather than embedding build-machine paths in a relocatable delivery.

For locked vendor packages without conventionally named standalone license or notice files, `rust-vendor-notices/` separately indexes pinned upstream original texts by package. The collector verifies package Cargo manifest hash, upstream Git commit and path, license text SHA-256 and Git blob; unknown gaps or identity mismatches stop delivery. This directory does not modify Cargo vendor files or checksums. Upstream license expressions remain unchanged; this does not approve a specific license choice or earlier release compliance.

The [Qt SDK correspondence check](qt-sdk-correspondence.md) always includes the pinned correspondence manifest and replayable AGL consumer patch. With --qt-prefix, it also includes per-file source declaration checks, build configuration and explicit gaps for that SDK. The delivery manifest recursively hashes patch subdirectories, and rebuild verification checks those bytes again. Without an SDK, no configuration-verification record is fabricated.

Delivery retains a pinned Qt Declarative Git commit object for offline verification of the release commit's relationship to the archive `.tag` tree object; the delivery manifest hashes it. This relationship does not establish complete archive coverage or SDK build equivalence.

See the [automated completion contract](wp12-automation.md) for actual Qt source builds and loading/compatible replacement. Offline application rebuilds using an existing SDK must not be described as rebuilding Qt itself from source.
