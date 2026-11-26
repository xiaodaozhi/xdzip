//! Relative path syntax only. A valid path grants no filesystem authority.

/// Ordinary punctuation whose validity can depend on the destination.
/// Separators and namespace syntax are excluded.
pub const DESTINATION_PUNCTUATION: [char; 6] = ['<', '>', '"', '|', '?', '*'];

/// Parse unchanged UTF-8 components under the initial cross-platform policy.
/// Rejects traversal, device names, ambiguous separators and control characters.
pub fn components(raw: &[u8], directory: bool) -> Option<Vec<&str>> {
    parse_components(raw, directory, true)
}

/// Parse archive names for ordinary extraction, deferring only filename punctuation.
pub fn extraction_components(raw: &[u8], directory: bool) -> Option<Vec<&str>> {
    parse_components(raw, directory, false)
}

fn parse_components(raw: &[u8], directory: bool, portable: bool) -> Option<Vec<&str>> {
    let text = std::str::from_utf8(raw).ok()?;
    if text.len() > 4096 {
        return None;
    }
    let text = if directory {
        text.strip_suffix('/')?
    } else {
        text
    };
    let parts: Vec<_> = text.split('/').collect();
    if parts.len() > 64 || parts.iter().any(|part| {
        let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        part.is_empty() || matches!(*part, "." | "..") || part.len() > 255
            || part.ends_with([' ', '.'])
            || (portable && part.contains(DESTINATION_PUNCTUATION))
            || part.chars().any(|c| c.is_control() || matches!(c, '\\' | ':')
                || matches!(c, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'))
            || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$" | "CONIN$" | "CONOUT$")
            || ["COM", "LPT"].iter().any(|prefix| stem.strip_prefix(prefix).is_some_and(|n| matches!(n, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³")))
    }) { None } else { Some(parts) }
}

/// Owned, syntactically validated relative components. Preserve original spelling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortablePath(Vec<String>);
impl PortablePath {
    /// Parse a relative file path; directories use the same components without a slash suffix.
    pub fn parse(text: &str) -> Option<Self> {
        components(text.as_bytes(), false)
            .map(|parts| Self(parts.into_iter().map(str::to_owned).collect()))
    }
    /// Parse an ordinary extraction path. The destination decides punctuation compatibility.
    pub fn parse_extraction(text: &str) -> Option<Self> {
        extraction_components(text.as_bytes(), false)
            .map(|parts| Self(parts.into_iter().map(str::to_owned).collect()))
    }
    /// Validated components, never an absolute host pathname.
    pub fn parts(&self) -> &[String] {
        &self.0
    }
    /// Archive-relative spelling for task reports and private transaction records.
    pub fn display(&self) -> String {
        self.0.join("/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_extraction_defers_punctuation_only() {
        for c in DESTINATION_PUNCTUATION {
            let name = format!("photos/{c}name.txt");
            assert!(PortablePath::parse(&name).is_none());
            assert_eq!(
                PortablePath::parse_extraction(&name).unwrap().display(),
                name
            );
        }
        for name in [
            "../file?.txt",
            "/file?.txt",
            "./file?.txt",
            "x//file?.txt",
            "C:file?.txt",
            "x/other:stream?",
            "x\\file?",
            "NUL.txt",
            "LPT².txt",
            "file?.",
            "file? ",
            "file?\0",
            "file?\u{202e}",
        ] {
            assert!(PortablePath::parse_extraction(name).is_none(), "{name:?}");
        }
    }
}
