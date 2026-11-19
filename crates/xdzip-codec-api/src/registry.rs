//! Built-in declarations. Actual records, host policy and platform limits narrow these capabilities.
use crate::Format;
mod generated;

/// Static format families and admitted profiles, never permission to decode an arbitrary record.
pub struct FormatDescriptor {
    /// Existing internal container discriminator.
    pub format: Format,
    /// Stable machine-readable identifier.
    pub id: &'static str,
    /// User-visible format label.
    pub label: &'static str,
    /// Container family, independently of outer compression.
    pub container: &'static str,
    /// Ordered outer compression filters.
    pub filters: &'static [&'static str],
    /// Read-method families; each container validates exact parameters.
    pub read_methods: &'static [&'static str],
    /// Creation profiles requiring independent source and output admission.
    pub write_methods: &'static [&'static str],
    /// Creation levels admitted by this format, independent of host presentation.
    pub creation_levels: &'static [u8],
    /// Read encryption families, subject to their container contracts.
    pub read_encryption: &'static [&'static str],
    /// Creation encryption profiles.
    pub write_encryption: &'static [&'static str],
    /// Host-authorized volume layouts accepted for reading.
    pub read_volumes: &'static [&'static str],
    /// Supported output volume layouts.
    pub write_volumes: &'static [&'static str],
    /// Output suffix candidates; the host still validates the entire basename.
    pub creation_suffixes: &'static [&'static str],
    /// Read entry-point suffixes; content validation remains authoritative.
    pub extensions: &'static [&'static str],
    /// Meaning of successful whole-content checking, without claiming absent checksums.
    pub check_kind: &'static str,
    /// Eligible records may be rewritten; actual archives can forbid modification.
    pub rewrite: bool,
    /// Fixed private worker identifier, absent for in-process backends.
    pub worker_id: Option<u8>,
}

impl FormatDescriptor {
    /// Resolve existing host creation method IDs only within this format's profile.
    pub fn creation_method(&self, id: u16) -> Option<&'static str> {
        self.write_methods
            .iter()
            .copied()
            .find(|method| generated::CREATION_METHODS.contains(&(*method, id)))
    }
    /// Resolve a method name only when this format admits creation with it.
    pub fn creation_method_id(&self, method: &str) -> Option<u16> {
        self.write_methods.contains(&method).then_some(())?;
        generated::CREATION_METHODS
            .iter()
            .find_map(|&(name, id)| (name == method).then_some(id))
    }
}

/// Enumerate built-in declarations without loading a runtime specification or external backend.
pub fn formats() -> &'static [FormatDescriptor] {
    generated::DESCRIPTORS
}

/// Resolve a creation suffix only; this is not archive detection or pathname validation.
pub fn creation_format(name: &str) -> Option<Format> {
    if is_sfx_creation(name) && !cfg!(all(windows, target_arch = "x86_64")) {
        return None;
    }
    let name = name.to_ascii_lowercase();
    formats()
        .iter()
        .flat_map(|descriptor| {
            descriptor
                .creation_suffixes
                .iter()
                .map(move |suffix| (descriptor.format, *suffix))
        })
        .filter(|(_, suffix)| name.ends_with(suffix))
        .max_by_key(|(_, suffix)| suffix.len())
        .map(|(format, _)| format)
}

/// Recognize only the explicit owned SFX creation suffix, not generic executables.
pub fn is_sfx_creation(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with(generated::SFX_SUFFIX)
}

/// Resolve only a fully validated zero-only stream's ambiguous container view.
/// The longest same-filter suffix cannot identify or validate compressed content.
pub fn compressed_view_hint(name: &str, ordinary: Format) -> Option<Format> {
    let name = name.to_ascii_lowercase();
    let filters = ordinary.descriptor().filters;
    if ordinary.descriptor().container != "single-file" || filters.is_empty() {
        return None;
    }
    formats()
        .iter()
        .filter(|descriptor| {
            descriptor.filters == filters && matches!(descriptor.container, "single-file" | "tar")
        })
        .flat_map(|descriptor| {
            descriptor
                .extensions
                .iter()
                .map(move |suffix| (descriptor.format, *suffix))
        })
        .filter(|(_, suffix)| name.ends_with(suffix))
        .max_by_key(|(_, suffix)| suffix.len())
        .map(|(format, _)| format)
}

