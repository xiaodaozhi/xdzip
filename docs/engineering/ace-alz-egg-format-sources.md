# ACE, ALZ and EGG source and licensing gates

This record pins the public material used to evaluate the remaining M8 formats. The files remain in `build/toolchain/references` and are not product dependencies.

## ACE

- Technical note: Marcel Lemke, *Technical information of the archiver ACE v1.2*, retrieved from `wdlkmpx/unace`, SHA-256 `F645C91348D5DCD560A19CA21DEF849D25C304EFFAACF65B03EFBB0A893E51F9`.
- Independent implementation: `droe/acefile`, commit `11ed076ae19e8228148731cfbc05f019a395b3b9`, BSD-2-Clause implementation license SHA-256 `AD376BD9695F0E355F5C523443ABF822BB896999A3252AAA614D313CC4F1F5B5`, implementation SHA-256 `BC701569AD8521964C62256472AB44976AC57FB46E52B147A21018BC96348F1F`.
- The implementation states that it reads ACE 1.0 and 2.0, including LZ77, DELTA, EXE, PIC, SOUND, passwords and volumes. Its license covers the Python implementation only and expressly does not claim rights over ACE technology.

The technical record is enough to define block headers, CRCs, stored members and the old LZ77 family. It is not a commercial clearance for the ACE technology, and the implementation's reference to WinAce and unace is not an independent interoperability corpus. The product gate therefore remains: obtain a legal decision on a clean-room decoder, pin a redistributable corpus, then admit a declared read-only subset. No ACE suffix or decoder is registered yet.

## ALZ

- Format specification: `alkegi/docs`, commit `36230c34d3056604cc63519e9855a79e894496b7`, CC0 license SHA-256 `F4E7F373B9B996950337E8D41A4A2939C2D90B7725E9BAF3D5084A22717AD328`.
- `alz.md` SHA-256 `CBD35E77AB53CFD08C0FF192FA2793438150E4935522608936B1A0942560E237`.
- The document specifies the ALZ and local-file signatures, variable-width sizes, CRC-32, UTF-8/CP949 names, Store, raw RFC 1951 Deflate and an ALZ-modified BZip2 stream. It also specifies PKWARE traditional encryption and split-volume layout.

ALZ is admitted in xdzip as a conservative read-only product slice: ordinary single-volume, unencrypted Store and raw Deflate with exact CRC/size/termination. The codec, runtime, CLI and association paths are covered by automatic tests. Modified BZip2, ZipCrypto and split volumes require independent complete fixtures and separate resource/security tests. A CC0 document does not by itself prove that every implementation detail is free of third-party patent or trademark obligations.

## EGG

- Public format document: `alkegi/docs`, same commit `36230c34d3056604cc63519e9855a79e894496b7`, CC0 license above.
- `egg.md` SHA-256 `4CA2CB6E8FA9A4F40C30EDE05A5E977BE113A96E1527E28A7B1ED0836D8A43A3`.
- Reference module: `dterracino/UnEgg`, commit `f21e653b02f7ab467bbc16af879e68a48dabcb2b`, EGG specification PDF SHA-256 `0BB7E6F174CAFB9F6726D70B7DEE8DBF33C034717058B1B0EB2B372D9A8FBB37`.
- The UnEGG source license SHA-256 values are `9B012928756D9243CC549FA0A7D7B5742ED8F69B00A2BE1C964F42EF21F1D8D6` for the source package and `30F2E45148D43D7A77574403807C13AA359EE9ACE841A00B6A5348AD961423F0` for the module package.

The UnEGG license says commercial use requires ESTsoft approval, redistribution is non-commercial, and the module cannot be used to develop an EGG compressor. It also states that ESTsoft retains all rights. The module is therefore test-only evidence and cannot be copied, linked, or treated as a product dependency.

The line format is technically separable from the restricted module. A future product slice may independently implement non-encrypted, non-solid Store, raw Deflate, standard BZip2 and the documented LZMA framing after a license and patent review. AZO, AES/LEA, ZipCrypto, split and solid archives remain outside the current product boundary. No EGG suffix or decoder is registered until that gate is cleared.

## Required implementation gates

1. Keep these references outside the product tree and verify the pinned hashes before use.
2. Build independent complete samples for every admitted method. A header-only or Store-only sample cannot establish method interoperability.
3. Add container, runtime, CLI and desktop entries together, with unknown methods visible but without payload authority.
4. Keep passwords, volumes, solid state and special-object semantics disabled until their own source, security and cancellation tests pass.
5. Record the legal decision and sample identities in the issue before changing the public capability registry.
