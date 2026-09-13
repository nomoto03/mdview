//! Resolving the destinations that appear in a document's links and images.
//!
//! Resolution is lexical: a destination becomes an absolute path without the
//! filesystem being consulted, so a link to a file that does not exist still
//! resolves. mdview opens it and reports the problem itself, which keeps the
//! "every local path is handed to mdview" rule of ADR-0001 unconditional.

use std::path::{Component, Path, PathBuf};

/// The absolute path a destination points at, or `None` when it is not a plain
/// local path and must be left exactly as the author wrote it: a fragment, a
/// URL carrying a scheme, or a destination naming a remote host.
pub fn resolve(dest: &str, base: &Path) -> Option<PathBuf> {
    if dest.is_empty() || dest.starts_with('#') {
        return None;
    }
    // Both `//host/path` and its backslash form name a host, not a local file.
    if dest.starts_with("//") || dest.starts_with("\\\\") {
        return None;
    }
    if has_scheme(dest) {
        return None;
    }
    // A fragment or query on a local path has nowhere to go: mdview assigns no
    // ids to headings, and a file takes no query string.
    let trimmed = dest.split(['#', '?']).next().unwrap_or(dest);
    if trimmed.is_empty() {
        return None;
    }
    // Markdown authors percent-encode spaces; a literal `%` that is not an
    // escape leaves the decode failing, and the raw text is the right answer.
    let raw = percent_decode(trimmed).unwrap_or_else(|| trimmed.to_owned());
    let candidate = Path::new(&raw);
    let joined = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        base.join(candidate)
    };
    let resolved = normalize(&joined);
    if is_unc(&resolved) {
        return None;
    }
    Some(resolved)
}

/// True when a destination begins with a URL scheme such as `https:` or
/// `mailto:`. A single-letter scheme is read as a Windows drive letter instead,
/// so an absolute path like `C:` followed by a directory stays a path.
fn has_scheme(dest: &str) -> bool {
    let Some(colon) = dest.find(':') else {
        return false;
    };
    if colon < 2 {
        return false;
    }
    let scheme = &dest[..colon];
    scheme.starts_with(|c: char| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-'))
}

/// Resolves `.` and `..` without touching the filesystem. Pushing a bare root
/// onto a path that already carries a Windows prefix keeps that prefix, so the
/// verbatim prefix `canonicalize` produces survives the walk.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// True for a path that names a host rather than a local file. Reading one
/// turns into an SMB connection that hands the user's NTLM hash to whatever
/// host the document named, so mdview refuses them outright. This is the single
/// exception to not restricting which paths a document may reference.
/// See docs/adr/0002-images-served-lazily-over-a-private-protocol.md
pub fn is_unc(path: &Path) -> bool {
    let text = path.as_os_str().to_string_lossy();
    if let Some(rest) = text.strip_prefix(r"\\?\") {
        let head = rest.split(['\\', '/']).next().unwrap_or("");
        return head.eq_ignore_ascii_case("UNC");
    }
    text.starts_with(r"\\") || text.starts_with("//")
}

/// Percent-encodes every byte outside the unreserved set, so a Windows path
/// survives a round trip through a URL intact.
pub fn percent_encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    for byte in text.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char);
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

/// Reverses [`percent_encode`]. `None` when the input is malformed or the bytes
/// it encodes are not UTF-8.
pub fn percent_decode(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len() {
                return None;
            }
            let hi = (bytes[i + 1] as char).to_digit(16)?;
            let lo = (bytes[i + 2] as char).to_digit(16)?;
            out.push((hi * 16 + lo) as u8);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

#[cfg(test)]
mod tests {
    use super::{is_unc, percent_decode, percent_encode, resolve};
    use std::path::{Path, PathBuf};

    fn base() -> PathBuf {
        PathBuf::from(r"C:\work\docs")
    }

    #[test]
    fn relative_destination_resolves_against_the_document() {
        let got = resolve("./other.md", &base()).unwrap();
        assert_eq!(got, PathBuf::from(r"C:\work\docs\other.md"));
    }

    // Shared assets one level up are a normal layout, so climbing out of the
    // document's own directory has to work.
    #[test]
    fn parent_segments_are_resolved() {
        let got = resolve("../assets/logo.png", &base()).unwrap();
        assert_eq!(got, PathBuf::from(r"C:\work\assets\logo.png"));
    }

    #[test]
    fn absolute_destination_is_kept() {
        let got = resolve(r"D:\elsewhere\notes.md", &base()).unwrap();
        assert_eq!(got, PathBuf::from(r"D:\elsewhere\notes.md"));
    }

    #[test]
    fn remote_and_fragment_destinations_are_left_alone() {
        for dest in ["https://example.com/a.md", "mailto:a@b.example", "#section"] {
            assert!(resolve(dest, &base()).is_none(), "dest: {dest}");
        }
    }

    #[test]
    fn network_destinations_are_refused() {
        for dest in [
            r"\\server\share\x.png",
            "//server/share/x.png",
            // Percent-escaped, which is how one survives CommonMark's own
            // unescaping. Caught by the check that runs after decoding rather
            // than by the one on the raw text.
            "%5C%5Cserver%5Cshare%5Cx.png",
        ] {
            assert!(resolve(dest, &base()).is_none(), "dest: {dest}");
        }
    }

    // The extension is deliberately not consulted: an executable resolves like
    // anything else, and is then opened by mdview rather than by the shell.
    #[test]
    fn executables_resolve_like_any_other_file() {
        let got = resolve("./setup.exe", &base()).unwrap();
        assert_eq!(got, PathBuf::from(r"C:\work\docs\setup.exe"));
    }

    #[test]
    fn percent_escapes_are_decoded() {
        let got = resolve("./my%20file.md", &base()).unwrap();
        assert_eq!(got, PathBuf::from(r"C:\work\docs\my file.md"));
    }

    #[test]
    fn unc_is_detected_through_the_verbatim_prefix() {
        assert!(is_unc(Path::new(r"\\?\UNC\server\share\x.png")));
        assert!(is_unc(Path::new(r"\\server\share\x.png")));
        assert!(!is_unc(Path::new(r"\\?\C:\work\docs\x.png")));
        assert!(!is_unc(Path::new(r"C:\work\docs\x.png")));
    }

    #[test]
    fn percent_round_trip_survives_a_windows_path() {
        let path = r"C:\work\docs\図 1.png";
        assert_eq!(percent_decode(&percent_encode(path)).unwrap(), path);
    }

    #[test]
    fn malformed_percent_input_is_rejected() {
        assert!(percent_decode("%zz").is_none());
        assert!(percent_decode("%4").is_none());
    }
}