impl Format {
    /// Retrieve this built-in's declaration; input features and host policy remain authoritative.
    pub fn descriptor(self) -> &'static FormatDescriptor {
        generated::descriptor(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ambiguous_view_hints_use_only_the_longest_same_filter_suffix() {
        for (name, ordinary, expected) in [
            ("ordinary.GZ", Format::Gzip, Some(Format::Gzip)),
            ("empty.TAR.GZ", Format::Gzip, Some(Format::TarGzip)),
            ("empty.tgz", Format::Gzip, Some(Format::TarGzip)),
            ("ordinary.xz", Format::Xz, Some(Format::Xz)),
            ("empty.txz", Format::Xz, Some(Format::TarXz)),
            ("ordinary.lz4", Format::Lz4, Some(Format::Lz4)),
            ("empty.tar.lz4", Format::Lz4, Some(Format::TarLz4)),
            ("ordinary.zst", Format::Zstd, Some(Format::Zstd)),
            ("empty.tzst", Format::Zstd, Some(Format::TarZstd)),
            ("empty.tbz2", Format::Bzip2, Some(Format::TarBzip2)),
            ("ordinary.lzma", Format::Lzma, Some(Format::Lzma)),
            ("ordinary.lz", Format::Lzip, Some(Format::Lzip)),
            ("wrong.xz", Format::Zstd, None),
            ("unknown.bin", Format::Gzip, None),
            ("archive.zip", Format::Zip, None),
        ] {
            assert_eq!(compressed_view_hint(name, ordinary), expected, "{name}");
        }
    }

    #[test]
    fn creation_candidates_keep_existing_case_and_read_only_boundaries() {
        assert_eq!(
            creation_format("archive.SFX.EXE"),
            cfg!(all(windows, target_arch = "x86_64")).then_some(Format::Zip)
        );
        for (name, expected) in [
            ("ARCHIVE.ZIP", Some(Format::Zip)),
            ("archive.tar", Some(Format::Tar)),
            ("archive.TAR.GZ", Some(Format::TarGzip)),
            ("archive.tgz", Some(Format::TarGzip)),
            ("archive.7Z", Some(Format::SevenZip)),
            ("archive.gz", Some(Format::Gzip)),
            ("archive.XZ", Some(Format::Xz)),
            ("archive.TAR.XZ", Some(Format::TarXz)),
            ("archive.TXZ", Some(Format::TarXz)),
            ("archive.LZ4", Some(Format::Lz4)),
            ("archive.TAR.LZ4", Some(Format::TarLz4)),
            ("archive.ZST", Some(Format::Zstd)),
            ("archive.TAR.ZST", Some(Format::TarZstd)),
            ("archive.TZST", Some(Format::TarZstd)),
            ("archive.rar", None),
            ("archive.cab", None),
            ("archive.iso", Some(Format::Iso)),
            ("archive.udf", None),
            ("archive.zip.exe", None),
        ] {
            assert_eq!(creation_format(name), expected, "{name}");
        }
    }

    #[test]
    fn declared_worker_identifiers_keep_the_private_protocol() {
        for (format, worker) in [
            (Format::Zip, None),
            (Format::Tar, None),
            (Format::TarGzip, None),
            (Format::Gzip, None),
            (Format::Bzip2, None),
            (Format::TarBzip2, None),
            (Format::Lzma, None),
            (Format::TarLzma, None),
            (Format::Lzip, None),
            (Format::TarLzip, None),
            (Format::Xz, None),
            (Format::TarXz, None),
            (Format::SevenZip, Some(7)),
            (Format::Rar, Some(4)),
            (Format::Cab, Some(8)),
            (Format::Iso, Some(9)),
            (Format::Udf, Some(10)),
        ] {
            assert_eq!(format.descriptor().worker_id, worker);
        }
        assert_eq!(Format::TarGzip.descriptor().filters, ["gzip"]);
        for format in [Format::Lz4, Format::TarLz4] {
            let declaration = format.descriptor();
            assert_eq!(declaration.creation_method(256), Some("lz4"));
            assert_eq!(declaration.creation_method_id("lz4"), Some(256));
            assert_eq!(declaration.creation_method(8), None);
            assert_eq!(declaration.creation_levels, [1, 6, 9]);
            assert_eq!(declaration.rewrite, format == Format::TarLz4);
            assert_eq!(declaration.worker_id, None);
        }
        assert_eq!(Format::Zip.descriptor().creation_method(256), None);
        let lha = Format::Lha.descriptor();
        assert_eq!(lha.creation_method(259), Some("lh5"));
        assert_eq!(lha.creation_method_id("lh5"), Some(259));
        assert_eq!(lha.creation_levels, [1, 6, 9]);
        assert!(lha.rewrite && lha.write_encryption.is_empty() && lha.write_volumes.is_empty());
        assert_eq!(Format::Zip.descriptor().creation_method(259), None);
        assert_eq!(lha.creation_method(5), None);
        let pma = Format::Pma.descriptor();
        assert_eq!(pma.read_methods, ["store", "pm1", "pm2"]);
        assert!(pma.write_methods.is_empty() && pma.creation_suffixes.is_empty());
        assert!(!pma.rewrite && pma.worker_id.is_none());
        assert!(pma.read_encryption.is_empty() && pma.read_volumes.is_empty());
        let arj = Format::Arj.descriptor();
        assert_eq!(arj.read_methods, ["store", "arj1", "arj2", "arj3", "arj4"]);
        assert!(arj.write_methods.is_empty() && arj.creation_suffixes.is_empty());
        assert!(!arj.rewrite && arj.worker_id.is_none());
        assert!(arj.read_encryption.is_empty() && arj.read_volumes.is_empty());
        let arc = Format::Arc.descriptor();
        assert_eq!(arc.read_methods.len(), 9);
        assert_eq!(arc.check_kind, "supported_content");
        assert!(arc.write_methods.is_empty() && arc.creation_suffixes.is_empty());
        assert!(!arc.rewrite && arc.worker_id.is_none());
        assert!(arc.read_encryption.is_empty() && arc.read_volumes.is_empty());
        for format in [Format::Zstd, Format::TarZstd] {
            let declaration = format.descriptor();
            assert_eq!(declaration.creation_method(93), Some("zstd"));
            assert_eq!(declaration.creation_method_id("zstd"), Some(93));
            assert_eq!(declaration.creation_method(0), None);
            assert_eq!(declaration.creation_method(8), None);
            assert_eq!(declaration.creation_levels, [1, 6, 9]);
            assert_eq!(declaration.check_kind, "format_declared_checks");
            assert_eq!(declaration.rewrite, format == Format::TarZstd);
            assert_eq!(declaration.worker_id, None);
        }
        assert_eq!(Format::Zip.descriptor().creation_method(93), None);
        for format in [Format::Xz, Format::TarXz] {
            assert_eq!(format.descriptor().creation_method(32), Some("lzma2"));
            assert_eq!(format.descriptor().creation_method(0), None);
            assert_eq!(format.descriptor().creation_method(8), None);
            assert_eq!(format.descriptor().rewrite, format == Format::TarXz);
            assert_eq!(format.descriptor().check_kind, "format_declared_checks");
        }
        for format in [
            Format::Rar,
            Format::Cab,
            Format::Compound,
            Format::Udf,
            Format::Pea,
            Format::Bzip2,
            Format::Lzma,
            Format::Lzip,
            Format::Compress,
            Format::Brotli,
        ] {
            assert!(format.descriptor().write_methods.is_empty());
            assert!(!format.descriptor().rewrite);
        }
        for format in [
            Format::TarBzip2,
            Format::TarLzma,
            Format::TarLzip,
            Format::TarCompress,
            Format::TarBrotli,
        ] {
            let declaration = format.descriptor();
            assert!(declaration.rewrite);
            assert!(declaration.write_methods.is_empty());
            assert!(declaration.creation_suffixes.is_empty());
        }
        let iso = Format::Iso.descriptor();
        assert_eq!(iso.creation_method(260), Some("iso9660"));
        assert_eq!(iso.creation_levels, [0]);
        assert!(!iso.rewrite && iso.write_encryption.is_empty() && iso.write_volumes.is_empty());
    }
}
