use super::*;
use std::io::{self, Read};

#[test]
fn trusted_check_requires_its_own_checksum_and_service_nonzero() {
    let key = test_key();
    let mut check = [0u8; 12];
    check[..8].copy_from_slice(key.check.as_ref());
    let mut hash = SecretSha256::new();
    hash.update(&check[..8]);
    check[8..].copy_from_slice(&hash.finish()[..4]);
    assert_eq!(key.verify_check(&check, false), CheckResult::TrustedMatch);
    check[0] ^= 1;
    assert_eq!(key.verify_check(&check, false), CheckResult::Untrusted);
    let mut hash = SecretSha256::new();
    hash.update(&check[..8]);
    check[8..].copy_from_slice(&hash.finish()[..4]);
    assert_eq!(
        key.verify_check(&check, false),
        CheckResult::TrustedMismatch
    );
    check[..8].fill(0);
    let mut hash = SecretSha256::new();
    hash.update(&check[..8]);
    check[8..].copy_from_slice(&hash.finish()[..4]);
    assert_eq!(key.verify_check(&check, true), CheckResult::Untrusted);
    assert_eq!(
        key.verify_check(&check, false),
        CheckResult::TrustedMismatch
    );
}

#[test]
fn native_password_profiles_bound_and_preserve_surrogate_boundary() {
    use password::Profile;
    let text = "a".repeat(126) + "🙂tail";
    let windows = encode_password(&text, Profile::Windows).unwrap();
    let unix = encode_password(&text, Profile::Unix).unwrap();
    assert_eq!(&windows.as_slice()[126..], &[0xed, 0xa0, 0xbd]);
    assert_eq!(&unix.as_slice()[126..], "🙂".as_bytes());
    assert_eq!(
        encode_password(&"🙂".repeat(127), Profile::Windows)
            .unwrap()
            .as_slice()
            .len(),
        63 * 4 + 3
    );
    assert_eq!(
        encode_password(&"🙂".repeat(127), Profile::Unix)
            .unwrap()
            .as_slice()
            .len(),
        508
    );
    assert_eq!(
        encode_password("", Profile::Windows).unwrap().as_slice(),
        b""
    );
    assert!(matches!(
        encode_password(&"a".repeat(4097), Profile::Unix),
        Err(DecodeError::Limit)
    ));
    assert_eq!(
        encode_password(&"a".repeat(4096), Profile::Unix)
            .unwrap()
            .as_slice(),
        &[b'a'; 127]
    );
    for profile in [Profile::Windows, Profile::Unix] {
        assert_eq!(
            encode_password("密码🙂", profile).unwrap().as_slice(),
            "密码🙂".as_bytes()
        );
    }
}

#[test]
fn native_nul_and_unit_truncation_have_explicit_candidate_and_cache_identity() {
    for profile in [Profile::Windows, Profile::Unix] {
        assert_eq!(
            encode_password("abc\0密码", profile).unwrap().as_slice(),
            b"abc"
        );
        assert_eq!(
            encode_password("\0prefix", profile).unwrap().as_slice(),
            b""
        );
        assert_eq!(encode_password("", profile).unwrap().as_slice(), b"");
        assert!(matches!(
            encode_password(&("x\0".to_owned() + &"y".repeat(4095)), profile),
            Err(DecodeError::Limit)
        ));
        let prefix = "a".repeat(127);
        assert_eq!(
            encode_password(&(prefix.clone() + "one"), profile)
                .unwrap()
                .as_slice(),
            encode_password(&(prefix.clone() + "two"), profile)
                .unwrap()
                .as_slice()
        );
    }
    let prefix = "a".repeat(126);
    assert_eq!(
        encode_password(&(prefix.clone() + "🙂"), Profile::Windows)
            .unwrap()
            .as_slice(),
        encode_password(&(prefix.clone() + "🙃"), Profile::Windows)
            .unwrap()
            .as_slice()
    );
    assert_ne!(
        encode_password(&(prefix.clone() + "🙂"), Profile::Unix)
            .unwrap()
            .as_slice(),
        encode_password(&(prefix + "🙃"), Profile::Unix)
            .unwrap()
            .as_slice()
    );
    let mut cache = KeyCache::default();
    let mut budget = KdfBudget::new(4 * 33, 100_000);
    let control = Cancellation::default();
    let p = KdfParameters {
        salt: [7; 16],
        exponent: 0,
    };
    let first = *cache
        .key(p, "abc\0ignored", &control, &mut budget)
        .unwrap()
        .aes;
    assert_eq!(
        *cache
            .key(p, "abc\0different", &control, &mut budget)
            .unwrap()
            .aes,
        first
    );
    assert_eq!(
        *cache.key(p, "abc", &control, &mut budget).unwrap().aes,
        first
    );
    assert_eq!(budget.rounds(), 33);
    assert_ne!(
        *cache
            .key(p, "abd\0ignored", &control, &mut budget)
            .unwrap()
            .aes,
        first
    );
    assert_eq!(budget.rounds(), 66);
    let empty = *cache.key(p, "", &control, &mut budget).unwrap().aes;
    assert_eq!(
        *cache
            .key(p, "\0ignored", &control, &mut budget)
            .unwrap()
            .aes,
        empty
    );
    assert_eq!(budget.rounds(), 99);
    let full = "a".repeat(127);
    let last = *cache
        .key(p, &(full.clone() + "one"), &control, &mut budget)
        .unwrap()
        .aes;
    assert_eq!(
        *cache
            .key(p, &(full + "two"), &control, &mut budget)
            .unwrap()
            .aes,
        last
    );
    assert_eq!(budget.rounds(), 132);
}

