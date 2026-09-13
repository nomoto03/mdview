use std::{
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

use notify::{RecursiveMode, Watcher};
use tauri::{Emitter, Manager};

#[derive(Clone, serde::Serialize)]
struct FileChangedPayload {
    html: String,
}

/// How long the file has to stay gone before its absence counts as a deletion.
///
/// A failed read is ambiguous: an atomic save (write a temporary file, rename
/// it over the original) leaves the document briefly missing, and so does an
/// actual deletion. Nothing distinguishes them at the moment they happen, so
/// the only way to tell is to wait and look again. Too short and an ordinary
/// save flashes a warning; too long and a real deletion goes unreported.
const DELETION_GRACE: Duration = Duration::from_secs(1);

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
                    match crate::read_utf8(&path) {
                        Ok(text) => publish(&app, &path, text),
                        Err(_) => {
                            std::thread::sleep(DELETION_GRACE);
                            match crate::read_utf8(&path) {
                                // The save had simply not landed yet.
                                Ok(text) => publish(&app, &path, text),
                                // Still unreadable, so say so — but leave the
                                // content alone. Losing what you were reading
                                // because a branch switch moved the file is
                                // worse than showing something slightly stale.
                                Err(_) => {
                                    let _ = app.emit("file-missing", ());
                                }
                            }
                        }
                    }
                }
                Ok(Err(_)) => continue,
                Err(_) => break,
            }
        }
    });
}

/// Stores the new content, then announces it.
///
/// The order is load-bearing and must not be swapped: `get_initial_content`
/// hands back whatever is stored, and the frontend subscribes before it calls
/// that command. Updating first means an event can only ever be followed by the
/// same content or newer, never by a stale snapshot. See `AppState::doc`.
fn publish(app: &tauri::AppHandle, path: &Path, text: String) {
    let html = crate::markdown::to_html(&text, path.parent());
    if let Some(state) = app.try_state::<crate::AppState>() {
        if let Ok(mut stored) = state.doc.lock() {
            if let Ok(doc) = stored.as_mut() {
                doc.html = html.clone();
            }
        }
    }
    let _ = app.emit("file-changed", FileChangedPayload { html });
}
