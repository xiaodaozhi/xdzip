use super::*;
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
#[test]
fn sha256_hashlib_boundaries_and_clone() {
    check_sha(
        0,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    );
    check_sha(
        1,
        "ca358758f6d27e6cf45272937977a748fd88391db679ceda7dc7bf1f005ee879",
    );
    check_sha(
        55,
        "90049a7259f123e69f1cea2403d496ffa7b647cd45ea9f4487dd778f9578b246",
    );
    check_sha(
        56,
        "40f60a3a579d89962bdffdbc476f9bd0d9a28231f4b65711c902127841d298e2",
    );
    check_sha(
        63,
        "bfb702913b0eabf76eb88475599bd4932114c08fce4e205593d6b7a34eb85a2e",
    );
    check_sha(
        64,
        "e7a402f6bfefa6ff48bc1fe1bb5b77e5c9c24ca9185937a8cf07a8fdaf2424f0",
    );
    check_sha(
        65,
        "d1c3eeb9bcf3d1593eaddd2664f0adb6f4a9c0dc048012780e4b098d4c0aaca8",
    );
    check_sha(
        119,
        "9dda2ff19b1a365f5a1aea438180f213ebd3b9471e96cb6eea744c0155a59a7e",
    );
    check_sha(
        120,
        "0efebb7ad03dacd34c32e6fe6590e1536696faaad47d0bbe68c4c5898716d8dd",
    );
    check_sha(
        127,
        "4c3c60813a50827cc68057ec8ae17d9806f6620a6c98d9be77d26b95b6972ffa",
    );
    check_sha(
        128,
        "a1e19a9775b673aaeb645655cad464ac5cfed26e721f9eb068194c70af4119e9",
    );
    check_sha(
        513,
        "f386c835190d0c750cdf2caa41eeeb68153e6d4575dbe431184a6e3d73b78117",
    );
    check_sha(
        65537,
        "718b9a71949fc252663f30dc7b02be55a5372e6cfb091fd78a86d5868a995e21",
    );
}
fn check_sha(n: usize, expected: &str) {
    let data: Vec<u8> = (0..n)
        .map(|i| (i.wrapping_mul(29).wrapping_add(7)) as u8)
        .collect();
    for chunk in [1, 7, 64, 129, 8192] {
        let mut hash = SecretSha256::new();
        for part in data.chunks(chunk) {
            hash.update(part);
        }
        let copy = hash.clone();
        assert_eq!(hex(hash.finish().as_ref()), expected);
        assert_eq!(hex(copy.finish().as_ref()), expected);
    }
}
#[test]
fn hmac_rfc2104_hashlib_long_keys_and_streaming() {
    check_hmac(
        0,
        0,
        "b613679a0814d9ec772f95d778c35fc5ff1697c493715653c6c712144292c5ad",
    );
    check_hmac(
        20,
        8,
        "f905b669861df53073049813962184257e8d6f33d690e3eed3daa263cfaaf5d7",
    );
    check_hmac(
        64,
        64,
        "08cd61052ec6cec238b9482948b899441533b7d679974960516ec472a801602d",
    );
    check_hmac(
        65,
        63,
        "9999331e706462906af94d9ee91b07330137fc597bb92f2cec6dc71f8884006b",
    );
    check_hmac(
        131,
        152,
        "4df9afb8b97c8c54332475d5c6eb02e99b01620984f419901739198a8a01b02f",
    );
    check_hmac(
        508,
        1024,
        "73ac4d51a55373d3107c8c41af772d2c37c4e510d0d2277c10be054552ae0045",
    );
}
fn check_hmac(kl: usize, ml: usize, expected: &str) {
    let key: Vec<u8> = (0..kl)
        .map(|i| i.wrapping_mul(17).wrapping_add(3) as u8)
        .collect();
    let msg: Vec<u8> = (0..ml)
        .map(|i| i.wrapping_mul(29).wrapping_add(7) as u8)
        .collect();
    let base = HmacSha256::new(&key);
    assert_eq!(hex(base.authenticate(&msg).as_ref()), expected);
    let mut streamed = base.clone();
    for bytes in msg.chunks(3) {
        streamed.update(bytes);
    }
    assert_eq!(hex(streamed.finish().as_ref()), expected);
    assert_eq!(hex(base.authenticate(&msg).as_ref()), expected);
}
#[test]
fn blake2sp_frozen_hashlib_and_official_rar_hashes() {
    check_blake(
        0,
        "dd0e891776933f43c7d032b08a917e25741f8aa9a12c12e1cac8801500f2ca4f",
    );
    check_blake(
        1,
        "49029041da5d740e916f0cd6f1b864f9130c2df21baf9736394faf991f96ec23",
    );
    check_blake(
        63,
        "05ac2c47d9f575cf5acaaab6c9f42c4a05002be3acf28f5e6c996fe3498300ec",
    );
    check_blake(
        64,
        "9efb285a8055e1ddfb7149a1b560c9bd8083588535af775d178b8795627b7a9d",
    );
    check_blake(
        65,
        "605a7fc12e41009b3262bdad6d09a9eda80bb2b9112ee96e24598fae5df9d8da",
    );
    check_blake(
        511,
        "a1d6fa6d3b280e7dfb8aa94cf07fb3170c2e74b65dbd6a05c80e49e84dcc3d1c",
    );
    check_blake(
        512,
        "862d50b38ca6789cec58006ab6dc82fadf9f728ebebebd5ed33961e971695c67",
    );
    check_blake(
        513,
        "d30eea88d83a00381f1e8f5337a9fd70c0901e2488151ed1a5338a1921dad1b9",
    );
    check_blake(
        1024,
        "64f9f128ca6fbb9ed4f892a8f0dea59ede94635a50296790c27982cfe457de1e",
    );
    check_blake(
        65537,
        "51e37c9cf76a0c7e2a13e84b7b8726bc60fa705c0d116ea029daba2a8e5f9a01",
    );
}
fn check_blake(n: usize, expected: &str) {
    let data: Vec<u8> = (0..n)
        .map(|i| i.wrapping_mul(29).wrapping_add(7) as u8)
        .collect();
    for chunk in [1, 7, 63, 64, 65, 511, 512, 8192] {
        let mut hash = Blake2sp::new();
        for part in data.chunks(chunk) {
            hash.update(part);
        }
        assert_eq!(hex(&hash.finish()), expected, "n={n} chunk={chunk}");
    }
}
#[test]
fn blake2s_hashlib_sequential_vectors() {
    {
        let data: Vec<u8> = (0usize..0)
            .map(|i| i.wrapping_mul(29).wrapping_add(7) as u8)
            .collect();
        let mut hash = Blake2s::new();
        for bytes in data.chunks(7) {
            hash.update(bytes);
        }
        assert_eq!(
            hex(&hash.finish()),
            "69217a3079908094e11121d042354a7c1f55b6482ca1a51e1b250dfd1ed0eef9"
        );
    }
    {
        let data: Vec<u8> = (0usize..1)
            .map(|i| i.wrapping_mul(29).wrapping_add(7) as u8)
            .collect();
        let mut hash = Blake2s::new();
        for bytes in data.chunks(7) {
            hash.update(bytes);
        }
        assert_eq!(
            hex(&hash.finish()),
            "6b821f55cab354e67d82cd1ae7c29b69dbdfc28072eb2e0e375e9ad798fedc70"
        );
    }
    {
        let data: Vec<u8> = (0usize..63)
            .map(|i| i.wrapping_mul(29).wrapping_add(7) as u8)
            .collect();
        let mut hash = Blake2s::new();
        for bytes in data.chunks(7) {
            hash.update(bytes);
        }
        assert_eq!(
            hex(&hash.finish()),
            "3e8fedb66a2f59ac76ca899c052242972c6dd5ca8c66106c519688f60c71c6c9"
        );
    }
    {
        let data: Vec<u8> = (0usize..64)
            .map(|i| i.wrapping_mul(29).wrapping_add(7) as u8)
            .collect();
        let mut hash = Blake2s::new();
        for bytes in data.chunks(7) {
            hash.update(bytes);
        }
        assert_eq!(
            hex(&hash.finish()),
            "7fc0d7be2392d2832e65ede094bb9d59db28cede394e56b2d654572cd89d7c35"
        );
    }
    {
        let data: Vec<u8> = (0usize..65)
            .map(|i| i.wrapping_mul(29).wrapping_add(7) as u8)
            .collect();
        let mut hash = Blake2s::new();
        for bytes in data.chunks(7) {
            hash.update(bytes);
        }
        assert_eq!(
            hex(&hash.finish()),
            "3d99f1c0c0271d0caeb636d4712e877e1327e7c00dbaa15d9067a882e5d1b361"
        );
    }
    {
        let data: Vec<u8> = (0usize..128)
            .map(|i| i.wrapping_mul(29).wrapping_add(7) as u8)
            .collect();
        let mut hash = Blake2s::new();
        for bytes in data.chunks(7) {
            hash.update(bytes);
        }
        assert_eq!(
            hex(&hash.finish()),
            "aa8720b8c48c5d79a196db33eafa9b37147e8e626ffaa8842463651706f8ffc4"
        );
    }
    {
        let data: Vec<u8> = (0usize..1024)
            .map(|i| i.wrapping_mul(29).wrapping_add(7) as u8)
            .collect();
        let mut hash = Blake2s::new();
        for bytes in data.chunks(7) {
            hash.update(bytes);
        }
        assert_eq!(
            hex(&hash.finish()),
            "9c5bbcae5fe1499f1ba29a63da45a185868d682fb15fe177a5df2cb01021c7f0"
        );
    }
    {
        let data: Vec<u8> = (0usize..65537)
            .map(|i| i.wrapping_mul(29).wrapping_add(7) as u8)
            .collect();
        let mut hash = Blake2s::new();
        for bytes in data.chunks(7) {
            hash.update(bytes);
        }
        assert_eq!(
            hex(&hash.finish()),
            "9d58f1f6803e2f24cf87158472890d92e0d6680ef9020342203677d25f1c39ec"
        );
    }
}
#[test]
fn fixed_state_memory_is_bounded() {
    assert!(std::mem::size_of::<Blake2sp>() < 2048);
    assert!(std::mem::size_of::<HmacSha256>() < 256);
}