#[test]
fn task_cache_charges_only_misses_and_password_change_erases_old_keys() {
    let control = Cancellation::default();
    let mut cache = KeyCache::default();
    let mut budget = KdfBudget::new(66, 100_000);
    let p = KdfParameters {
        salt: [1; 16],
        exponent: 0,
    };
    let first = *cache.key(p, "password", &control, &mut budget).unwrap().aes;
    assert_eq!(budget.rounds(), 33);
    let bytes = budget.hash_bytes();
    assert_eq!(
        *cache.key(p, "password", &control, &mut budget).unwrap().aes,
        first
    );
    assert_eq!(budget.rounds(), 33);
    assert_eq!(budget.hash_bytes(), bytes);
    assert_ne!(
        *cache.key(p, "changed", &control, &mut budget).unwrap().aes,
        first
    );
    assert_eq!(cache.entries.iter().flatten().count(), 1);
    assert!(matches!(
        cache.key(p, "password", &control, &mut budget),
        Err(DecodeError::Limit)
    ));
    assert_eq!(budget.rounds(), 66);
    assert!(std::mem::size_of::<KeyCache>() + 4096 < KeyCache::MEMORY_BYTES as usize);
}

#[test]
fn kdf_preflight_hash_quota_exponent_and_cancel_do_not_insert_keys() {
    let control = Cancellation::default();
    let p = KdfParameters {
        salt: [0; 16],
        exponent: 0,
    };
    let mut cache = KeyCache::default();
    // N=1 means 33 HMACs, 128-byte pads + 52-byte first + 32*64.
    let mut budget = KdfBudget::new(33, 2227);
    assert!(matches!(
        cache.key(p, "", &control, &mut budget),
        Err(DecodeError::Limit)
    ));
    assert_eq!(budget.rounds(), 0);
    budget = KdfBudget::new(33, 2228);
    cache.key(p, "", &control, &mut budget).unwrap();
    assert_eq!(budget.hash_bytes(), 2228);
    let mut budget = KdfBudget::default();
    assert!(matches!(
        cache.key(
            KdfParameters { exponent: 25, ..p },
            "",
            &control,
            &mut budget
        ),
        Err(DecodeError::Limit)
    ));
    control.cancel();
    assert!(matches!(
        cache.key(p, "", &control, &mut budget),
        Err(DecodeError::Cancelled)
    ));
    assert_eq!(budget.rounds(), 0);
}

fn test_key() -> KeyMaterial {
    KeyMaterial {
        aes: Zeroizing::new(
            unhex("603deb1015ca71be2b73aef0857d77811f352c073b6108d72d9810a30914dff4")
                .try_into()
                .unwrap(),
        ),
        hash: Zeroizing::new([0x53; 32]),
        check: Zeroizing::new([0x37; 8]),
    }
}
const CIPHER: &str = "f58c4c04d6e5f1ba779eabfb5f7bfbd69cfc4e967edb808d679f777bc6702c7d39f23369a9d9bacfa530e26304231461b2eb05e2c39be9fcda6c19078c6a9d1b";
const PLAIN: &str = "6bc1bee22e409f96e93d7e117393172aae2d8a571e03ac9c9eb76fac45af8e5130c81c46a35ce411e5fbc1191a0a52eff69f2445df4f9b17ad2b417be66c3710";
fn iv() -> [u8; 16] {
    std::array::from_fn(|i| i as u8)
}

