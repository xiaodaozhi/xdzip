pub(crate) fn plain(kind: &str) -> Vec<u8> {
    if let Some(count) = kind.strip_prefix("clear-") {
        let count: u8 = count.parse().unwrap();
        return [
            (65..65 + count).collect::<Vec<_>>(),
            b"AFTERRESET".repeat(12),
        ]
        .concat();
    }
    match kind {
        "empty" => vec![],
        "phrase" => b"abracadabra abracadabra\n".repeat(128),
        "allbytes" => (0..=255).cycle().take(65536).collect(),
        "runs" => (0..=255u16)
            .flat_map(|value| std::iter::repeat_n(value as u8, 3 + usize::from(value % 253)))
            .collect(),
        "escaped" => [
            vec![0x90; 257],
            vec![b'A'; 512],
            b"\x90B\x90\x90C".repeat(128),
        ]
        .concat(),
        "random" => {
            let mut state = 0x12345678u32;
            (0..65536)
                .map(|_| {
                    state ^= state << 13;
                    state ^= state >> 17;
                    state ^= state << 5;
                    state as u8
                })
                .collect()
        }
        _ => panic!("unknown owned ARC pattern"),
    }
}
