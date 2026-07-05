#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod markdown;
mod watcher;

use std::path::{Path, PathBuf};
use tauri::Manager;

#[derive(Clone, serde::Serialize)]
struct Document {
    file_name: String,
    html: String,
}

struct AppState {
    doc: Result<Document, String>,
    path: Option<PathBuf>,
}

fn read_utf8(path: &Path) -> Result<String, String> {
    let display = path.display();
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(format!("File not found: {display}"));
        }
        Err(e) => return Err(format!("Cannot read {display}: {e}")),
    };
    let text =
        String::from_utf8(bytes).map_err(|_| format!("{display} is not valid UTF-8."))?;
    // Windows editors (Notepad, PowerShell) often prepend a UTF-8 BOM,
    // which would otherwise break "#" heading detection at file start.
    Ok(text.strip_prefix('\u{feff}').map(str::to_owned).unwrap_or(text))
}

fn load_document() -> (Result<Document, String>, Option<PathBuf>) {
    let Some(arg) = std::env::args().nth(1) else {
        return (Err("No file specified.\nUsage: mdview <file.md>".into()), None);
    };
    let path = PathBuf::from(&arg);
    let text = match read_utf8(&path) {
        Ok(t) => t,
        Err(e) => return (Err(e), None),
    };
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| arg.clone());
    let doc = Document {
        file_name,
        html: markdown::to_html(&text),
    };
    (Ok(doc), path.canonicalize().ok())
}

#[tauri::command]
fn get_initial_content(state: tauri::State<AppState>) -> Result<Document, String> {
    state.doc.clone()
}

fn main() {
    let (doc, path) = load_document();
    tauri::Builder::default()
        .manage(AppState {
            doc: doc.clone(),
            path: path.clone(),
        })
        .invoke_handler(tauri::generate_handler![get_initial_content])
        .setup(move |app| {
            let window = app.get_webview_window("main").expect("main window");
            match &doc {
                Ok(d) => {
                    let _ = window.set_title(&d.file_name);
                }
                Err(_) => {
                    let _ = window.set_title("mdview — error");
                }
            }
            if let Some(p) = path {
                watcher::spawn(app.handle().clone(), p);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running mdview");
}

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
    fn bom_stripped() {
        let path = temp_file("mdview-test-bom.md", b"\xEF\xBB\xBF# ok\n");
        let text = read_utf8(&path).unwrap();
        assert_eq!(text, "# ok\n");
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
