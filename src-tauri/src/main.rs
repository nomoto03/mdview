#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod links;
mod markdown;
mod watcher;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::Manager;

const USAGE: &str = r"# mdview

Windows向けの軽量Markdownビューア。

    mdview <file.md>
    mdview --help
    mdview --version
";

#[derive(Clone, serde::Serialize)]
struct Document {
    file_name: String,
    html: String,
}

struct AppState {
    /// Guarded because the watcher replaces the content as the file changes.
    ///
    /// The watcher MUST store new content here *before* it emits
    /// `file-changed`, and the frontend MUST subscribe before it calls
    /// `get_initial_content`. Together those two orderings close the window in
    /// which a save could be delivered as an event and then overwritten by a
    /// stale snapshot — or dropped entirely because nothing was listening yet.
    /// Swapping either one silently brings the bug back.
    doc: Mutex<Result<Document, String>>,
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
    // Following a link to an image or a PDF opens it in mdview rather than in
    // the shell, so unreadable input is a routine outcome now and not an
    // exceptional one. Naming what is actually wrong beats one blanket message.
    // See docs/adr/0001-links-open-new-window-never-the-shell.md
    if bytes.starts_with(&[0xFF, 0xFE]) || bytes.starts_with(&[0xFE, 0xFF]) {
        return Err(format!(
            "{display} is UTF-16 encoded.\nmdview reads UTF-8 only."
        ));
    }
    if bytes.iter().take(8192).any(|b| *b == 0) {
        return Err(format!("{display} is a binary file, so it cannot be shown."));
    }
    let text = String::from_utf8(bytes).map_err(|_| {
        format!("{display} is not valid UTF-8.\nmdview reads UTF-8 only (Shift_JIS and other encodings are not supported).")
    })?;
    // Windows editors (Notepad, PowerShell) often prepend a UTF-8 BOM,
    // which would otherwise break "#" heading detection at file start.
    Ok(text.strip_prefix('\u{feff}').map(str::to_owned).unwrap_or(text))
}

/// mdview's own output, shown as an ordinary document rather than as an error
/// so that `--help` does not arrive dressed as a failure.
fn notice(markdown_source: &str) -> Document {
    Document {
        file_name: "mdview".into(),
        html: markdown::to_html(markdown_source, None),
    }
}

fn load_document() -> (Result<Document, String>, Option<PathBuf>) {
    let Some(arg) = std::env::args().nth(1) else {
        return (Ok(notice(USAGE)), None);
    };
    match arg.as_str() {
        "--help" | "-h" | "/?" => return (Ok(notice(USAGE)), None),
        "--version" | "-V" => {
            let version = env!("CARGO_PKG_VERSION");
            return (Ok(notice(&format!("# mdview {version}\n"))), None);
        }
        _ => {}
    }
    let path = PathBuf::from(&arg);
    let text = match read_utf8(&path) {
        Ok(t) => t,
        Err(e) => return (Err(e), None),
    };
    // The canonical path is what everything downstream works from: the watcher
    // target, and the directory link and image destinations resolve against.
    let canonical = path.canonicalize().ok();
    let base = canonical.as_deref().and_then(Path::parent);
    let file_name = canonical
        .as_deref()
        .unwrap_or(path.as_path())
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| arg.clone());
    let doc = Document {
        file_name,
        html: markdown::to_html(&text, base),
    };
    (Ok(doc), canonical)
}

#[tauri::command]
fn get_initial_content(state: tauri::State<AppState>) -> Result<Document, String> {
    state.doc.lock().expect("document lock").clone()
}

