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
    /// Eligible records may be rewritten; actual archives can forbid modification.
    pub rewrite: bool,
    /// Fixed private worker identifier, absent for in-process backends.
    pub worker_id: Option<u8>,
}

/// Enumerate built-in declarations without loading a runtime specification or external backend.
pub fn formats() -> &'static [FormatDescriptor] {
    generated::DESCRIPTORS
}

/// Resolve a creation suffix only; this is not archive detection or pathname validation.
pub fn creation_format(name: &str) -> Option<Format> {
    let name = name.to_ascii_lowercase();
    formats().iter().find_map(|descriptor| {
        descriptor
            .creation_suffixes
            .iter()
            .any(|suffix| name.ends_with(suffix))
            .then_some(descriptor.format)
    })
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
            ("archive.gz", None),
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
            (Format::SevenZip, Some(7)),
            (Format::Rar, Some(4)),
            (Format::Cab, Some(8)),
            (Format::Iso, Some(9)),
            (Format::Udf, Some(10)),
        ] {
            assert_eq!(format.descriptor().worker_id, worker);
        }
        assert_eq!(Format::TarGzip.descriptor().filters, ["gzip"]);
        for format in [Format::Rar, Format::Cab, Format::Iso, Format::Udf] {
            assert!(format.descriptor().write_methods.is_empty());
            assert!(!format.descriptor().rewrite);
        }
    }
}
