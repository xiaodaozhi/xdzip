use super::*;
use std::io::{self, Read};
use xdzip_crypto::SecretSha256;

#[test]
fn legacy_explicit_narrow_projection_never_guesses_nonascii_encoding() {
    assert!(matches!(
        NarrowPassword::ascii("密码"),
        Err(DecodeError::Unsupported)
    ));
    assert!(matches!(
        NarrowPassword::ascii(&"a".repeat(4097)),
        Err(DecodeError::Limit)
    ));
    let first = NarrowPassword::from_projected(&[0x81, 0x40, 0, 0x99]).unwrap();
    let second = NarrowPassword::from_projected(&[0x81, 0x40]).unwrap();
    let mut a = LegacyCipher::old13(&first);
    let mut b = LegacyCipher::old13(&second);
    for byte in 0..=255 {
        let mut x = [byte];
        let mut y = [byte];
        a.apply(&mut x);
        b.apply(&mut y);
        assert_eq!(x, y);
    }
    let a = NarrowPassword::from_projected(&[7; 200]).unwrap();
    let b = NarrowPassword::from_projected(&[7; 127]).unwrap();
    let mut a = LegacyCipher::old15(&a);
    let mut b = LegacyCipher::old15(&b);
    let mut x = [3];
    let mut y = [3];
    a.apply(&mut x);
    b.apply(&mut y);
    assert_eq!(x, y);
}

#[test]
fn legacy_wide_projection_preserves_platform_difference_and_bounds() {
    let text = "a".repeat(126) + "🙂x";
    let win = wide_password(&text, Profile::Windows).unwrap();
    let unix = wide_password(&text, Profile::Unix).unwrap();
    assert_eq!(&win.as_slice()[252..], &[0x3d, 0xd8]);
    assert_eq!(&unix.as_slice()[252..], &[0x42, 0xf6]);
    assert_eq!(win.len, 254);
    assert_eq!(unix.len, 254);
    assert!(matches!(
        wide_password(&"a".repeat(4097), Profile::Windows),
        Err(DecodeError::Limit)
    ));
    assert!(std::mem::size_of::<LegacyAesCache>() + 4096 < LegacyAesCache::MEMORY_BYTES as usize);
}

#[test]
fn legacy_nul_prefix_and_127_units_are_projected_before_key_identity() {
    for profile in [Profile::Windows, Profile::Unix] {
        assert_eq!(
            wide_password("abc\0密码", profile).unwrap().as_slice(),
            &[b'a', 0, b'b', 0, b'c', 0]
        );
        assert_eq!(wide_password("\0ignored", profile).unwrap().as_slice(), &[]);
        assert!(matches!(
            wide_password(&("a\0".to_owned() + &"b".repeat(4095)), profile),
            Err(DecodeError::Limit)
        ));
        let full = "a".repeat(127);
        assert_eq!(
            wide_password(&(full.clone() + "one"), profile)
                .unwrap()
                .as_slice(),
            wide_password(&(full + "two"), profile).unwrap().as_slice()
        );
    }
    let p1 = NarrowPassword::ascii("abc\0密码").unwrap();
    let p2 = NarrowPassword::ascii("abc").unwrap();
    let mut a = LegacyCipher::old13(&p1);
    let mut b = LegacyCipher::old13(&p2);
    let mut x = [3];
    let mut y = [3];
    a.apply(&mut x);
    b.apply(&mut y);
    assert_eq!(x, y);
    assert!(NarrowPassword::ascii(&("a".repeat(127) + "密码")).is_ok());
}

#[test]
fn legacy_aes128_nist_cbc_fragmented_nonzero_tail_and_exact_range() {
    let key = LegacyAesKey {
        key: Zeroizing::new(
            unhex("2b7e151628aed2a6abf7158809cf4f3c")
                .try_into()
                .unwrap(),
        ),
        iv: Zeroizing::new(std::array::from_fn(|i| i as u8)),
    };
    let data = unhex(
        "7649abac8119b246cee98e9b12e9197d5086cb9b507219ee95db113a917678b273bed6b8e3c1743b7116e69e222295163ff1caa1681fac09120eca307586e1a7",
    );
    let plain = "6bc1bee22e409f96e93d7e117393172aae2d8a571e03ac9c9eb76fac45af8e5130c81c46a35ce411e5fbc1191a0a52eff69f2445df4f9b17ad2b417be66c3710";
    let control = Cancellation::default();
    let mut source = std::io::Cursor::new(data);
    let mut reader =
        LegacyReader::new(&mut source, 64, LegacyCipher::aes128(&key), &control).unwrap();
    let mut out = [0; 49];
    reader.read_exact(&mut out).unwrap();
    assert_eq!(hex(&out), &plain[..98]);
    reader.finish_padding(15).unwrap();
    assert_eq!(reader.consumed_bytes(), 64);
    assert_eq!(reader.decoded_bytes(), 64);
    drop(reader);
    assert_eq!(source.position(), 64);
}