struct Pieces {
    bytes: Vec<u8>,
    at: usize,
    piece: usize,
}
impl Read for Pieces {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let n = out.len().min(self.piece).min(self.bytes.len() - self.at);
        out[..n].copy_from_slice(&self.bytes[self.at..self.at + n]);
        self.at += n;
        Ok(n)
    }
}
#[test]
fn nist_aes256_cbc_continuous_across_unaligned_physical_pieces() {
    let control = Cancellation::default();
    for piece in [1, 3, 15, 16, 17, 31, 64] {
        let source = Pieces {
            bytes: unhex(CIPHER),
            at: 0,
            piece,
        };
        let mut reader = CbcReader::new(source, 64, &test_key(), iv(), &control).unwrap();
        let mut output = Vec::new();
        reader.read_to_end(&mut output).unwrap();
        reader.finish().unwrap();
        reader.finish().unwrap();
        assert_eq!(hex(&output), PLAIN);
        assert_eq!(reader.consumed_bytes(), 64);
        assert_eq!(reader.decoded_bytes(), 64);
    }
}

#[test]
fn cbc_explicit_nonzero_alignment_tail_exact_range_and_no_pkcs_guess() {
    let control = Cancellation::default();
    let mut bytes = unhex(CIPHER);
    bytes.push(0x99);
    let mut source = std::io::Cursor::new(bytes);
    {
        let mut reader = CbcReader::new(&mut source, 64, &test_key(), iv(), &control).unwrap();
        let mut output = [0; 49];
        reader.read_exact(&mut output).unwrap();
        assert_eq!(reader.remaining_plaintext(), 15);
        reader.finish_padding(15).unwrap();
        assert_eq!(hex(&output), &PLAIN[..98]);
    }
    assert_eq!(source.position(), 64); // trailing source byte belongs to caller
    let bytes = unhex(CIPHER);
    let mut reader = CbcReader::new(bytes.as_slice(), 64, &test_key(), iv(), &control).unwrap();
    let mut output = [0; 48];
    reader.read_exact(&mut output).unwrap();
    assert!(reader.finish_padding(15).is_err());
    assert!(reader.read(&mut output).is_err());
}

#[test]
fn cbc_truncated_nonblock_unread_finish_and_cancel_poison() {
    let control = Cancellation::default();
    assert!(CbcReader::new(&[][..], 15, &test_key(), iv(), &control).is_err());
    let bytes = unhex(CIPHER);
    let mut reader = CbcReader::new(&bytes[..63], 64, &test_key(), iv(), &control).unwrap();
    assert!(reader.read_to_end(&mut Vec::new()).is_err());
    assert!(reader.finish().is_err());
    assert!(reader.read(&mut [0; 1]).is_err());
    let mut reader = CbcReader::new(bytes.as_slice(), 64, &test_key(), iv(), &control).unwrap();
    assert!(reader.finish().is_err());
    assert!(reader.read(&mut [0; 1]).is_err());
    let mut reader = CbcReader::new(bytes.as_slice(), 64, &test_key(), iv(), &control).unwrap();
    reader.read_exact(&mut [0; 1]).unwrap();
    control.cancel();
    let error = reader.read(&mut [0; 1]).unwrap_err();
    assert_eq!(
        error.get_ref().unwrap().downcast_ref::<DecodeError>(),
        Some(&DecodeError::Cancelled)
    );
    assert!(reader.finish().is_err());
}

struct Fault {
    before: usize,
    bytes: Vec<u8>,
    error: DecodeError,
    bare: bool,
}
impl Read for Fault {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.bare {
            self.bare = false;
            return Err(io::ErrorKind::Interrupted.into());
        }
        if self.before == 0 {
            return Err(io::Error::new(io::ErrorKind::Interrupted, self.error));
        }
        let n = out.len().min(self.before).min(self.bytes.len());
        out[..n].copy_from_slice(&self.bytes[..n]);
        self.bytes.drain(..n);
        self.before -= n;
        Ok(n)
    }
}
#[test]
fn cbc_typed_interrupted_first_partial_and_combinators_preserve_and_poison() {
    let control = Cancellation::default();
    for error in [
        DecodeError::SourceIo,
        DecodeError::Cancelled,
        DecodeError::Limit,
    ] {
        for before in [0, 7, 16, 23] {
            for combinator in [false, true] {
                let source = Fault {
                    before,
                    bytes: unhex(CIPHER),
                    error,
                    bare: true,
                };
                let mut reader = CbcReader::new(source, 64, &test_key(), iv(), &control).unwrap();
                let fault = if combinator {
                    reader.read_to_end(&mut Vec::new()).unwrap_err()
                } else {
                    loop {
                        match reader.read(&mut [0; 64]) {
                            Err(e) => break e,
                            Ok(n) => assert!((1..=16).contains(&n)),
                        }
                    }
                };
                assert_ne!(fault.kind(), io::ErrorKind::Interrupted);
                assert_eq!(
                    fault.get_ref().unwrap().downcast_ref::<DecodeError>(),
                    Some(&error)
                );
                assert_eq!(reader.consumed_bytes(), before as u64);
                assert!(reader.read(&mut [0; 1]).is_err());
            }
        }
    }
}

