fn random(size: usize) -> Vec<u8> {
    let mut state = 0x12345678u32;
    (0..size)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as u8
        })
        .collect()
}
pub(crate) fn plain(kind: &str) -> Vec<u8> {
    match kind {
        "empty" => vec![],
        "phrase" => b"abracadabra abracadabra\n".repeat(128),
        "spaces" => vec![b' '; 65536],
        "allbytes" => (0..=255).cycle().take(65536).collect(),
        "random" => random(32768),
        "mixed" => [
            (0..=255).collect::<Vec<u8>>(),
            b"abracadabra abracadabra\n".repeat(128),
        ]
        .concat()
        .into_iter()
        .cycle()
        .take(131072)
        .collect(),
        "window21k" => {
            let b = random(24000);
            [b.clone(), b[3000..13000].to_vec()].concat()
        }
        "window14k" => {
            let b = random(15800);
            [b.clone(), b[1000..13000].to_vec()].concat()
        }
        _ => panic!("unknown owned input"),
    }
}
