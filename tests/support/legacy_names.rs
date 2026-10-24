// tests/support/legacy_names.rs

//! Owned raw-name fixtures, independent of the product's encoders and charset detector.
#![allow(dead_code)]
pub const CONTENT: &[u8] = b"owned legacy payload\n";
fn n16(data: &mut [u8], at: usize, value: usize) {
    data[at..at + 2].copy_from_slice(&(value as u16).to_le_bytes());
}
fn n32(data: &mut [u8], at: usize, value: usize) {
    data[at..at + 4].copy_from_slice(&(value as u32).to_le_bytes());
}
pub fn crc(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for b in data {
        crc ^= u32::from(*b);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}
pub fn unicode_extra(raw: &[u8], unicode: &[u8]) -> Vec<u8> {
    let mut extra = vec![0x75, 0x70, 0, 0, 1];
    n16(&mut extra, 2, unicode.len() + 5);
    extra.extend(crc(raw).to_le_bytes());
    extra.extend(unicode);
    extra
}
pub fn zip(names: &[(&[u8], u16, &[u8])]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut central = Vec::new();
    for (raw, flags, extra) in names {
        let directory = raw.ends_with(b"/");
        let content = if directory { &[][..] } else { CONTENT };
        let offset = bytes.len();
        let mut local = [0; 30];
        local[..4].copy_from_slice(b"PK\x03\x04");
        n16(&mut local, 4, 20);
        n16(&mut local, 6, usize::from(*flags));
        n32(&mut local, 14, crc(content) as usize);
        n32(&mut local, 18, content.len());
        n32(&mut local, 22, content.len());
        n16(&mut local, 26, raw.len());
        n16(&mut local, 28, extra.len());
        bytes.extend(local);
        bytes.extend(*raw);
        bytes.extend(*extra);
        bytes.extend(content);
        let mut record = [0; 46];
        record[..4].copy_from_slice(b"PK\x01\x02");
        n16(&mut record, 4, 20);
        n16(&mut record, 6, 20);
        n16(&mut record, 8, usize::from(*flags));
        n32(&mut record, 16, crc(content) as usize);
        n32(&mut record, 20, content.len());
        n32(&mut record, 24, content.len());
        n16(&mut record, 28, raw.len());
        n16(&mut record, 30, extra.len());
        n32(&mut record, 38, if directory { 16 } else { 0 });
        n32(&mut record, 42, offset);
        central.extend(record);
        central.extend(*raw);
        central.extend(*extra);
    }
    let offset = bytes.len();
    let size = central.len();
    bytes.extend(central);
    let mut end = [0; 22];
    end[..4].copy_from_slice(b"PK\x05\x06");
    n16(&mut end, 8, names.len());
    n16(&mut end, 10, names.len());
    n32(&mut end, 12, size);
    n32(&mut end, 16, offset);
    bytes.extend(end);
    bytes
}
fn octal(block: &mut [u8], at: usize, length: usize, value: usize) {
    let text = format!("{value:0width$o}\0", width = length - 1);
    block[at..at + length].copy_from_slice(text.as_bytes());
}
pub fn tar(names: &[&[u8]]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for raw in names {
        assert!(raw.len() <= 100);
        let directory = raw.ends_with(b"/");
        let content = if directory { &[][..] } else { CONTENT };
        let mut header = [0; 512];
        header[..raw.len()].copy_from_slice(raw);
        octal(&mut header, 100, 8, 0o644);
        octal(&mut header, 108, 8, 0);
        octal(&mut header, 116, 8, 0);
        octal(&mut header, 124, 12, content.len());
        octal(&mut header, 136, 12, 0);
        header[148..156].fill(b' ');
        header[156] = if directory { b'5' } else { b'0' };
        header[257..263].copy_from_slice(b"ustar\0");
        header[263..265].copy_from_slice(b"00");
        let sum: usize = header.iter().map(|b| usize::from(*b)).sum();
        let check = format!("{sum:06o}\0 ");
        header[148..156].copy_from_slice(check.as_bytes());
        bytes.extend(header);
        bytes.extend(content);
        bytes.resize(bytes.len() + ((512 - content.len() % 512) % 512), 0);
    }
    bytes.resize(bytes.len() + 1024, 0);
    bytes
}
/// Byte constants were encoded with Python's independent standard-library codecs.
pub type Sample = Vec<(&'static [u8], &'static str)>;
pub fn samples() -> Vec<(&'static str, Sample)> {
    vec![
        ("GBK-overlap",vec![(b"\xc4\xbf\xc2\xbc/".as_slice(),"目录/"),(b"\xc4\xbf\xc2\xbc/\xd6\xd0\xce\xc4.txt".as_slice(),"目录/中文.txt")]),
        ("UnicodePath", vec![(b"legacy/".as_slice(),"资料/"),(b"legacy/original.txt".as_slice(),"资料/原文件.txt")]),
        ("UTF8-unmarked",vec![("多语言资料/".as_bytes(),"多语言资料/"),("多语言资料/日本語と中文.txt".as_bytes(),"多语言资料/日本語と中文.txt")]),("GBK",vec![(b"\xd6\xd0\xce\xc4\xd7\xca\xc1\xcf/".as_slice(),"中文资料/"),(b"\xd6\xd0\xce\xc4\xd7\xca\xc1\xcf/\xd6\xd0\xce\xc4\xce\xc4\xbc\xfe\xd3\xeb\xd1\xb9\xcb\xf5\xb2\xe2\xca\xd4.txt".as_slice(),"中文资料/中文文件与压缩测试.txt"),(b"\xd6\xd0\xce\xc4\xd7\xca\xc1\xcf/\xbf\xe7\xc6\xbd\xcc\xa8\xbd\xe2\xd1\xb9\xca\xb9\xd3\xc3\xcb\xb5\xc3\xf7.txt".as_slice(),"中文资料/跨平台解压使用说明.txt")]),
("GB18030",vec![(b"\xd6\xd0\xce\xc4\xd7\xca\xc1\xcf/".as_slice(),"中文资料/"),(b"\xd6\xd0\xce\xc4\xd7\xca\xc1\xcf/\xd6\xd0\xce\xc4\xce\xc4\xbc\xfe\xd3\xeb\xd1\xb9\xcb\xf5\xb2\xe2\xca\xd4.txt".as_slice(),"中文资料/中文文件与压缩测试.txt"),(b"\xd6\xd0\xce\xc4\xd7\xca\xc1\xcf/\xbf\xe7\xc6\xbd\xcc\xa8\xbd\xe2\xd1\xb9\xca\xb9\xd3\xc3\xcb\xb5\xc3\xf7.txt".as_slice(),"中文资料/跨平台解压使用说明.txt"),(b"\xd6\xd0\xce\xc4\xd7\xca\xc1\xcf/\x954\xb25\xd2\xb0\xbc\xd2\x949\xfc6.txt".as_slice(),"中文资料/𠮷野家😀.txt")]),
("Big5",vec![(b"\xc1c\xc5\xe9\xa4\xa4\xa4\xe5\xb8\xea\xae\xc6/".as_slice(),"繁體中文資料/"),(b"\xc1c\xc5\xe9\xa4\xa4\xa4\xe5\xb8\xea\xae\xc6/\xc0\xc9\xae\xd7\xbb\xa1\xa9\xfa\xbbP\xb4\xfa\xb8\xd5.txt".as_slice(),"繁體中文資料/檔案說明與測試.txt"),(b"\xc1c\xc5\xe9\xa4\xa4\xa4\xe5\xb8\xea\xae\xc6/\xc0\xa3\xc1Y\xa4\xe5\xa5\xf3\xa9M\xac\xdb\xa4\xf9.txt".as_slice(),"繁體中文資料/壓縮文件和相片.txt")]),
("Shift_JIS",vec![(b"\x93\xfa\x96{\x8c\xea\x8e\x91\x97\xbf/".as_slice(),"日本語資料/"),(b"\x93\xfa\x96{\x8c\xea\x8e\x91\x97\xbf/\x83\x5c\x83t\x83g\x83E\x83F\x83A\x90\xe0\x96\xbe\x8f\x91.txt".as_slice(),"日本語資料/ソフトウェア説明書.txt"),(b"\x93\xfa\x96{\x8c\xea\x8e\x91\x97\xbf/\x82\xd0\x82\xe7\x82\xaa\x82\xc8\x83J\x83^\x83J\x83i.txt".as_slice(),"日本語資料/ひらがなカタカナ.txt")]),
("EUC-KR",vec![(b"\xc7\xd1\xb1\xdb\xc0\xda\xb7\xe1/".as_slice(),"한글자료/"),(b"\xc7\xd1\xb1\xdb\xc0\xda\xb7\xe1/\xbe\xd0\xc3\xe0\xc6\xc4\xc0\xcf\xbc\xb3\xb8\xed\xbc\xad.txt".as_slice(),"한글자료/압축파일설명서.txt"),(b"\xc7\xd1\xb1\xdb\xc0\xda\xb7\xe1/\xbb\xe7\xc1\xf8\xb0\xfa\xb9\xae\xbc\xad\xc0\xd4\xb4\xcf\xb4\xd9.txt".as_slice(),"한글자료/사진과문서입니다.txt")]),
("windows-1251",vec![(b"\xd0\xf3\xf1\xf1\xea\xe8\xe5 \xe4\xee\xea\xf3\xec\xe5\xed\xf2\xfb/".as_slice(),"Русские документы/"),(b"\xd0\xf3\xf1\xf1\xea\xe8\xe5 \xe4\xee\xea\xf3\xec\xe5\xed\xf2\xfb/\xce\xef\xe8\xf1\xe0\xed\xe8\xe5 \xe0\xf0\xf5\xe8\xe2\xe0.txt".as_slice(),"Русские документы/Описание архива.txt"),(b"\xd0\xf3\xf1\xf1\xea\xe8\xe5 \xe4\xee\xea\xf3\xec\xe5\xed\xf2\xfb/\xd2\xe5\xf1\xf2\xe8\xf0\xee\xe2\xe0\xed\xe8\xe5 \xf4\xe0\xe9\xeb\xee\xe2.txt".as_slice(),"Русские документы/Тестирование файлов.txt")]),
("CP437",vec![(b"caf\x82.txt".as_slice(),"café.txt")])]
}

pub fn zip_sample(sample: &[(&[u8], &str)], encoding: &str, target: bool) -> Vec<u8> {
    let extras: Vec<_> = sample
        .iter()
        .map(|(raw, name)| {
            if encoding == "UnicodePath" {
                unicode_extra(raw, name.as_bytes())
            } else {
                vec![]
            }
        })
        .collect();
    let mut specs: Vec<_> = sample
        .iter()
        .zip(&extras)
        .map(|((raw, _), extra)| (*raw, 0, extra.as_slice()))
        .collect();
    if target {
        specs.push((b"target/", 0, &[]));
    }
    zip(&specs)
}