#[test]
fn cbc_source_changed_marker_survives_partial_read_to_end() {
    struct Changed {
        bytes: Vec<u8>,
        at: usize,
    }
    impl Read for Changed {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            if self.at == 23 {
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    xdzip_codec_api::ScanError::SourceChanged,
                ));
            }
            let n = out.len().min(23 - self.at);
            out[..n].copy_from_slice(&self.bytes[self.at..self.at + n]);
            self.at += n;
            Ok(n)
        }
    }
    let control = Cancellation::default();
    let mut reader = CbcReader::new(
        Changed {
            bytes: unhex(CIPHER),
            at: 0,
        },
        64,
        &test_key(),
        iv(),
        &control,
    )
    .unwrap();
    let mut output = Vec::new();
    let error = reader.read_to_end(&mut output).unwrap_err();
    assert_eq!(output.len(), 16);
    assert_eq!(reader.consumed_bytes(), 23);
    assert_eq!(
        error
            .get_ref()
            .unwrap()
            .downcast_ref::<xdzip_codec_api::ScanError>(),
        Some(&xdzip_codec_api::ScanError::SourceChanged)
    );
    assert!(reader.read(&mut [0; 1]).is_err());
}
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
#[test]
fn independent_hashlib_pbkdf2_checkpoints() {
    checkpoint(
        "c7445ee180f8b59fd62b433708bc57cd",
        15,
        "password",
        "d75c7cb7e22d81138d1a15988c785d5141b4fa5c2a691a49505abb7d60aee690",
        "523ac49a98ec2fc579792683b1c0af69490ed5e775933e74ab382f03f278d8b7",
        "ea35671d70f12d4b",
    );
    checkpoint(
        "c7445ee180f8b59fd62b433708bc57cd",
        15,
        "password2",
        "b8dc363cdcda2dcfe7f26dce7ece8dc330d0a367ccd91e36377dfb3862963ae7",
        "3bb7978d3cc573cb37560f3daa21a8f08f94e3c293d0dd08391e9c8871b39c04",
        "834cd448e915ebd6",
    );
    checkpoint(
        "800efa5b4145900c76305088d9cb0df4",
        15,
        "password",
        "e2d7b4c9a62554a5433a5d9d502dbd053007f3613783a46365e8f9c56d98b49d",
        "658b49967aae10da4fb2bdafd31d7b6804c8a6f9a24757abea341d0d9a7e70d2",
        "e20f382e27de7778",
    );
    checkpoint(
        "000102030405060708090a0b0c0d0e0f",
        0,
        "",
        "c6b7413bebb763bda962e5d94e24327e07d4daa9e97c14ea4126ba4b7ccb0d16",
        "33733b599b86375aff1856e5bde19410cd7529f478c0ae44540f1c4382cd9a14",
        "18800c2685e30021",
    );
    checkpoint(
        "000102030405060708090a0b0c0d0e0f",
        1,
        "密码🙂",
        "8c0aeeb0459ec4905d2d6f1187dad085edc19158573e92cc1b7c2ac693e2723c",
        "c5d92db8852a186198ba3a5921565f46876024903d9c3cee7095f979428ac8bc",
        "312a184f92f5d929",
    );
    checkpoint(
        "000102030405060708090a0b0c0d0e0f",
        4,
        "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        "8f3c6da4acaa8ef1e79d53f9afbbf03d77c574c8e15a4a30e0a49b831799407a",
        "6d9406b7683562b45556cff49fbd630de4b59a3912ca8c0fafa32459b5b7c68c",
        "b36efdd626b18c45",
    );
    checkpoint(
        "000102030405060708090a0b0c0d0e0f",
        2,
        "yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy",
        "ec84bafe7dcb9bf2d3dadfe31d5c3922828278cb2681fbb98d4e6fdcb8123944",
        "f826a3437ea239740770b7f41414dfe5397a0f5806a59e86f2caed75da50da34",
        "12ff254d04bafd7d",
    );
    checkpoint(
        "000102030405060708090a0b0c0d0e0f",
        0,
        "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
        "4d2863143da3e6ea1094393266a58ce894cf3a3dd3a2b533dc8bf8212290760c",
        "b0da49ed7ae88d72aab5b2555aa583efef6c783070d8aa9c91881ab2c9ecaa2a",
        "53c3ea1c247dd8fa",
    );
}
fn checkpoint(salt: &str, exponent: u8, password: &str, aes: &str, hash: &str, check: &str) {
    let parameters = KdfParameters {
        salt: unhex(salt).try_into().unwrap(),
        exponent,
    };
    let mut budget = KdfBudget::default();
    let control = Cancellation::default();
    let encoded = encode_password(password, Profile::native()).unwrap();
    let key = derive(parameters, &encoded, &control, &mut budget).unwrap();
    assert_eq!(hex(key.aes.as_ref()), aes);
    assert_eq!(hex(key.hash.as_ref()), hash);
    assert_eq!(hex(key.check.as_ref()), check);
    assert_eq!(budget.rounds(), (1 << exponent) + 32);
}

