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
        let method = match id {
            0 if self.write_methods.contains(&"copy") => "copy",
            0 => "store",
            8 => "deflate",
            32 => "lzma2",
            _ => return None,
        };
        self.write_methods.contains(&method).then_some(method)
    }
}

/// Enumerate built-in declarations without loading a runtime specification or external backend.
pub fn formats() -> &'static [FormatDescriptor] {
    generated::DESCRIPTORS
}

/// Resolve a creation suffix only; this is not archive detection or pathname validation.
pub fn creation_format(name: &str) -> Option<Format> {
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
    fn creation_candidates_keep_existing_case_and_read_only_boundaries() {
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
            ("archive.rar", None),
            ("archive.cab", None),
            ("archive.iso", None),
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
        for format in [Format::Xz, Format::TarXz] {
            assert_eq!(format.descriptor().creation_method(32), Some("lzma2"));
            assert_eq!(format.descriptor().creation_method(0), None);
            assert_eq!(format.descriptor().creation_method(8), None);
            assert!(!format.descriptor().rewrite);
            assert_eq!(format.descriptor().check_kind, "format_declared_checks");
        }
        for format in [
            Format::Rar,
            Format::Cab,
            Format::Iso,
            Format::Udf,
            Format::Bzip2,
            Format::TarBzip2,
            Format::Lzma,
            Format::TarLzma,
            Format::Lzip,
            Format::TarLzip,
        ] {
            assert!(format.descriptor().write_methods.is_empty());
            assert!(!format.descriptor().rewrite);
        }
    }
}
