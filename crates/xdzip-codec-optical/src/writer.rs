//! ECMA-119 §§7–10 and Annex C. Planning grants no filesystem authority.
use std::collections::BTreeMap;
use std::io::Read;
use xdzip_codec_api::Cancellation;
use xdzip_codec_api::encode::{
    EncodeEntry, EncodeError, EncodeInput, EncodeOutput, EncodeProgress, EncodeSettings,
};
use xdzip_domain::path::PortablePath;

const SECTOR: usize = 2048;
const METADATA: usize = 64 << 20;
const IMAGE: u64 = 20 << 30;

struct Node {
    input: Option<usize>,
    parent: usize,
    children: Vec<usize>,
    name: String,
    directory: bool,
    size: u32,
    time: [u8; 7],
    alias: Vec<u8>,
    joliet: Vec<u8>,
    block: [u32; 2],
    length: [u32; 2],
}
fn invalid<T>() -> Result<T, EncodeError> {
    Err(EncodeError::Invalid)
}
fn checked(cancel: &Cancellation) -> Result<(), EncodeError> {
    cancel.check().map_err(|_| EncodeError::Cancelled)
}
fn blocks(bytes: usize) -> usize {
    bytes.div_ceil(SECTOR)
}
fn both16(out: &mut [u8], value: u16) {
    out[..2].copy_from_slice(&value.to_le_bytes());
    out[2..4].copy_from_slice(&value.to_be_bytes());
}
fn both32(out: &mut [u8], value: u32) {
    out[..4].copy_from_slice(&value.to_le_bytes());
    out[4..8].copy_from_slice(&value.to_be_bytes());
}
fn timestamp(seconds: i64) -> [u8; 7] {
    // Civil calendar conversion for a bounded UTC timestamp, no OS dependency.
    let seconds = seconds.clamp(0, 4_102_444_799);
    let days = seconds / 86400 + 719468;
    let era = days / 146097;
    let doe = days - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    [
        (year - 1900) as u8,
        month as u8,
        day as u8,
        (seconds / 3600 % 24) as u8,
        (seconds / 60 % 60) as u8,
        (seconds % 60) as u8,
        0,
    ]
}
fn record(node: &Node, ns: usize, name: &[u8]) -> Vec<u8> {
    let length = 33 + name.len() + usize::from(name.len().is_multiple_of(2));
    let mut out = vec![0; length];
    out[0] = length as u8;
    both32(
        &mut out[2..10],
        node.block[if node.directory { ns } else { 0 }],
    );
    both32(
        &mut out[10..18],
        if node.directory {
            node.length[ns]
        } else {
            node.size
        },
    );
    out[18..25].copy_from_slice(&node.time);
    out[25] = if node.directory { 2 } else { 0 };
    both16(&mut out[28..32], 1);
    out[32] = name.len() as u8;
    out[33..33 + name.len()].copy_from_slice(name);
    out
}
fn identifier(node: &Node, ns: usize) -> &[u8] {
    if ns == 0 { &node.alias } else { &node.joliet }
}
fn directory(nodes: &[Node], index: usize, ns: usize) -> Vec<u8> {
    let node = &nodes[index];
    let mut out = Vec::new();
    let mut children = node.children.clone();
    children.sort_by(|a, b| identifier(&nodes[*a], ns).cmp(identifier(&nodes[*b], ns)));
    for (child, name) in [(index, vec![0]), (node.parent, vec![1])]
        .into_iter()
        .chain(
            children
                .into_iter()
                .map(|child| (child, identifier(&nodes[child], ns).to_vec())),
        )
    {
        let bytes = record(&nodes[child], ns, &name);
        let remaining = SECTOR - out.len() % SECTOR;
        if bytes.len() > remaining {
            out.resize(out.len() + remaining, 0);
        }
        out.extend(bytes);
    }
    out.resize(blocks(out.len()) * SECTOR, 0);
    out
}
fn path_table(nodes: &[Node], dirs: &[usize], ns: usize, big: bool) -> Vec<u8> {
    let numbers: BTreeMap<_, _> = dirs
        .iter()
        .enumerate()
        .map(|(n, index)| (*index, (n + 1) as u16))
        .collect();
    let mut out = Vec::new();
    for &index in dirs {
        let node = &nodes[index];
        let name = if index == 0 {
            &[0][..]
        } else {
            identifier(node, ns)
        };
        out.extend([name.len() as u8, 0]);
        out.extend(if big {
            node.block[ns].to_be_bytes()
        } else {
            node.block[ns].to_le_bytes()
        });
        let number = numbers[&node.parent];
        out.extend(if big {
            number.to_be_bytes()
        } else {
            number.to_le_bytes()
        });
        out.extend(name);
        if name.len() % 2 == 1 {
            out.push(0);
        }
    }
    out
}
struct Plan {
    nodes: Vec<Node>,
    dirs: [Vec<usize>; 2],
    tables: [[u32; 2]; 2],
    table_bytes: [u32; 2],
    metadata_blocks: usize,
    total_blocks: u32,
}
impl Plan {
    fn new(entries: &[EncodeEntry], cancel: &Cancellation) -> Result<Self, EncodeError> {
        checked(cancel)?;
        if entries.len() > 100_000 || entries.iter().map(|e| e.name.len()).sum::<usize>() > 4 << 20
        {
            return Err(EncodeError::Limit);
        }
        let mut nodes = vec![Node {
            input: None,
            parent: 0,
            children: vec![],
            name: String::new(),
            directory: true,
            size: 0,
            time: timestamp(0),
            alias: vec![0],
            joliet: vec![0],
            block: [0; 2],
            length: [0; 2],
        }];
        let mut names = BTreeMap::from([(String::new(), 0)]);
        let mut order: Vec<_> = entries.iter().enumerate().collect();
        order.sort_by(|a, b| a.1.name.cmp(&b.1.name));
        for (input, entry) in order {
            checked(cancel)?;
            let path = PortablePath::parse(&entry.name).ok_or(EncodeError::Invalid)?;
            if path.parts().len() > if entry.directory { 7 } else { 8 } {
                return Err(EncodeError::Limit);
            }
            // Joliet counts UCS-2 identifier bytes, the file version and one per parent directory.
            let path_length = path
                .parts()
                .iter()
                .map(|part| part.encode_utf16().count() * 2)
                .sum::<usize>()
                + path.parts().len()
                - 1
                + if entry.directory { 0 } else { 4 };
            if path_length > 240 {
                return Err(EncodeError::Limit);
            }
            if entry.size >= 1u64 << 32 || (entry.directory && entry.size != 0) {
                return Err(EncodeError::Limit);
            }
            let (parent_name, name) = entry.name.rsplit_once('/').unwrap_or(("", &entry.name));
            let parent = *names.get(parent_name).ok_or(EncodeError::Invalid)?;
            if !nodes[parent].directory || names.contains_key(&entry.name) {
                return invalid();
            }
            if name.chars().count() > 64 || name.chars().any(|ch| ch as u32 > 0xffff || ch == ';') {
                return Err(EncodeError::Unsupported);
            }
            let index = nodes.len();
            let alias = if entry.directory {
                format!("D{index:07}")
            } else {
                format!("F{index:07}.BIN;1")
            }
            .into_bytes();
            let full = if entry.directory {
                name.to_string()
            } else {
                format!("{name};1")
            };
            let joliet = full.encode_utf16().flat_map(u16::to_be_bytes).collect();
            nodes.push(Node {
                input: Some(input),
                parent,
                children: vec![],
                name: entry.name.clone(),
                directory: entry.directory,
                size: entry.size as u32,
                time: timestamp(entry.modified_unix),
                alias,
                joliet,
                block: [0; 2],
                length: [0; 2],
            });
            nodes[parent].children.push(index);
            names.insert(entry.name.clone(), index);
        }
        let mut dirs: [Vec<usize>; 2] = Default::default();
        for (ns, list) in dirs.iter_mut().enumerate() {
            list.push(0);
            let mut pos = 0;
            while pos < list.len() {
                checked(cancel)?;
                let mut children: Vec<_> = nodes[list[pos]]
                    .children
                    .iter()
                    .copied()
                    .filter(|i| nodes[*i].directory)
                    .collect();
                children.sort_by(|a, b| identifier(&nodes[*a], ns).cmp(identifier(&nodes[*b], ns)));
                list.extend(children);
                pos += 1;
            }
            if list.len() > u16::MAX as usize {
                return Err(EncodeError::Limit);
            }
        }
        let mut next = 19usize;
        let mut tables = [[0; 2]; 2];
        let mut table_bytes = [0; 2];
        for ns in 0..2 {
            let bytes = path_table(&nodes, &dirs[ns], ns, false).len();
            table_bytes[ns] = bytes as u32;
            for table in &mut tables[ns] {
                *table = next as u32;
                next += blocks(bytes);
            }
            for &index in &dirs[ns] {
                checked(cancel)?;
                let bytes = directory(&nodes, index, ns).len();
                nodes[index].block[ns] = next as u32;
                nodes[index].length[ns] = bytes as u32;
                next += blocks(bytes);
                if next > METADATA / SECTOR {
                    return Err(EncodeError::Limit);
                }
            }
        }
        let metadata_blocks = next;
        for node in &mut nodes {
            if !node.directory {
                node.block[0] = next as u32;
                next += blocks(node.size as usize);
                if next as u64 * SECTOR as u64 > IMAGE {
                    return Err(EncodeError::Limit);
                }
            }
        }
        Ok(Self {
            nodes,
            dirs,
            tables,
            table_bytes,
            metadata_blocks,
            total_blocks: next as u32,
        })
    }
    fn metadata(&self, cancel: &Cancellation) -> Result<Vec<u8>, EncodeError> {
        let mut bytes = vec![0; self.metadata_blocks * SECTOR];
        for ns in 0..2 {
            checked(cancel)?;
            let vd = &mut bytes[(16 + ns) * SECTOR..(17 + ns) * SECTOR];
            vd[0] = if ns == 0 { 1 } else { 2 };
            vd[1..6].copy_from_slice(b"CD001");
            vd[6] = 1;
            vd[8..72].fill(b' ');
            vd[190..813].fill(b' ');
            if ns == 0 {
                vd[40..45].copy_from_slice(b"XDZIP");
            } else {
                vd[8..72].fill(0);
                for (target, ch) in vd[40..]
                    .as_chunks_mut::<2>()
                    .0
                    .iter_mut()
                    .zip("XDZIP".encode_utf16())
                {
                    target.copy_from_slice(&ch.to_be_bytes());
                }
                vd[88..91].copy_from_slice(b"%/E");
                vd[190..813].fill(0);
            }
            both32(&mut vd[80..88], self.total_blocks);
            both16(&mut vd[120..124], 1);
            both16(&mut vd[124..128], 1);
            both16(&mut vd[128..132], SECTOR as u16);
            both32(&mut vd[132..140], self.table_bytes[ns]);
            vd[140..144].copy_from_slice(&self.tables[ns][0].to_le_bytes());
            vd[148..152].copy_from_slice(&self.tables[ns][1].to_be_bytes());
            vd[156..190].copy_from_slice(&record(&self.nodes[0], ns, &[0]));
            for date in vd[813..881].as_chunks_mut::<17>().0 {
                date[..16].fill(b'0');
            }
            vd[881] = 1;
            for big in 0..2 {
                let table = path_table(&self.nodes, &self.dirs[ns], ns, big == 1);
                let start = self.tables[ns][big] as usize * SECTOR;
                bytes[start..start + table.len()].copy_from_slice(&table);
            }
            for &index in &self.dirs[ns] {
                checked(cancel)?;
                let directory = directory(&self.nodes, index, ns);
                let start = self.nodes[index].block[ns] as usize * SECTOR;
                bytes[start..start + directory.len()].copy_from_slice(&directory);
            }
        }
        let end = &mut bytes[18 * SECTOR..19 * SECTOR];
        end[0] = 255;
        end[1..6].copy_from_slice(b"CD001");
        end[6] = 1;
        Ok(bytes)
    }
}