#[test]
fn keyed_crc_and_blake_tweaks_match_independent_hmac() {
    let key = test_key();
    assert_eq!(key.tweak_crc(0xfedcba98), 0xf90a090f);
    assert_eq!(
        hex(&key.tweak_blake(std::array::from_fn(|i| i as u8))),
        "8363d31abfb79557eb66dbb6511a95e487eeb410722415da1cc23e1009b77863"
    );
}
#[test]
fn sixteen_slot_cache_evicts_without_reusing_old_quota_or_unbounded_growth() {
    let mut cache = KeyCache::default();
    let control = Cancellation::default();
    let mut budget = KdfBudget::new(18 * 33, 1_000_000);
    for i in 0..17 {
        cache
            .key(
                KdfParameters {
                    salt: [i; 16],
                    exponent: 0,
                },
                "pw",
                &control,
                &mut budget,
            )
            .unwrap();
    }
    assert_eq!(cache.entries.iter().flatten().count(), 16);
    cache
        .key(
            KdfParameters {
                salt: [0; 16],
                exponent: 0,
            },
            "pw",
            &control,
            &mut budget,
        )
        .unwrap();
    assert_eq!(budget.rounds(), 18 * 33);
    assert!(matches!(
        cache.key(
            KdfParameters {
                salt: [99; 16],
                exponent: 0
            },
            "pw",
            &control,
            &mut budget
        ),
        Err(DecodeError::Limit)
    ));
}

