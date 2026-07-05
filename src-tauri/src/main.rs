#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod markdown;

use std::path::Path;

fn read_utf8(path: &Path) -> Result<String, String> {
    let display = path.display();
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(format!("File not found: {display}"));
        }
        Err(e) => return Err(format!("Cannot read {display}: {e}")),
    };
    String::from_utf8(bytes).map_err(|_| format!("{display} is not valid UTF-8."))
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::read_utf8;
    use std::path::PathBuf;

    fn temp_file(name: &str, bytes: &[u8]) -> PathBuf {
        let path = std::env::temp_dir().join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn missing_file_error() {
        let path = std::env::temp_dir().join("mdview-nonexistent-xyz.md");
        let err = read_utf8(&path).unwrap_err();
        assert!(err.contains("not found"), "error: {err}");
    }

    #[test]
    fn invalid_utf8_error() {
        let path = temp_file("mdview-test-invalid.md", &[0xFF, 0xFE, 0x80]);
        let err = read_utf8(&path).unwrap_err();
        assert!(err.contains("UTF-8"), "error: {err}");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn valid_utf8_ok() {
        let path = temp_file("mdview-test-valid.md", "# ok\n日本語\n".as_bytes());
        let text = read_utf8(&path).unwrap();
        assert_eq!(text, "# ok\n日本語\n");
        let _ = std::fs::remove_file(&path);
    }
}
