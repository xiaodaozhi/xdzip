use crate::invalid;
use std::io;

pub(crate) fn initial_order() -> [u8; 256] {
    let mut order = [0; 256];
    let mut at = 0;
    for range in [
        0x20..=0x7f,
        0x00..=0x1f,
        0xa0..=0xdf,
        0x80..=0x9f,
        0xe0..=0xff,
    ] {
        for byte in range {
            order[at] = byte;
            at += 1;
        }
    }
    order
}

pub(crate) fn observe(
    order: &mut [u8; 256],
    byte: u8,
    mut charge: impl FnMut(u64) -> io::Result<()>,
) -> io::Result<()> {
    let mut rank = 0;
    while rank < order.len() {
        charge(1)?;
        if order[rank] == byte {
            break;
        }
        rank += 1;
    }
    if rank == order.len() {
        return Err(invalid());
    }
    for index in (1..=rank).rev() {
        charge(1)?;
        order[index] = order[index - 1];
    }
    order[0] = byte;
    Ok(())
}