#[test]
fn fixed_rar_ciphertexts_match_independent_dotnet_aes_plain_hashes() {
    let control = Cancellation::default();
    let mut budget = KdfBudget::default();
    let mut cache = KeyCache::default();
    {
        let data = unhex(
            "63ba54b2d010a5495fb4bad3c069a83b1bbcbb3827d7879378732f7c93e2d228c400defea6c03b7e6c67d4055000dcf8",
        );
        let p = KdfParameters {
            salt: unhex("c7445ee180f8b59fd62b433708bc57cd")
                .try_into()
                .unwrap(),
            exponent: 15,
        };
        let key = cache.key(p, "password", &control, &mut budget).unwrap();
        let iv = unhex("fa34df06ab5d05d49307770f1d8e8053")
            .try_into()
            .unwrap();
        let mut reader = CbcReader::new(data.as_slice(), 48, key, iv, &control).unwrap();
        let mut plain = Vec::new();
        reader.read_to_end(&mut plain).unwrap();
        reader.finish().unwrap();
        let mut hash = SecretSha256::new();
        hash.update(&plain);
        assert_eq!(
            hex(hash.finish().as_ref()),
            "746ba3c5864a7c755bd57eead08d845d5ae325a376955dcc16ad5d699896f348"
        );
    }
    {
        let data = unhex("db6972f4f6568ea6fd1965e995b336853437f52aa0d32787b2f5c10232d8aa5e");
        let p = KdfParameters {
            salt: unhex("c7445ee180f8b59fd62b433708bc57cd")
                .try_into()
                .unwrap(),
            exponent: 15,
        };
        let key = cache.key(p, "password2", &control, &mut budget).unwrap();
        let iv = unhex("f3cdd6b31cb45ba77cac36588eb30e69")
            .try_into()
            .unwrap();
        let mut reader = CbcReader::new(data.as_slice(), 32, key, iv, &control).unwrap();
        let mut plain = Vec::new();
        reader.read_to_end(&mut plain).unwrap();
        reader.finish().unwrap();
        let mut hash = SecretSha256::new();
        hash.update(&plain);
        assert_eq!(
            hex(hash.finish().as_ref()),
            "7fb5676348716d6f3fdf748ae43cfc148f5778c0e9996c3c9c4ffe89bd780d3d"
        );
    }
    {
        let data = unhex(
            "35a594adf9ea9fd8db1f4a0012b1049383f0dfa3c8deadf9e79342925ef01ef3d13c214f914b3ea19129e95a921f82ec",
        );
        let p = KdfParameters {
            salt: unhex("800efa5b4145900c76305088d9cb0df4")
                .try_into()
                .unwrap(),
            exponent: 15,
        };
        let key = cache.key(p, "password", &control, &mut budget).unwrap();
        let iv = unhex("08ed5475a7d30bbe8fb4f1a776f1502f")
            .try_into()
            .unwrap();
        let mut reader = CbcReader::new(data.as_slice(), 48, key, iv, &control).unwrap();
        let mut plain = Vec::new();
        reader.read_to_end(&mut plain).unwrap();
        reader.finish().unwrap();
        let mut hash = SecretSha256::new();
        hash.update(&plain);
        assert_eq!(
            hex(hash.finish().as_ref()),
            "815f752a15413bb8e9a6c8a44e838d6c3d891391ed5da56b62840eea4cb487ae"
        );
    }
    {
        let data = unhex("23338eb2515032e0a355646e13e472c9");
        let p = KdfParameters {
            salt: unhex("800efa5b4145900c76305088d9cb0df4")
                .try_into()
                .unwrap(),
            exponent: 15,
        };
        let key = cache.key(p, "password", &control, &mut budget).unwrap();
        let iv = unhex("191e76f5b7454150b6d8016ded74aedf")
            .try_into()
            .unwrap();
        let mut reader = CbcReader::new(data.as_slice(), 16, key, iv, &control).unwrap();
        let mut plain = Vec::new();
        reader.read_to_end(&mut plain).unwrap();
        reader.finish().unwrap();
        let mut hash = SecretSha256::new();
        hash.update(&plain);
        assert_eq!(
            hex(hash.finish().as_ref()),
            "e4d571a1a5edf8bd247b0d86554cb935235788fb4ec7336142ea62967c37939f"
        );
    }
    {
        let data = unhex("8a285fa8739b7d716c45284640559d02");
        let p = KdfParameters {
            salt: unhex("800efa5b4145900c76305088d9cb0df4")
                .try_into()
                .unwrap(),
            exponent: 15,
        };
        let key = cache.key(p, "password", &control, &mut budget).unwrap();
        let iv = unhex("5b64dc44d98c46940240d6b23d5762d6")
            .try_into()
            .unwrap();
        let mut reader = CbcReader::new(data.as_slice(), 16, key, iv, &control).unwrap();
        let mut plain = Vec::new();
        reader.read_to_end(&mut plain).unwrap();
        reader.finish().unwrap();
        let mut hash = SecretSha256::new();
        hash.update(&plain);
        assert_eq!(
            hex(hash.finish().as_ref()),
            "f0b01d7e4fdfc4b3254dc2065303ccd7852e2bc0915c3cb650445a85ca634881"
        );
    }
    {
        let data = unhex("39e07044c4b2bcf98e35e4e0ead903b0");
        let p = KdfParameters {
            salt: unhex("800efa5b4145900c76305088d9cb0df4")
                .try_into()
                .unwrap(),
            exponent: 15,
        };
        let key = cache.key(p, "password", &control, &mut budget).unwrap();
        let iv = unhex("69736882f53f3b76a436dfd596a0846b")
            .try_into()
            .unwrap();
        let mut reader = CbcReader::new(data.as_slice(), 16, key, iv, &control).unwrap();
        let mut plain = Vec::new();
        reader.read_to_end(&mut plain).unwrap();
        reader.finish().unwrap();
        let mut hash = SecretSha256::new();
        hash.update(&plain);
        assert_eq!(
            hex(hash.finish().as_ref()),
            "3bb14b48a51ca262bcdc4c172998ea5846e099cb3653474aa047c83445b93a1b"
        );
    }
}
