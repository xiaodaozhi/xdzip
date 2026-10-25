// Project-owned fixture identities and inputs are recorded in tests/fixtures/crypto/sources.json.
pub(super) struct Owned {
    pub parts: &'static [&'static [u8]],
    pub lengths: &'static [usize],
    pub encrypted: bool,
    pub noise: bool,
}
pub(super) const OWNED: &[Owned] = &[
    Owned {
        parts: &[include_bytes!(
            "../tests/fixtures/crypto/empty-store-crc/owned.rar"
        )],
        lengths: &[0],
        encrypted: false,
        noise: false,
    },
    Owned {
        parts: &[include_bytes!(
            "../tests/fixtures/crypto/plain-store-blake/owned.rar"
        )],
        lengths: &[65],
        encrypted: false,
        noise: false,
    },
    Owned {
        parts: &[include_bytes!(
            "../tests/fixtures/crypto/empty-encrypted-store/owned.rar"
        )],
        lengths: &[0],
        encrypted: true,
        noise: false,
    },
    Owned {
        parts: &[include_bytes!(
            "../tests/fixtures/crypto/encrypted-store-crc/owned.rar"
        )],
        lengths: &[17],
        encrypted: true,
        noise: false,
    },
    Owned {
        parts: &[include_bytes!(
            "../tests/fixtures/crypto/encrypted-store-blake/owned.rar"
        )],
        lengths: &[513],
        encrypted: true,
        noise: false,
    },
    Owned {
        parts: &[include_bytes!(
            "../tests/fixtures/crypto/encrypted-lz-blake-headers/owned.rar"
        )],
        lengths: &[65537],
        encrypted: true,
        noise: false,
    },
    Owned {
        parts: &[include_bytes!(
            "../tests/fixtures/crypto/encrypted-solid-blake-headers/owned.rar"
        )],
        lengths: &[32769, 65537],
        encrypted: true,
        noise: false,
    },
    Owned {
        parts: &[
            include_bytes!("../tests/fixtures/crypto/encrypted-split-store-blake/owned.part1.rar"),
            include_bytes!("../tests/fixtures/crypto/encrypted-split-store-blake/owned.part2.rar"),
            include_bytes!("../tests/fixtures/crypto/encrypted-split-store-blake/owned.part3.rar"),
            include_bytes!("../tests/fixtures/crypto/encrypted-split-store-blake/owned.part4.rar"),
            include_bytes!("../tests/fixtures/crypto/encrypted-split-store-blake/owned.part5.rar"),
            include_bytes!("../tests/fixtures/crypto/encrypted-split-store-blake/owned.part6.rar"),
        ],
        lengths: &[20000],
        encrypted: true,
        noise: false,
    },
    Owned {
        parts: &[
            include_bytes!(
                "../tests/fixtures/crypto/encrypted-split-solid-lz-blake/owned.part01.rar"
            ),
            include_bytes!(
                "../tests/fixtures/crypto/encrypted-split-solid-lz-blake/owned.part02.rar"
            ),
            include_bytes!(
                "../tests/fixtures/crypto/encrypted-split-solid-lz-blake/owned.part03.rar"
            ),
            include_bytes!(
                "../tests/fixtures/crypto/encrypted-split-solid-lz-blake/owned.part04.rar"
            ),
            include_bytes!(
                "../tests/fixtures/crypto/encrypted-split-solid-lz-blake/owned.part05.rar"
            ),
            include_bytes!(
                "../tests/fixtures/crypto/encrypted-split-solid-lz-blake/owned.part06.rar"
            ),
            include_bytes!(
                "../tests/fixtures/crypto/encrypted-split-solid-lz-blake/owned.part07.rar"
            ),
            include_bytes!(
                "../tests/fixtures/crypto/encrypted-split-solid-lz-blake/owned.part08.rar"
            ),
            include_bytes!(
                "../tests/fixtures/crypto/encrypted-split-solid-lz-blake/owned.part09.rar"
            ),
            include_bytes!(
                "../tests/fixtures/crypto/encrypted-split-solid-lz-blake/owned.part10.rar"
            ),
            include_bytes!(
                "../tests/fixtures/crypto/encrypted-split-solid-lz-blake/owned.part11.rar"
            ),
            include_bytes!(
                "../tests/fixtures/crypto/encrypted-split-solid-lz-blake/owned.part12.rar"
            ),
            include_bytes!(
                "../tests/fixtures/crypto/encrypted-split-solid-lz-blake/owned.part13.rar"
            ),
            include_bytes!(
                "../tests/fixtures/crypto/encrypted-split-solid-lz-blake/owned.part14.rar"
            ),
            include_bytes!(
                "../tests/fixtures/crypto/encrypted-split-solid-lz-blake/owned.part15.rar"
            ),
        ],
        lengths: &[32769, 65537],
        encrypted: true,
        noise: true,
    },
];
