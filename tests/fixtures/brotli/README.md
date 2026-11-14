# Authored Brotli inputs

Case construction and plain test messages are project-authored. Dictionary cases derive payloads from the licensed [RFC 7932 normative word data](../../../third_party/brotli-rfc7932/README.md); its BSD notice remains applicable. Reference implementation source or binaries are not included. Fixtures are input data, never test results or product capability declarations.

The independent author reference is [Google Brotli v1.2.1](https://github.com/google/brotli/releases/tag/v1.2.1), MIT, Python wheel `brotli-1.2.1-cp312-cp312-win_amd64.whl`, SHA-256 `f5342b8256476554b4045db8b7bf4dc4044d0405485f220f0cd80764bc0d7faf`. Keep it under `build/toolchain/references/brotli-1.2.1`; every extracted file is compared with the pinned wheel before import. Only its public Python encoder/decoder API is used. A compatible Python runs `scripts/generate-brotli-fixtures.py --reference build/toolchain/references/brotli-1.2.1 --rfc build/toolchain/references/brotli-rfc/rfc7932.txt`; `--check` compares without rewriting fixtures. Other-platform references require separately verified identities.

The generator uses both the public reference compressor and independently authored wire construction. All full decoded bytes are compared by the reference decoder. It also writes fixed size/CRC expectations used by Rust regression tests. CRC here identifies known fixture output; ordinary Brotli streams do not carry this checksum. Evidence manifests stay in `build/review/format-expansion/`.

Cases cover qualities 0–11 in generic/text/font modes, window declarations 10–24, empty streams, all byte values, structured text/UTF-8, signed integers, long repeated data and explicitly flushed meta-blocks retaining history. Four authored context cases select different singleton literal trees from all four context modes, beginning after a raw-byte prelude. Their context maps use zero-run and inverse move-to-front coding. The block-switch case alternates literal, command and distance block types independently, with count-one blocks and actual different literal output.

`dictionary-all.br` constructs all 121 transforms for the first and last dictionary words of every length 4–24, including zero-length transforms and non-ASCII byte fermentation, and validates the exact complete output independently. Twelve distance cases cover postfix values 0–3 and direct fields 0/1/15. Raw and metadata cases exercise byte alignment, ignored metadata and a final metadata-only block. Corrupt/truncated input and precise memory/input/output/work/cancellation boundaries are constructed in Rust tests. Unknown shared dictionaries, RFC 9841 large-window profiles and concatenation are separate from these ordinary RFC 7932 cases.

Product handoff additionally covers owned TAR payloads, a damaged TAR header that must not become ordinary Brotli, empty-TAR/ordinary zero-only ambiguity, and a raw compressed stream whose bytes contain a TAR marker at physical offset 257 while its actual decoded payload has no TAR header.

| Input | Packed bytes | Decoded bytes | SHA-256 |
| --- | ---: | ---: | --- |
| `q0-generic.br` | 88 | 144 | `fabb8647d089e8f25df61b802d3380a627d32b07734c1bf982a5b36184efdde4` |
| `q0-text.br` | 88 | 144 | `fabb8647d089e8f25df61b802d3380a627d32b07734c1bf982a5b36184efdde4` |
| `q0-font.br` | 88 | 144 | `fabb8647d089e8f25df61b802d3380a627d32b07734c1bf982a5b36184efdde4` |
| `q1-generic.br` | 44 | 144 | `8a2feb90267028928b59950f65f465b723735ae1f98759c9d8f512fa2510318d` |
| `q1-text.br` | 44 | 144 | `8a2feb90267028928b59950f65f465b723735ae1f98759c9d8f512fa2510318d` |
| `q1-font.br` | 44 | 144 | `8a2feb90267028928b59950f65f465b723735ae1f98759c9d8f512fa2510318d` |
| `q2-generic.br` | 42 | 144 | `6c490d9d44fe5a24ec33fd2013afeca4ee557c93296c272aa57ea867e2e683af` |
| `q2-text.br` | 42 | 144 | `6c490d9d44fe5a24ec33fd2013afeca4ee557c93296c272aa57ea867e2e683af` |
| `q2-font.br` | 42 | 144 | `6c490d9d44fe5a24ec33fd2013afeca4ee557c93296c272aa57ea867e2e683af` |
| `q3-generic.br` | 25 | 144 | `e0fcf160c54992d86f6db2773fd817fec7b3985ab784396c356d01158872ec87` |
| `q3-text.br` | 25 | 144 | `e0fcf160c54992d86f6db2773fd817fec7b3985ab784396c356d01158872ec87` |
| `q3-font.br` | 25 | 144 | `e0fcf160c54992d86f6db2773fd817fec7b3985ab784396c356d01158872ec87` |
| `q4-generic.br` | 25 | 144 | `d24ff3f732d0e1e210c0e8ece2912a570b95c027c47a3f3629a96ccafe5bf38a` |
| `q4-text.br` | 25 | 144 | `d24ff3f732d0e1e210c0e8ece2912a570b95c027c47a3f3629a96ccafe5bf38a` |
| `q4-font.br` | 25 | 144 | `0ae8aaeaccaee5d29d60b0e6dea34888fafd35d521dd95e709120a2b89a95eaf` |
| `q5-generic.br` | 25 | 144 | `89d5a392a2ef09048f8e64348165696c5f15ae302b706e4adde7a60c9d8c2340` |
| `q5-text.br` | 25 | 144 | `89d5a392a2ef09048f8e64348165696c5f15ae302b706e4adde7a60c9d8c2340` |
| `q5-font.br` | 25 | 144 | `1052f56f6ba3fd11ec666aec75af8a3f3df8678e9332b402429de365b7f5fedc` |
| `q6-generic.br` | 25 | 144 | `89d5a392a2ef09048f8e64348165696c5f15ae302b706e4adde7a60c9d8c2340` |
| `q6-text.br` | 25 | 144 | `89d5a392a2ef09048f8e64348165696c5f15ae302b706e4adde7a60c9d8c2340` |
| `q6-font.br` | 25 | 144 | `1052f56f6ba3fd11ec666aec75af8a3f3df8678e9332b402429de365b7f5fedc` |
| `q7-generic.br` | 25 | 144 | `89d5a392a2ef09048f8e64348165696c5f15ae302b706e4adde7a60c9d8c2340` |
| `q7-text.br` | 25 | 144 | `89d5a392a2ef09048f8e64348165696c5f15ae302b706e4adde7a60c9d8c2340` |
| `q7-font.br` | 25 | 144 | `1052f56f6ba3fd11ec666aec75af8a3f3df8678e9332b402429de365b7f5fedc` |
| `q8-generic.br` | 25 | 144 | `89d5a392a2ef09048f8e64348165696c5f15ae302b706e4adde7a60c9d8c2340` |
| `q8-text.br` | 25 | 144 | `89d5a392a2ef09048f8e64348165696c5f15ae302b706e4adde7a60c9d8c2340` |
| `q8-font.br` | 25 | 144 | `1052f56f6ba3fd11ec666aec75af8a3f3df8678e9332b402429de365b7f5fedc` |
| `q9-generic.br` | 25 | 144 | `89d5a392a2ef09048f8e64348165696c5f15ae302b706e4adde7a60c9d8c2340` |
| `q9-text.br` | 25 | 144 | `89d5a392a2ef09048f8e64348165696c5f15ae302b706e4adde7a60c9d8c2340` |
| `q9-font.br` | 25 | 144 | `1052f56f6ba3fd11ec666aec75af8a3f3df8678e9332b402429de365b7f5fedc` |
| `q10-generic.br` | 28 | 144 | `9b1ccfce8d475ef66198d282bea5e5eee1487fcc5853f85c925a87efdc940a29` |
| `q10-text.br` | 28 | 144 | `9b1ccfce8d475ef66198d282bea5e5eee1487fcc5853f85c925a87efdc940a29` |
| `q10-font.br` | 28 | 144 | `9b1ccfce8d475ef66198d282bea5e5eee1487fcc5853f85c925a87efdc940a29` |
| `q11-generic.br` | 28 | 144 | `9b1ccfce8d475ef66198d282bea5e5eee1487fcc5853f85c925a87efdc940a29` |
| `q11-text.br` | 28 | 144 | `9b1ccfce8d475ef66198d282bea5e5eee1487fcc5853f85c925a87efdc940a29` |
| `q11-font.br` | 28 | 144 | `9b1ccfce8d475ef66198d282bea5e5eee1487fcc5853f85c925a87efdc940a29` |
| `window-10.br` | 26 | 144 | `f7b173be799f3150f0e00f838dc6a282bfdf7d501aa5d4d2a8ae15992ddb5e39` |
| `window-11.br` | 26 | 144 | `dbe88840b0dfd47a516026c272e44e6847167e909cf78add3486f67a582118cf` |
| `window-12.br` | 26 | 144 | `a1a7e8080e902064c15be4a9a46235d1d1254fab5c993be9649373d0c73fd30b` |
| `window-13.br` | 26 | 144 | `2059564683f1bfca45b4aaf799b756415b066ae2463873f8de61aa9879cfa59d` |
| `window-14.br` | 26 | 144 | `01a5da17a3d41627ddc28b02a8a5096bf134127a551b55d696d8bd23706281fc` |
| `window-15.br` | 26 | 144 | `4d0e1022bb2a8848881388944df5cabb12c927cc105d3355f22797c98286bee1` |
| `window-16.br` | 25 | 144 | `89d5a392a2ef09048f8e64348165696c5f15ae302b706e4adde7a60c9d8c2340` |
| `window-17.br` | 26 | 144 | `9064c941a64c26ac580d5e1d3ddb9c3ea0b0692b20c459db91772776805d937f` |
| `window-18.br` | 25 | 144 | `ee14c2093906a293084b6a7cb1e6c5ec4c54c63bb6ff76132099fae309d032bc` |
| `window-19.br` | 25 | 144 | `6fa5567bfcde566047f9d4c4391e64662648adbeeb2d7f37dd4fd00a5ce548c1` |
| `window-20.br` | 25 | 144 | `e9e6949b59b755fd7a3dff3dfc5154cf5b2acc4293a00175b2c099f4bbacaea3` |
| `window-21.br` | 25 | 144 | `2acbce38da9c261f8ac3bad448504ce687751650afff7b31f3ea12ffbdb8bceb` |
| `window-22.br` | 25 | 144 | `97fb7483f7964651538e1dbeb5bfafd5a50c639bdbe9d22901f0755cfcac2918` |
| `window-23.br` | 25 | 144 | `53b71aebbc5f7cdce7b41651456b37f082e0f096f69c7629daece12b6a3e15f6` |
| `window-24.br` | 25 | 144 | `7f5ab4763b0cb35b5316c494715f84235621f9d7ad99dde8a3301aa1966e94e3` |
| `empty.br` | 1 | 0 | `41b805ea7ac014e23556e98bb374702a08344268f92489a02f0880849394a1e4` |
| `all-bytes.br` | 212 | 256 | `319dbe4f50dc775347c9c9421376525a2215fa453b40f12b433bb0a3956cea9b` |
| `varied.br` | 320 | 113600 | `9e425c3193de3abe832ae45fddba89da7afcb8a2c3c2c0cc113f81f21528e91b` |
| `repeated.br` | 32 | 1152000 | `f7f0f44eb60c2a8b772b44f5ab767f8ceaeeebf79b73572464512f5d48dab7ab` |
| `integers.br` | 119051 | 262144 | `c715e368f81d09945007002d878f3ac54f8222c41725d9473d10122108110ac6` |
| `meta-blocks.br` | 654 | 334400 | `0201ea52e56e87b43c47cf4d1a4ae0566604d87ee03f9e75500717b039fd73ff` |
| `context-0.br` | 54 | 258 | `a49b8c8cf07ac39ccd6c7cbbaad5032e0a1ec2614498ab630971709b1fddea2b` |
| `context-1.br` | 54 | 258 | `11c75a087287b5157090b65e7fb3329b3acbcbf0546f54d348412b9e8bf0aec9` |
| `context-2.br` | 54 | 258 | `2f9f3863f13f3c21adbbfbeb4c693970147074b8a8c1b576e9fa1db92cb2be09` |
| `context-3.br` | 54 | 258 | `523f95aa5226a974eb64d1c029c209563a7be54394562154f00b1b09e9f762e7` |
| `block-switches.br` | 61 | 97 | `3bc811973fa301b7739976dc2bbc04fa50382ade15d4494ff2185c65f9f33cec` |
| `dictionary-all.br` | 19704 | 84834 | `6ff21d9bc54c7e8e0c56424e116dd8b1d50ccf34fa1e913b2c5851d37f4bceb1` |
| `distance-0-0.br` | 1415 | 5353 | `212d2d75e742d826262abc0933c4f30ec2a45422c3e9e80e70226762553d38ac` |
| `distance-0-1.br` | 1421 | 5353 | `dfc56c24db0ebb65b8b1ddcde77ce54f1152d1d5b5983518fed0c895f7c275c1` |
| `distance-0-15.br` | 1468 | 5353 | `b91f1365d3e5db3cc36b1525e3495d24b50e31385d39f8550d664a94b6d22d32` |
| `distance-1-0.br` | 1427 | 5353 | `ab7e621c63f2b6d065b26742ba1cc82c31d7dda81411e349803bc18305489e5d` |
| `distance-1-1.br` | 1427 | 5353 | `cc4dab2fb9c0cef3686fe8435121dcf71fcd07de319693989e27b26330fb4217` |
| `distance-1-15.br` | 1431 | 5353 | `61ee5c63435c5e583de37ca89fbbab079425e911d287e0da76a1c854cf525f4a` |
| `distance-2-0.br` | 1439 | 5353 | `abcdef9873055d60ff5ae73e9f0b6deb29adacfcf66410cc164e52bd2d72c7f8` |
| `distance-2-1.br` | 1440 | 5353 | `2a3c48b862cb7edaa71ebad3b409128abf80e5e2cb60e873bebc374802672441` |
| `distance-2-15.br` | 1447 | 5353 | `ca12e93ff15ad3769bf7d3d4d7a7f697add3c29c8f9f97b6d9aebd650a4fab8d` |
| `distance-3-0.br` | 1464 | 5353 | `3e0d10e1bebb319502aad6bc7a1d67622bb42abf08eb166b5b4fa1ea99f43461` |
| `distance-3-1.br` | 1465 | 5353 | `faad05f25df34c1aca98f28f0df2b718076e2988c3c662f75653a0631332a66d` |
| `distance-3-15.br` | 1478 | 5353 | `65e572f374cd8979a22cb2a00c689beb02ab479fe846fc7dea4988dd9f3410c2` |
| `metadata.br` | 171 | 144 | `3d7418829c583827870ba27b835bc454f2840766a55e2648a1a25768807ef2ea` |
| `empty-meta.br` | 5 | 0 | `21e7203727291bedea9f05750e67fbc50bb4d38c30db65c6c1e905022577af74` |
| `raw.br` | 5 | 1 | `71ecce5657d97080bddd4427c49111bd604f3bcd4a13dbe61de23bc325dbb3db` |
| `files.tar.br` | 85 | 10240 | `b04d135345a6daf94928abf6d6af7d2a84da525235a04816b37476fdbafef870` |
| `bad-header.br` | 88 | 10240 | `f0f5919194eea28c4abe49f15c3e5dd393dde4c64e805474863d3d8e0e3a4928` |
| `empty.tar.br` | 10 | 1024 | `e01985b5b1b4cf8a077a82cf19c10365b692ea93e64718b7eb86d3abdc72049f` |
| `empty-tar.br` | 10 | 1024 | `e01985b5b1b4cf8a077a82cf19c10365b692ea93e64718b7eb86d3abdc72049f` |
| `zeros.br` | 12 | 4096 | `b6644147adc01dce08648c6b6207112c728a44fcdaf0d263f603c626fa6a2dd1` |
| `raw-tar-marker.br` | 516 | 512 | `b6ba12bc7e844624b427494f505f388fd11f93820bba2bbd4c659a32ac9592e3` |