/// Opens a local destination in a new mdview window.
///
/// The target is always handed to mdview itself and never to the shell, so a
/// document cannot launch an arbitrary program by naming it in a link. The
/// extension is deliberately not consulted.
/// See docs/adr/0001-links-open-new-window-never-the-shell.md
#[tauri::command]
fn follow_link(target: String) -> Result<(), String> {
    let decoded = links::percent_decode(&target).ok_or("malformed target")?;
    let path = PathBuf::from(decoded);
    if links::is_unc(&path) {
        return Err("refusing to open a network path".into());
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    std::process::Command::new(exe)
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Hands a remote destination to whatever the OS has registered for it.
///
/// The scheme is checked here rather than trusted from the caller, so this
/// cannot become a general-purpose launcher. `explorer.exe` receives the URL as
/// a single argument, so nothing inside it is read as shell syntax.
#[tauri::command]
fn open_external(url: String) -> Result<(), String> {
    let allowed = ["http://", "https://", "mailto:"];
    if !allowed.iter().any(|prefix| url.starts_with(prefix)) {
        return Err("unsupported scheme".into());
    }
    std::process::Command::new("explorer.exe")
        .arg(&url)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn mime_for(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    match ext.as_deref() {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        // Safe to serve: an SVG loaded through <img> never runs its scripts.
        Some("svg") => "image/svg+xml",
        Some("bmp") => "image/bmp",
        Some("ico") => "image/x-icon",
        Some("avif") => "image/avif",
        _ => "application/octet-stream",
    }
}

/// Answers one `mdimg` request by reading the file it names.
///
/// Failures are reported as status codes and never as page content: a missing
/// image leaves a broken image where it sat, and the document around it stays
/// readable.
fn serve_asset(encoded: &str) -> tauri::http::Response<Vec<u8>> {
    let empty = |status: u16| {
        tauri::http::Response::builder()
            .status(status)
            .body(Vec::new())
            .expect("static response")
    };
    let Some(decoded) = links::percent_decode(encoded) else {
        return empty(400);
    };
    // Checked again after canonicalising: a junction or symlink part-way along
    // an otherwise ordinary path can still land on a network share.
    let Ok(path) = PathBuf::from(decoded).canonicalize() else {
        return empty(404);
    };
    if links::is_unc(&path) {
        return empty(403);
    }
    match std::fs::read(&path) {
        Ok(bytes) => tauri::http::Response::builder()
            .header("Content-Type", mime_for(&path))
            .body(bytes)
            .expect("asset response"),
        Err(_) => empty(404),
    }
}

/// Refuses every navigation that would leave the app's own origin.
///
/// This is what neuters `<meta http-equiv="refresh">` and
/// `<form action="https://…">`, neither of which CSP can stop: `form-action`
/// does not fall back to `default-src`, and a meta refresh is outside CSP
/// entirely. Deciding to *act* on a destination is not done here — all this
/// hook receives is a URL, with no way to tell a click from an automatic
/// redirect, so acting belongs to the frontend's click handler.
/// See docs/adr/0003-clicks-in-js-navigation-denied-in-rust.md
///
/// It lives in a plugin because the window is declared in `tauri.conf.json`;
/// this is the hook that reaches such a window without moving its creation
/// into Rust.
fn navigation_guard<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("guard")
        .on_navigation(|_webview, url| {
            // `tauri dev` serves the frontend from a local Vite server.
            #[cfg(debug_assertions)]
            if url.host_str() == Some("localhost") {
                return true;
            }
            match url.scheme() {
                "http" | "https" => url.host_str() == Some("tauri.localhost"),
                "tauri" | "about" => true,
                _ => false,
            }
        })
        .build()
}

fn main() {
    let (doc, path) = load_document();
    let title = match &doc {
        Ok(d) => d.file_name.clone(),
        Err(_) => "mdview — error".to_string(),
    };
    tauri::Builder::default()
        .plugin(navigation_guard())
        .register_asynchronous_uri_scheme_protocol("mdimg", |_ctx, request, responder| {
            let encoded = request.uri().path().trim_start_matches('/').to_string();
            // Read off the webview thread so a large image cannot stall the UI.
            std::thread::spawn(move || responder.respond(serve_asset(&encoded)));
        })
        .manage(AppState {
            doc: Mutex::new(doc),
        })
        .invoke_handler(tauri::generate_handler![
            get_initial_content,
            follow_link,
            open_external
        ])
        .setup(move |app| {
            let window = app.get_webview_window("main").expect("main window");
            let _ = window.set_title(&title);
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
    use super::{mime_for, read_utf8};
    use std::path::{Path, PathBuf};

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

    // Following a link to an image is ordinary, so "binary" is named as such
    // rather than reported as an encoding problem.
    #[test]
    fn binary_file_is_named_as_binary() {
        let path = temp_file(
            "mdview-test-binary.png",
            b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0d",
        );
        let err = read_utf8(&path).unwrap_err();
        assert!(err.contains("binary"), "error: {err}");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn utf16_is_named_as_utf16() {
        let path = temp_file("mdview-test-utf16.md", b"\xFF\xFE#\x00 \x00o\x00k\x00");
        let err = read_utf8(&path).unwrap_err();
        assert!(err.contains("UTF-16"), "error: {err}");
        let _ = std::fs::remove_file(&path);
    }

    // Text that is neither binary nor UTF-16 but still not UTF-8 — a Shift_JIS
    // file, for instance — is reported as an encoding problem.
    #[test]
    fn other_encodings_are_reported_as_encoding_errors() {
        // "日本語" in Shift_JIS.
        let path = temp_file("mdview-test-sjis.md", b"\x93\xfa\x96\x7b\x8c\xea\n");
        let err = read_utf8(&path).unwrap_err();
        assert!(err.contains("not valid UTF-8"), "error: {err}");
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

    #[test]
    fn mime_types_cover_the_common_image_formats() {
        assert_eq!(mime_for(Path::new("a.PNG")), "image/png");
        assert_eq!(mime_for(Path::new("a.jpeg")), "image/jpeg");
        assert_eq!(mime_for(Path::new("a.svg")), "image/svg+xml");
        assert_eq!(mime_for(Path::new("a.zip")), "application/octet-stream");
    }
}