#[test]
fn legacy_kdf_cache_distinguishes_no_salt_and_zero_salt_and_charges_only_misses() {
    let mut cache = LegacyAesCache::default();
    let control = Cancellation::default();
    let mut budget = LegacyKdfBudget::new(2 * ROUNDS, 1 << 30);
    let first = *cache
        .key("password", None, &control, &mut budget)
        .unwrap()
        .key;
    assert_eq!(budget.work_left, ROUNDS);
    assert_eq!(
        *cache
            .key_projected_units(
                &"password".encode_utf16().collect::<Vec<_>>(),
                None,
                &control,
                &mut budget
            )
            .unwrap()
            .key,
        first
    );
    assert_eq!(budget.work_left, ROUNDS);
    assert!(matches!(
        cache.key_projected_units(&[1; 128], None, &control, &mut budget),
        Err(DecodeError::Limit)
    ));
    assert_eq!(
        *cache
            .key("password", None, &control, &mut budget)
            .unwrap()
            .key,
        first
    );
    assert_eq!(budget.work_left, ROUNDS);
    assert_ne!(
        *cache
            .key("password", Some([0; 8]), &control, &mut budget)
            .unwrap()
            .key,
        first
    );
    assert_eq!(budget.work_left, 0);
    assert!(matches!(
        cache.key("password", Some([1; 8]), &control, &mut budget),
        Err(DecodeError::Limit)
    ));
    assert_eq!(cache.entries.iter().flatten().count(), 2);
    control.cancel();
    assert!(matches!(
        cache.key("password", None, &control, &mut budget),
        Err(DecodeError::Cancelled)
    ));
}

#[test]
fn old20_preflight_and_block_state_are_bounded() {
    let password = NarrowPassword::ascii("password").unwrap();
    let control = Cancellation::default();
    let mut budget = LegacyKdfBudget::new(0, 1000);
    assert!(matches!(
        LegacyCipher::old20(&password, &control, &mut budget),
        Err(DecodeError::Limit)
    ));
    assert_eq!(budget.bytes_left, 1000);
    let mut budget = LegacyKdfBudget::default();
    let mut a = LegacyCipher::old20(&password, &control, &mut budget).unwrap();
    let mut block = [0x73; 16];
    a.apply(&mut block);
    let first = block;
    a.apply(&mut block);
    assert_ne!(block, first);
    assert!(
        std::mem::size_of::<LegacyCipher>() < LegacyCipher::MEMORY_BYTES as usize,
        "actual={}",
        std::mem::size_of::<LegacyCipher>()
    );
    control.cancel();
    assert!(matches!(
        LegacyCipher::old20(&password, &control, &mut budget),
        Err(DecodeError::Cancelled)
    ));
}

#[test]
fn old20_cached_preparation_is_immutable_and_not_free_work_after_password_change() {
    let password = NarrowPassword::ascii("password").unwrap();
    let control = Cancellation::default();
    let mut budget = LegacyKdfBudget::default();
    let mut cache = Old20Cache::default();
    let mut first = cache.cipher(&password, &control, &mut budget).unwrap();
    let after = budget.work_left;
    let mut first_block = [3; 16];
    first.apply(&mut first_block);
    first.apply(&mut [7; 16]);
    let mut next = cache.cipher(&password, &control, &mut budget).unwrap();
    let mut next_block = [3; 16];
    next.apply(&mut next_block);
    assert_eq!(next_block, first_block);
    assert_eq!(budget.work_left, after);
    assert!(std::mem::size_of::<Old20Cache>() < Old20Cache::MEMORY_BYTES as usize);
    let changed = NarrowPassword::ascii("changed").unwrap();
    let mut denied = LegacyKdfBudget::new(0, 0);
    assert!(matches!(
        cache.cipher(&changed, &control, &mut denied),
        Err(DecodeError::Limit)
    ));
    assert!(matches!(
        cache.cipher(&password, &control, &mut denied),
        Err(DecodeError::Limit)
    ));
}

