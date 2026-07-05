use std::{path::PathBuf, sync::mpsc, time::Duration};

use notify::{RecursiveMode, Watcher};
use tauri::Emitter;

#[derive(Clone, serde::Serialize)]
struct FileChangedPayload {
    html: String,
}

// Watches the parent directory (not the file itself) so that editors
// doing atomic saves (write temp file + rename over the original) are
// still detected after the original inode is replaced.
pub fn spawn(app: tauri::AppHandle, path: PathBuf) {
    std::thread::spawn(move || {
        let parent = match path.parent() {
            Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
            _ => PathBuf::from("."),
        };
        let target_name = match path.file_name() {
            Some(n) => n.to_os_string(),
            None => return,
        };

        let (tx, rx) = mpsc::channel();
        let mut watcher = match notify::recommended_watcher(tx) {
            Ok(w) => w,
            Err(_) => return,
        };
        if watcher.watch(&parent, RecursiveMode::NonRecursive).is_err() {
            return;
        }

        loop {
            match rx.recv() {
                Ok(Ok(event)) => {
                    if !event
                        .paths
                        .iter()
                        .any(|p| p.file_name() == Some(target_name.as_os_str()))
                    {
                        continue;
                    }
                    // Debounce: swallow follow-up events until 300ms of silence.
                    while rx.recv_timeout(Duration::from_millis(300)).is_ok() {}
                    // Read failures are mid-save transients; the next event retries.
                    if let Ok(text) = crate::read_utf8(&path) {
                        let _ = app.emit(
                            "file-changed",
                            FileChangedPayload {
                                html: crate::markdown::to_html(&text),
                            },
                        );
                    }
                }
                Ok(Err(_)) => continue,
                Err(_) => break,
            }
        }
    });
}
