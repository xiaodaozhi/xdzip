# Owned AES Crypt inputs

All plaintexts, test credentials and harness code are project-owned and redistributable under the repository license. The phrase is `abracadabra abracadabra\n` repeated six times. Tail cases use sequential bytes, all-bytes uses 0–255, and multi uses 131077 bytes with `(i * 73 ^ (i >> 8)) & 255`. Empty files are explicit. Unicode and long password profiles exercise UTF-16LE ACKDF and UTF-8 PBKDF2. Public deterministic test secrets and IVs are fixture data, not production secrets.

The independent encoder/reader is [aescry 1.0.0](https://github.com/danielpclark/aescry), `.crate` SHA-256 `5edea8662cc04eb5fa53ce716df3de4bb28237ff0a2e951dfdc2795c851be7dd`, under MIT OR Apache-2.0. `reference.rs` is owned API harness code. No reference implementation source or binary is included in product or fixtures. Reference crypto/framing implementation code was not used as product implementation text. The separate Python decoder uses Python hashlib/hmac and the public cryptography cipher API, independently checking the full output, both MACs where present and version-specific padding/length. The Windows reference verification used cryptography 50.0.2.

Run `scripts/generate-aescrypt-fixtures.py --reference build/toolchain/references/aescry-1.0.0` using a compatible Python with cryptography; add `--check` to compare without rewriting inputs. The pinned reference crate and extracted regular-file bytes are verified before building an isolated, locked, offline API harness. Keep its source, dependency cache and binary under `build/toolchain/`. Evidence JSON stays under `build/review/format-expansion/`. Setup on other platforms requires separately verified references; these inputs do not establish native platform acceptance.

Versions 0–3 cover ordinary, empty, 1/15/16/17-byte payloads and Unicode credentials. Versions 2–3 add long credentials and unauthenticated extensions including an unsafe FILENAME, opaque URI and empty container, none of which grants authority. Version 3 additionally covers iteration 1, the encoder's default 600000 rounds, all byte values and a payload crossing the 64 KiB work batch. Other v3 fixtures use 2 rounds for bounded ordinary regressions; this is not a security recommendation. Corruption, iteration/header limits, cancellation and legacy unauthenticated modulo changes are constructed by tests. The read-only product contract and legacy authentication limitations are in [AES Crypt envelopes](../../../docs/engineering/aescrypt-envelopes.md); acceptance ID is **FORMAT-AESCRYPT**.

| Input | Packed bytes | Decoded bytes | SHA-256 |
| --- | ---: | ---: | --- |
| `v0-phrase.aes` | 197 | 144 | `3b22609de7dca77ab0f0aee3618cad41a09fa7c32ff6aef3d07152a3bf593969` |
| `v0-empty.aes` | 53 | 0 | `0dccd86b1fe14aa69b18571e47e84387e30c26936de211e5f69ff32ce077a4d8` |
| `v0-tail1.aes` | 69 | 1 | `714ca1479f84535304beb3a5e1e728bff3ff2e02eec60e71b158794512e09249` |
| `v0-tail15.aes` | 69 | 15 | `92f4e4536773b890fba079568209d9570213dbe2f588ecafb764009f4152050b` |
| `v0-block16.aes` | 69 | 16 | `77f1e29ebc658a1602b952a28f9006d5fac0a3c134d2904e81544499225142c9` |
| `v0-tail17.aes` | 85 | 17 | `fd64c05d380ca1b61cf160ad0e455d26e46148947b7cc608df2550cde1766e63` |
| `v0-unicode.aes` | 197 | 144 | `6bb1c93f90974ec752be74e999f6cf5ab36a767993ec2d55012059543d679425` |
| `v1-phrase.aes` | 278 | 144 | `60b04daec7b4fe7d4a8018598bee812b42fae704b1f69fac784776541c5f56d0` |
| `v1-empty.aes` | 134 | 0 | `9d0722adaa243acd02eb76727d77c8dca60608b48f4d7b5c6063f37ea0e08771` |
| `v1-tail1.aes` | 150 | 1 | `21b6e8ea000ea258a18a86b16009bf42a1e81044fafd842d90403d2aaeb38ffc` |
| `v1-tail15.aes` | 150 | 15 | `f3ff66c0db160b37fd6cca2978a5cef4b963cac6bded5811e67a3f98a74da394` |
| `v1-block16.aes` | 150 | 16 | `b68429cfe3ab6c630a80eb405a1d9cca4946eb63e0ceb9061952902add916d8c` |
| `v1-tail17.aes` | 166 | 17 | `1a2b5636477794b3fa4ab32c4f4816dddf0fd4afa6772aebc25af1ba6cad4a4d` |
| `v1-unicode.aes` | 278 | 144 | `a5ba02a20a0842bf0bd2f37e57ec9c0e1f763eec7a07af39bcebeb8c4ee5d107` |
| `v2-phrase.aes` | 280 | 144 | `390d1a5344bc6b259309aa4ea36497e3b80f961cd4598375075da55706825574` |
| `v2-empty.aes` | 136 | 0 | `ba606bb6e22c3b061e9a1aa62e027565fc30cca8598aee14f25a9147645f9cf3` |
| `v2-tail1.aes` | 152 | 1 | `7f8a15f6054b8b78e43f98715b48f4ca7d4bb31c7f2226dc999a1e543d57011c` |
| `v2-tail15.aes` | 152 | 15 | `a6dac5dad0a924d04ced3721e3e0936ce703e4be51764bc958925878b8b6155d` |
| `v2-block16.aes` | 152 | 16 | `de4b3dc5db1e90f2168c103369ddc2736fb1736ee47a6fa74053c352670c295e` |
| `v2-tail17.aes` | 168 | 17 | `3e79ebdfb1f72f7dc3f43fff6d313ec6206d108cd8ea70e2af9f5194ae7db364` |
| `v2-unicode.aes` | 280 | 144 | `9d54e5f1c3228bca4af4c0b55f40b7da2e525a3807915b6e908f66d808f6d935` |
| `v3-phrase.aes` | 299 | 144 | `8acf91782b9b91385883361f7f32bb6adbec005c3d8ed72aac3644b0c5ed6552` |
| `v3-empty.aes` | 155 | 0 | `747f8979361bdb64c69012b8255bba68db4c89b7ad37d1034d551284adaec8a1` |
| `v3-tail1.aes` | 155 | 1 | `ae23f11d7de13e66af3911b8ecfa4f8a68525ca42c0878400dbba6fd68fd8d02` |
| `v3-tail15.aes` | 155 | 15 | `a029f14dbb4734d8d015c5e9af0606d56f4b2c724c3888a344dc9a7c76e4693b` |
| `v3-block16.aes` | 171 | 16 | `46db5ba9e5c0dd5182c014feb85d58e631926523c356b15b3ebc87c12deffe9c` |
| `v3-tail17.aes` | 171 | 17 | `3abf7df506287ed02b3dd6169de2417289b308c9273aa2bda5f1b60f9729eb45` |
| `v3-unicode.aes` | 299 | 144 | `3a2ea4c2e1950f86047306fcfc771f749c55ca36f26a92ed729e67a974d9eadd` |
| `v2-long.aes` | 280 | 144 | `ce903dfa2a4965972ea20866dbd7b1a2df7dcc439e4721d887d8938ba178cbe3` |
| `v2-extensions.aes` | 493 | 144 | `72a7ebd9074ace814112ed99309d04b3402125eab96c2cfdb24f8ca78a8affaf` |
| `v3-long.aes` | 299 | 144 | `260a0559d30c160cf91012b813a234b0906e66f48d7ba3817267442a5ca560d0` |
| `v3-extensions.aes` | 512 | 144 | `30b1d189ce5c604d0c4ba00bda3bd7337c2a6c7300870a4fdbbc60ebb0a3155f` |
| `v3-default.aes` | 299 | 144 | `076243f0502607e819f3d202be7f8163b0efabeb3517c8362d24e0b1e0be419e` |
| `v3-iter1.aes` | 171 | 17 | `69c121a711272bb3a778b9788793d762af33b1477d1cea5e33c0ddb65e0749ef` |
| `v3-all-bytes.aes` | 411 | 256 | `40dc7ab2f52f58041c94cd212644a9b2102265502fc300df7fd944bbaf7ff1a4` |
| `v3-multi.aes` | 131227 | 131077 | `b73ba201cad6a6a3d7f3c06a1f6a7a45eba60a602d5b489e21869b544038cc97` |