#[test]
fn legacy_reader_typed_fault_and_truncation_poison_without_suppressing_error() {
    struct Fault {
        at: usize,
        first: bool,
    }
    impl Read for Fault {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            if self.first {
                self.first = false;
                return Err(io::ErrorKind::Interrupted.into());
            }
            if self.at == 23 {
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    xdzip_codec_api::ScanError::SourceChanged,
                ));
            }
            let n = out.len().min(23 - self.at);
            out[..n].fill(7);
            self.at += n;
            Ok(n)
        }
    }
    let control = Cancellation::default();
    let mut budget = LegacyKdfBudget::default();
    let p = NarrowPassword::ascii("pw").unwrap();
    for cipher in [
        LegacyCipher::old13(&p),
        LegacyCipher::old15(&p),
        LegacyCipher::old20(&p, &control, &mut budget).unwrap(),
    ] {
        let mut reader =
            LegacyReader::new(Fault { at: 0, first: true }, 32, cipher, &control).unwrap();
        let mut out = Vec::new();
        let error = reader.read_to_end(&mut out).unwrap_err();
        assert_eq!(
            error
                .get_ref()
                .unwrap()
                .downcast_ref::<xdzip_codec_api::ScanError>(),
            Some(&xdzip_codec_api::ScanError::SourceChanged)
        );
        assert_eq!(reader.consumed_bytes(), 23);
        assert!(reader.finish().is_err());
        assert!(reader.read(&mut [0; 1]).is_err());
    }
    let mut reader = LegacyReader::new(&[][..], 1, LegacyCipher::old13(&p), &control).unwrap();
    assert!(reader.read_to_end(&mut Vec::new()).is_err());
    assert!(reader.finish().is_err());
    let mut reader = LegacyReader::new(&[3][..], 1, LegacyCipher::old13(&p), &control).unwrap();
    assert!(reader.finish_padding(15).is_err());
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|b| format!("{b:02x}")).collect()
}
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
#[test]
fn sha1_independent_hashlib_padding_boundaries() {
    check_sha1(0, "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    check_sha1(1, "5d1be7e9dda1ee8896be5b7e34a85ee16452a7b4");
    check_sha1(55, "c93ad8d6ddf6ee1c6a592f80b50c4692d972502a");
    check_sha1(56, "6180c8ee549fcbe4fd3fb2784f0e4e20aa548a24");
    check_sha1(63, "7edf186ac2d6e83b4a86657445de4a3e7437165b");
    check_sha1(64, "8bbc356346031e2ba16efea5beacea9b4ccd199f");
    check_sha1(65, "921e20c47ba9a9aa64b659f0aaf6b5d85739bebd");
    check_sha1(119, "2538aa936a5385a2e9c7eb4a67f5aeb5b820a7ae");
    check_sha1(120, "33299a031d84bd91b3101eb0a12c36d332fe75e8");
    check_sha1(127, "1cc06b0fc22268349d586924c5a4a7ed91cf3c49");
    check_sha1(128, "312ebe5f72f4f88104ba52fd5b81fd32e5c0cd4f");
    check_sha1(513, "cbcecf7ef9923afa2585ad940c3d3ad58337193f");
    check_sha1(65537, "3659dba3001115dd2b600dae7432600eb3f9fc6f");
}
fn check_sha1(n: usize, expected: &str) {
    let bytes: Vec<u8> = (0..n)
        .map(|i| i.wrapping_mul(29).wrapping_add(7) as u8)
        .collect();
    for size in [1, 7, 64, 8192] {
        let mut hash = sha1::Sha1::new();
        for p in bytes.chunks(size) {
            hash.update(p);
        }
        let copy = hash.clone();
        assert_eq!(hex(hash.finish().as_ref()), expected);
        assert_eq!(hex(copy.finish().as_ref()), expected);
    }
}
#[test]
fn fixed_rar3_aes_keys_match_independent_hashlib() {
    check_rar3(
        "password",
        Some(unhex("be021b9a792a8a55").try_into().unwrap()),
        "a52c5531cd3f94c7cc75332f91aadcfc",
        "b45e05c74948ed33f28b0ad47ad43937",
    );
    check_rar3(
        "password2",
        Some(unhex("afe27d13f3a55352").try_into().unwrap()),
        "5fc142b01298ec8755f76bffc7409cba",
        "17cbe2674c7cbda5a67be07c571b1fdc",
    );
    check_rar3(
        "password",
        Some([0; 8]),
        "1d355f475be0f8cf465153f6d2703f64",
        "330575d664babc8a4ff6c90440e5b85d",
    );
}
fn check_rar3(password: &str, salt: Option<[u8; 8]>, key: &str, iv: &str) {
    let control = Cancellation::default();
    let mut budget = LegacyKdfBudget::default();
    let password = wide_password(password, Profile::native()).unwrap();
    let actual = derive(&password, salt, &control, &mut budget).unwrap();
    assert_eq!(hex(actual.key.as_ref()), key);
    assert_eq!(hex(actual.iv.as_ref()), iv);
}
#[test]
fn old_byte_profiles_follow_independent_mathematical_vectors() {
    byte_case(
        "",
        "0724415e7b98b5d2ef0c294663809dbad7f4112e4b6885a2bfdcf91633506d",
        false,
    );
    byte_case(
        "",
        "2550a179d09b5bb7d92b79af7728f7d6b37240ef155fce7b6b5e73c5c5001b",
        true,
    );
    byte_case(
        "a",
        "83bb31e5d70775210b33993d1f3f9d39132b8115e7f745d19ba3e96d2f2f6d",
        false,
    );
    byte_case(
        "a",
        "720be41a32c5a940b8b1140b102468f15c1b9e9f629b437898fa7779d1cf8c",
        true,
    );
    byte_case(
        "password",
        "38bc030dda6abdd3ac48a7c9ae56c1efe0940b45420285cbd4a02f81966e09",
        false,
    );
    byte_case(
        "password",
        "44f8deae551e685a871a36e68e20cea47edd8674e85db31a36e16633601c06",
        true,
    );
    byte_case(
        "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        "07ac51f69b40e58a2fd4791ec3680db257fca146eb9035da7f24c96e13b85d",
        false,
    );
    byte_case(
        "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        "9207b62b3349373725494c74ae1b830ef1281c6d260fcf4ec2e01ab48ca03c",
        true,
    );
}
fn byte_case(password: &str, expected: &str, fifteen: bool) {
    let p = NarrowPassword::ascii(password).unwrap();
    let control = Cancellation::default();
    for chunk in [1, 3, 16, 8192] {
        let cipher = if fifteen {
            LegacyCipher::old15(&p)
        } else {
            LegacyCipher::old13(&p)
        };
        let source: Vec<u8> = (0usize..31)
            .map(|i| i.wrapping_mul(29).wrapping_add(7) as u8)
            .collect();
        let mut reader = LegacyReader::new(source.as_slice(), 31, cipher, &control).unwrap();
        let mut out = Vec::new();
        let mut buffer = vec![0; chunk];
        loop {
            let n = reader.read(&mut buffer).unwrap();
            if n == 0 {
                break;
            }
            out.extend_from_slice(&buffer[..n]);
        }
        reader.finish().unwrap();
        assert_eq!(hex(&out), expected);
        assert_eq!(reader.consumed_bytes(), 31);
        assert_eq!(reader.decoded_bytes(), 31);
    }
}

#[test]
fn fixed_rar3_ciphertexts_match_independent_dotnet_plain_hashes() {
    let control = Cancellation::default();
    let mut cache = LegacyAesCache::default();
    let mut budget = LegacyKdfBudget::default();
    {
        let bytes = unhex("c9f9f60d107fa8d1d67cebd4a4b6c8d46b828a1e9faeb00df8dab6c7440aeae4");
        let key = cache
            .key(
                "password",
                Some(unhex("be021b9a792a8a55").try_into().unwrap()),
                &control,
                &mut budget,
            )
            .unwrap();
        let mut reader =
            LegacyReader::new(bytes.as_slice(), 32, LegacyCipher::aes128(key), &control).unwrap();
        let mut plain = Vec::new();
        reader.read_to_end(&mut plain).unwrap();
        reader.finish().unwrap();
        let mut hash = SecretSha256::new();
        hash.update(&plain);
        assert_eq!(
            hex(hash.finish().as_ref()),
            "43060a484fa73fe1ceda816dd916b3e16d8b7f8a53b7d2fbfbeff50ef9055969"
        );
    }
    {
        let bytes = unhex("76f83fc947681b502d368cc078c9d3a945aaf68903dcbe18e08c6657db04f5e9");
        let key = cache
            .key(
                "password2",
                Some(unhex("afe27d13f3a55352").try_into().unwrap()),
                &control,
                &mut budget,
            )
            .unwrap();
        let mut reader =
            LegacyReader::new(bytes.as_slice(), 32, LegacyCipher::aes128(key), &control).unwrap();
        let mut plain = Vec::new();
        reader.read_to_end(&mut plain).unwrap();
        reader.finish().unwrap();
        let mut hash = SecretSha256::new();
        hash.update(&plain);
        assert_eq!(
            hex(hash.finish().as_ref()),
            "96e1cfd5d59ac0b189cf928e6bf500dafdb5694abce036b2ddac6ddb79662603"
        );
    }
}

#[test]
fn fixed_rar3_encrypted_headers_match_crc_and_true_physical_span() {
    let control = Cancellation::default();
    let mut cache = LegacyAesCache::default();
    let mut budget = LegacyKdfBudget::default();
    let key = cache
        .key("password", Some([0; 8]), &control, &mut budget)
        .unwrap();
    {
        let bytes = unhex(
            "740b86332830b58740325e636d1ce2f68a56d1c2951ef62faaf4776dc113949ddc4320e5253af82b3f6e7da7f0a53d1825a2c437e05b4e13625c7c63c50d7335",
        );
        let mut reader =
            LegacyReader::new(bytes.as_slice(), 64, LegacyCipher::aes128(key), &control).unwrap();
        let mut plain = vec![0; 50];
        reader.read_exact(&mut plain).unwrap();
        reader.finish_padding(15).unwrap();
        assert_eq!(
            (xdzip_compression::crc32(&plain[2..]) & 65535) as u16,
            u16::from_le_bytes(plain[..2].try_into().unwrap())
        );
        assert_eq!(plain[2], 116);
        assert_eq!(u16::from_le_bytes(plain[5..7].try_into().unwrap()), 50);
        assert_eq!(&plain[32..37], b"a.txt");
    }
    {
        let bytes = unhex(
            "222308f6f57b591ba3b47f3167b19c95591699601fde900334e23da9fdfe8d9075355bfe49d1f02d5ac8964edaa54d1d40fea8db1ee6628c22e9c2f784e2b455",
        );
        let mut reader =
            LegacyReader::new(bytes.as_slice(), 64, LegacyCipher::aes128(key), &control).unwrap();
        let mut plain = vec![0; 50];
        reader.read_exact(&mut plain).unwrap();
        reader.finish_padding(15).unwrap();
        assert_eq!(
            (xdzip_compression::crc32(&plain[2..]) & 65535) as u16,
            u16::from_le_bytes(plain[..2].try_into().unwrap())
        );
        assert_eq!(plain[2], 116);
        assert_eq!(u16::from_le_bytes(plain[5..7].try_into().unwrap()), 50);
        assert_eq!(&plain[32..37], b"b.txt");
    }
    {
        let bytes = unhex(
            "ca890109465a62efbbd963d338424d1ea00b8b18a3aebb0ac1f51e2a0c8866ff499642695306c40e4742dda5c1f8efa41a9924f5ffb5b33a96babb13bc828ebc",
        );
        let mut reader =
            LegacyReader::new(bytes.as_slice(), 64, LegacyCipher::aes128(key), &control).unwrap();
        let mut plain = vec![0; 50];
        reader.read_exact(&mut plain).unwrap();
        reader.finish_padding(15).unwrap();
        assert_eq!(
            (xdzip_compression::crc32(&plain[2..]) & 65535) as u16,
            u16::from_le_bytes(plain[..2].try_into().unwrap())
        );
        assert_eq!(plain[2], 116);
        assert_eq!(u16::from_le_bytes(plain[5..7].try_into().unwrap()), 50);
        assert_eq!(&plain[32..37], b"c.txt");
    }
    {
        let bytes = unhex(
            "ef04e937deffa39329e59c4521575117d6fce3e105ae210700a2430cd555e2c5a89368110d013cf7841fd1cdd5ffc2bddb9ffdacee95cf9553a4cfed832dfa67",
        );
        let mut reader =
            LegacyReader::new(bytes.as_slice(), 64, LegacyCipher::aes128(key), &control).unwrap();
        let mut plain = vec![0; 50];
        reader.read_exact(&mut plain).unwrap();
        reader.finish_padding(15).unwrap();
        assert_eq!(
            (xdzip_compression::crc32(&plain[2..]) & 65535) as u16,
            u16::from_le_bytes(plain[..2].try_into().unwrap())
        );
        assert_eq!(plain[2], 116);
        assert_eq!(u16::from_le_bytes(plain[5..7].try_into().unwrap()), 50);
        assert_eq!(&plain[32..37], b"d.txt");
    }
    {
        let bytes = unhex("f09f4577eb3c24da4892abfe7915d7a3");
        let mut reader =
            LegacyReader::new(bytes.as_slice(), 16, LegacyCipher::aes128(key), &control).unwrap();
        let mut plain = vec![0; 7];
        reader.read_exact(&mut plain).unwrap();
        reader.finish_padding(15).unwrap();
        assert_eq!(
            (xdzip_compression::crc32(&plain[2..]) & 65535) as u16,
            u16::from_le_bytes(plain[..2].try_into().unwrap())
        );
        assert_eq!(plain[2], 123);
        assert_eq!(u16::from_le_bytes(plain[5..7].try_into().unwrap()), 7);
    }
}

#[test]
fn old20_independent_mathematical_mapping_and_multiblock_vectors() {
    let control = Cancellation::default();
    let mut budget = LegacyKdfBudget::default();
    {
        let password = NarrowPassword::ascii("").unwrap();
        let cipher = LegacyCipher::old20(&password, &control, &mut budget).unwrap();
        let data: Vec<u8> = (0usize..64)
            .map(|i| i.wrapping_mul(29).wrapping_add(7) as u8)
            .collect();
        let mut reader = LegacyReader::new(data.as_slice(), 64, cipher, &control).unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        reader.finish().unwrap();
        assert_eq!(
            hex(&actual),
            "aa7e42d9b1b21b735cdb17e66c58de16bbf5058f1cb9c50accd649a26b0888056efdae0aa9832962f58989f119437f61541bda329fa180043370a6d9606fe82d"
        );
    }
    {
        let password = NarrowPassword::ascii("a").unwrap();
        let cipher = LegacyCipher::old20(&password, &control, &mut budget).unwrap();
        let data: Vec<u8> = (0usize..64)
            .map(|i| i.wrapping_mul(29).wrapping_add(7) as u8)
            .collect();
        let mut reader = LegacyReader::new(data.as_slice(), 64, cipher, &control).unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        reader.finish().unwrap();
        assert_eq!(
            hex(&actual),
            "975e1b6b533df84dbcadca0c2c546de815248df26d9df9b59d21125891a6bbe6a923969721ed5458a20afecb5e4097d2f66fc0eaa038dc299c8b4ecb5e41a825"
        );
    }
    {
        let password = NarrowPassword::ascii("password").unwrap();
        let cipher = LegacyCipher::old20(&password, &control, &mut budget).unwrap();
        let data: Vec<u8> = (0usize..64)
            .map(|i| i.wrapping_mul(29).wrapping_add(7) as u8)
            .collect();
        let mut reader = LegacyReader::new(data.as_slice(), 64, cipher, &control).unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        reader.finish().unwrap();
        assert_eq!(
            hex(&actual),
            "7c8bf2adf3b89569bddbf5f514cab861df4c1444d3091c86479ad4d8bf3b5ef86c8d7274420a451c1c79f7c6fd15e2522b8d3fd3bbaa51946f70ddd1fda734a8"
        );
    }
    {
        let password=NarrowPassword::ascii("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx").unwrap();
        let cipher = LegacyCipher::old20(&password, &control, &mut budget).unwrap();
        let data: Vec<u8> = (0usize..64)
            .map(|i| i.wrapping_mul(29).wrapping_add(7) as u8)
            .collect();
        let mut reader = LegacyReader::new(data.as_slice(), 64, cipher, &control).unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        reader.finish().unwrap();
        assert_eq!(
            hex(&actual),
            "16c737914272cf405b53dcdf3eca79d1bb30cd6f69e3b618c45afd25de5597cfc2f00a8089b4596a27f22c399b0e6c0571b3ab375c218610956d61176d0a304c"
        );
    }
}

// Official UnRAR7.23 arbitrary-cipher output observations; wrappers intentionally
// fail their content CRC (exit3). This is a raw oracle, not archive success.
// Manifest SHA256 6e0d6873ff3e6c02e656ff58c2df291f8cbda68327a5b7b6379ae07f1dcc1c29
#[test]
fn official_raw_old13_old15_old20_decrypt_oracles() {
    let control = Cancellation::default();
    let mut budget = LegacyKdfBudget::default();
    {
        let password = NarrowPassword::from_projected(&unhex("61")).unwrap();
        let cipher = LegacyCipher::old13(&password);
        let bytes = unhex("0724415e7b98b5d2ef0c294663809dbad7f4112e4b6885a2bfdcf91633506d");
        let mut reader =
            LegacyReader::new(bytes.as_slice(), bytes.len() as u64, cipher, &control).unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        reader.finish().unwrap();
        assert_eq!(
            hex(&actual),
            "83bb31e5d70775210b33993d1f3f9d39132b8115e7f745d19ba3e96d2f2f6d",
            "family13/ascii-a"
        );
    }
    {
        let password = NarrowPassword::from_projected(&unhex("70617373776f7264")).unwrap();
        let cipher = LegacyCipher::old13(&password);
        let bytes = unhex("0724415e7b98b5d2ef0c294663809dbad7f4112e4b6885a2bfdcf91633506d");
        let mut reader =
            LegacyReader::new(bytes.as_slice(), bytes.len() as u64, cipher, &control).unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        reader.finish().unwrap();
        assert_eq!(
            hex(&actual),
            "38bc030dda6abdd3ac48a7c9ae56c1efe0940b45420285cbd4a02f81966e09",
            "family13/ascii-password"
        );
    }
    {
        let password=NarrowPassword::from_projected(&unhex("61616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161")).unwrap();
        let cipher = LegacyCipher::old13(&password);
        let bytes = unhex("0724415e7b98b5d2ef0c294663809dbad7f4112e4b6885a2bfdcf91633506d");
        let mut reader =
            LegacyReader::new(bytes.as_slice(), bytes.len() as u64, cipher, &control).unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        reader.finish().unwrap();
        assert_eq!(
            hex(&actual),
            "13e74733abaf3f5b0337f7431b7f6febf387a7538b4f9f7be3d75763fb1fcf",
            "family13/ascii-a127"
        );
    }
    {
        let password = NarrowPassword::from_projected(&unhex("61")).unwrap();
        let cipher = LegacyCipher::old15(&password);
        let bytes = unhex("0724415e7b98b5d2ef0c294663809dbad7f4112e4b6885a2bfdcf91633506d");
        let mut reader =
            LegacyReader::new(bytes.as_slice(), bytes.len() as u64, cipher, &control).unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        reader.finish().unwrap();
        assert_eq!(
            hex(&actual),
            "720be41a32c5a940b8b1140b102468f15c1b9e9f629b437898fa7779d1cf8c",
            "family15/ascii-a"
        );
    }
    {
        let password = NarrowPassword::from_projected(&unhex("70617373776f7264")).unwrap();
        let cipher = LegacyCipher::old15(&password);
        let bytes = unhex("0724415e7b98b5d2ef0c294663809dbad7f4112e4b6885a2bfdcf91633506d");
        let mut reader =
            LegacyReader::new(bytes.as_slice(), bytes.len() as u64, cipher, &control).unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        reader.finish().unwrap();
        assert_eq!(
            hex(&actual),
            "44f8deae551e685a871a36e68e20cea47edd8674e85db31a36e16633601c06",
            "family15/ascii-password"
        );
    }
    {
        let password=NarrowPassword::from_projected(&unhex("61616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161")).unwrap();
        let cipher = LegacyCipher::old15(&password);
        let bytes = unhex("0724415e7b98b5d2ef0c294663809dbad7f4112e4b6885a2bfdcf91633506d");
        let mut reader =
            LegacyReader::new(bytes.as_slice(), bytes.len() as u64, cipher, &control).unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        reader.finish().unwrap();
        assert_eq!(
            hex(&actual),
            "6338bbb070cf146dd1b215e22afbcbfd4978180b203d28263fdd9258458f76",
            "family15/ascii-a127"
        );
    }
    {
        let password = NarrowPassword::from_projected(&unhex("61")).unwrap();
        let cipher = LegacyCipher::old20(&password, &control, &mut budget).unwrap();
        let bytes = unhex(
            "0724415e7b98b5d2ef0c294663809dbad7f4112e4b6885a2bfdcf91633506d8aa7c4e1fe1b3855728facc9e603203d5a7794b1ceeb0825425f7c99b6d3f00d2a",
        );
        let mut reader =
            LegacyReader::new(bytes.as_slice(), bytes.len() as u64, cipher, &control).unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        reader.finish().unwrap();
        assert_eq!(
            hex(&actual),
            "975e1b6b533df84dbcadca0c2c546de815248df26d9df9b59d21125891a6bbe6a923969721ed5458a20afecb5e4097d2f66fc0eaa038dc299c8b4ecb5e41a825",
            "family20/ascii-a"
        );
    }
    {
        let password = NarrowPassword::from_projected(&unhex("70617373776f7264")).unwrap();
        let cipher = LegacyCipher::old20(&password, &control, &mut budget).unwrap();
        let bytes = unhex(
            "0724415e7b98b5d2ef0c294663809dbad7f4112e4b6885a2bfdcf91633506d8aa7c4e1fe1b3855728facc9e603203d5a7794b1ceeb0825425f7c99b6d3f00d2a",
        );
        let mut reader =
            LegacyReader::new(bytes.as_slice(), bytes.len() as u64, cipher, &control).unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        reader.finish().unwrap();
        assert_eq!(
            hex(&actual),
            "7c8bf2adf3b89569bddbf5f514cab861df4c1444d3091c86479ad4d8bf3b5ef86c8d7274420a451c1c79f7c6fd15e2522b8d3fd3bbaa51946f70ddd1fda734a8",
            "family20/ascii-password"
        );
    }
    {
        let password=NarrowPassword::from_projected(&unhex("61616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161")).unwrap();
        let cipher = LegacyCipher::old20(&password, &control, &mut budget).unwrap();
        let bytes = unhex(
            "0724415e7b98b5d2ef0c294663809dbad7f4112e4b6885a2bfdcf91633506d8aa7c4e1fe1b3855728facc9e603203d5a7794b1ceeb0825425f7c99b6d3f00d2a",
        );
        let mut reader =
            LegacyReader::new(bytes.as_slice(), bytes.len() as u64, cipher, &control).unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        reader.finish().unwrap();
        assert_eq!(
            hex(&actual),
            "9f0de9d56dadc549f56f58bfa83cb4c56c04ea1f0a7b8940147bb46ed21a6f2f97f8e8ab7ffb155ca474366ab57f312c30217a8f4ebb46a89a74d2f335321625",
            "family20/ascii-a127"
        );
    }
}
#[test]
fn official_raw_rar3_long_schedule_mutation_cbc_oracles() {
    let control = Cancellation::default();
    let mut cache = LegacyAesCache::default();
    let mut budget = LegacyKdfBudget::default();
    {
        let password = "A".repeat(33);
        let key = cache.key(&password, None, &control, &mut budget).unwrap();
        let cipher = LegacyCipher::aes128(key);
        let bytes = unhex(
            "0724415e7b98b5d2ef0c294663809dbad7f4112e4b6885a2bfdcf91633506d8aa7c4e1fe1b3855728facc9e603203d5a7794b1ceeb0825425f7c99b6d3f00d2a",
        );
        let mut reader = LegacyReader::new(bytes.as_slice(), 64, cipher, &control).unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        reader.finish().unwrap();
        assert_eq!(
            hex(&actual),
            "0f38ec005b02985b4203843667afaf07b581e494dee81bdd322e0046ceb4f3064f901a10ffd1326ae0bc996d994bff1aa1389b41654b3327c6dd244c68526399",
            "RAR3 R0 length66"
        );
    }
    {
        let password = "A".repeat(60);
        let key = cache
            .key(
                &password,
                Some(unhex("0001020304050607").try_into().unwrap()),
                &control,
                &mut budget,
            )
            .unwrap();
        let cipher = LegacyCipher::aes128(key);
        let bytes = unhex(
            "0724415e7b98b5d2ef0c294663809dbad7f4112e4b6885a2bfdcf91633506d8aa7c4e1fe1b3855728facc9e603203d5a7794b1ceeb0825425f7c99b6d3f00d2a",
        );
        let mut reader = LegacyReader::new(bytes.as_slice(), 64, cipher, &control).unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        reader.finish().unwrap();
        assert_eq!(
            hex(&actual),
            "1adc7c3212e13362c61fef36ab08ea639088e031276be372286152d80a963b5e6f6320be6ec5b43ca255449d4096fb19c5be8109bfe4dc208acf2e0ed842de04",
            "RAR3 R0 length128"
        );
    }
    {
        let password = "A".repeat(127);
        let key = cache
            .key(
                &password,
                Some(unhex("0001020304050607").try_into().unwrap()),
                &control,
                &mut budget,
            )
            .unwrap();
        let cipher = LegacyCipher::aes128(key);
        let bytes = unhex(
            "0724415e7b98b5d2ef0c294663809dbad7f4112e4b6885a2bfdcf91633506d8aa7c4e1fe1b3855728facc9e603203d5a7794b1ceeb0825425f7c99b6d3f00d2a",
        );
        let mut reader = LegacyReader::new(bytes.as_slice(), 64, cipher, &control).unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        reader.finish().unwrap();
        assert_eq!(
            hex(&actual),
            "cd7ba157779505c40b0d3b458c0937ed57cd7126fcc68b65931e142537b585a2109f2da61597ddc9990a1579f6bd8456f137d494608436c7c05799438b99aae8",
            "RAR3 R0 length262"
        );
    }
}

// Fixed public Junrar v8.1.0 test data, commit57091f9ccd43661cf8f12c389917cc24950df707.
// Only test metadata/password lines from Rar14EncryptedExtractionTest.java were read,
// SHA256162e94ccd1e63533aff624d72f7c587c9fe259d090ed8581a37e5b79332642f7.
// Original finite ciphertext and original source/license are not project-authored fixtures.
#[test]
fn independent_historical_store_ciphertexts_produce_known_complete_content() {
    let control = Cancellation::default();
    let password = NarrowPassword::ascii("password").unwrap();
    let mut budget = LegacyKdfBudget::default();
    // rar14-password-stored.rar SHA256de1d90a29238af382650cd73fa1fbc0e5a5083d7099c768d4079e3fbfe073a54.
    {
        let bytes = unhex(
            "21a99071d25c2d1fa432e69de7580ceb63c56d4a6cdf203a54ac32fa0f02ca8c97cc43f3d6be78100c20810ee9c8f9859ab5f988473a30f3a49abfe5d3c1e6f3e3b0c3d1a67c7bd15c141642b92a6f9f09a587b6d7532a",
        );
        let cipher = LegacyCipher::old13(&password);
        let mut reader = LegacyReader::new(bytes.as_slice(), 87, cipher, &control).unwrap();
        let mut plain = [0u8; 87];
        reader.read_exact(&mut plain).unwrap();
        reader.finish_padding(15).unwrap();
        let mut hash = SecretSha256::new();
        hash.update(&plain);
        assert_eq!(
            hex(hash.finish().as_ref()),
            "5f512316b5a4d27c6563b83299c5dd7061fc0c4ee6969a4483363b836c2d6951"
        );
        assert_eq!(reader.consumed_bytes(), 87);
    }
    // rar15-password-stored.rar SHA2567c7e4d5dce619990e0ff44c162ef0f3e90db6c32ddeac1c8ce0a38923918eaab.
    {
        let bytes = unhex(
            "119dcdd01fa8e8a809787b80df8e633ec54cf03bc04c16dbe04df740216c0d71efdf70aad87bf70783e807ae041e7e2b6874f90dea5799472927a9754c742d3010b2b7d9d983ec67dc502827679f9af333cb52e853f016",
        );
        let cipher = LegacyCipher::old15(&password);
        let mut reader = LegacyReader::new(bytes.as_slice(), 87, cipher, &control).unwrap();
        let mut plain = [0u8; 87];
        reader.read_exact(&mut plain).unwrap();
        reader.finish_padding(15).unwrap();
        let mut hash = SecretSha256::new();
        hash.update(&plain);
        assert_eq!(
            hex(hash.finish().as_ref()),
            "5f512316b5a4d27c6563b83299c5dd7061fc0c4ee6969a4483363b836c2d6951"
        );
        assert_eq!(reader.consumed_bytes(), 87);
        assert_eq!(xdzip_compression::crc32(&plain), 2524841378);
    }
    // rar20-password-stored.rar SHA2564bb49f71f8380161439250c1a9f13894175565bfab9440126d85b8b69a99f0bf.
    {
        let bytes = unhex(
            "2d422ae3e5206e813239c637f0aeff180d6266bb652bfbeb7f4618b0ec235a4422a85cf9e49b0bff424a2e84cf02d0d4949dbcbc3c853b3fee4e1b2e870b01182774639daa9dc66bde1ad320111db098cd8091f1da0d28062aa8843d65a2ce9d",
        );
        let cipher = LegacyCipher::old20(&password, &control, &mut budget).unwrap();
        let mut reader = LegacyReader::new(bytes.as_slice(), 96, cipher, &control).unwrap();
        let mut plain = [0u8; 87];
        reader.read_exact(&mut plain).unwrap();
        reader.finish_padding(15).unwrap();
        let mut hash = SecretSha256::new();
        hash.update(&plain);
        assert_eq!(
            hex(hash.finish().as_ref()),
            "5f512316b5a4d27c6563b83299c5dd7061fc0c4ee6969a4483363b836c2d6951"
        );
        assert_eq!(reader.consumed_bytes(), 96);
        assert_eq!(xdzip_compression::crc32(&plain), 2524841378);
    }
}
