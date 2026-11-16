// Generated independently verified owned PMA expectations, not decoder output.
#[rustfmt::skip]
pub(crate) const CASES: &[(&str, u8, usize, usize, usize, u16)] = &[
    ("pm2-boundary-128.pma", 2, 33, 107, 128, 0xf924),
    ("pm2-boundary-256.pma", 2, 33, 252, 256, 0xbad3),
    ("pm2-boundary-512.pma", 2, 33, 281, 512, 0xb045),
    ("pm2-boundary-1024.pma", 2, 33, 288, 1024, 0x4ebe),
    ("pm2-boundary-2048.pma", 2, 33, 299, 2048, 0xa768),
    ("pm2-boundary-4096.pma", 2, 33, 321, 4096, 0x6cea),
    ("pm2-boundary-8192.pma", 2, 33, 351, 8192, 0xd95c),
    ("pm2-boundary-16384.pma", 2, 33, 410, 16384, 0xe1c8),
    ("pm2-spaces.pma", 2, 33, 26, 8192, 0x65a5),
    ("pm2-zeros.pma", 2, 33, 56, 65536, 0x0000),
    ("pm2-allbytes.pma", 2, 33, 726, 65536, 0x3b54),
    ("pm2-long.pma", 2, 33, 1241, 131072, 0xd50b),
    ("pm0-allbytes.pma", 0, 33, 256, 256, 0xbad3),
    ("empty.pma", 255, 0, 0, 0, 0x0000),
    ("pm2-deep28.pma", 2, 33, 518, 128, 0x26d6),
    ("pm2-prologue-zero.pma", 2, 33, 281, 512, 0xb045),
];