pub(super) fn encode(
    input: &dyn EncodeInput,
    output: &mut dyn EncodeOutput,
    settings: EncodeSettings<'_>,
    cancel: &Cancellation,
    progress: &mut dyn FnMut(EncodeProgress<'_>),
) -> Result<(), EncodeError> {
    if settings.method != "iso9660"
        || settings.level != 0
        || settings.password.is_some()
        || settings.solid
        || settings.encrypt_headers
    {
        return Err(EncodeError::Unsupported);
    }
    let plan = Plan::new(input.entries(), cancel)?;
    let metadata = plan.metadata(cancel)?;
    for chunk in metadata.chunks(128 << 10) {
        checked(cancel)?;
        output.write_all(chunk).map_err(|_| EncodeError::Output)?;
    }
    let mut buffer = vec![0; 128 << 10];
    let mut total = 0;
    for node in &plan.nodes {
        checked(cancel)?;
        if node.directory {
            continue;
        }
        let mut source = input.open(node.input.ok_or(EncodeError::Invalid)?)?;
        let mut remaining = u64::from(node.size);
        while remaining > 0 {
            checked(cancel)?;
            let take = remaining.min(buffer.len() as u64) as usize;
            source
                .read_exact(&mut buffer[..take])
                .map_err(|_| EncodeError::Source)?;
            output
                .write_all(&buffer[..take])
                .map_err(|_| EncodeError::Output)?;
            remaining -= take as u64;
            total += take as u64;
            progress(EncodeProgress {
                name: &node.name,
                input_bytes: total,
            });
        }
        if source
            .read(&mut buffer[..1])
            .map_err(|_| EncodeError::Source)?
            != 0
        {
            return Err(EncodeError::Source);
        }
        let pad = (SECTOR - node.size as usize % SECTOR) % SECTOR;
        checked(cancel)?;
        output
            .write_all(&[0; SECTOR][..pad])
            .map_err(|_| EncodeError::Output)?;
    }
    checked(cancel)?;
    output.flush().map_err(|_| EncodeError::Output)
}

#[cfg(test)]
mod tests;
