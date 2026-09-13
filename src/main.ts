import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface Doc {
  file_name: string;
  html: string;
}

interface FileChangedPayload {
  html: string;
}

const OPEN_SCHEME = "mdview-open:";
const EXTERNAL_PROTOCOLS = ["http:", "https:", "mailto:"];

const filename = document.getElementById("filename")!;
const status = document.getElementById("status")!;
const content = document.getElementById("content")!;

function render(html: string) {
  const y = window.scrollY;
  // Trusted: HTML is produced by our own Rust converter from a local file.
  content.innerHTML = html;
  requestAnimationFrame(() => window.scrollTo(0, y));
}

function showError(message: string) {
  filename.textContent = "Error";
  document.title = "mdview — error";
  content.innerHTML = "";
  const pre = document.createElement("pre");
  pre.className = "error";
  pre.textContent = message;
  content.appendChild(pre);
}

// Only a real click may launch anything.
//
// The Rust navigation handler refuses every destination outside the app's own
// origin, which is what stops <meta http-equiv="refresh"> and <form action>
// from acting on their own. It cannot make this decision instead, because all
// it receives is a URL — it has no way to tell a click from an automatic
// redirect. A click event *is* the evidence that a person asked.
// See docs/adr/0003-clicks-in-js-navigation-denied-in-rust.md
content.addEventListener("click", (event) => {
  const target = event.target;
  if (!(target instanceof Element)) return;
  const anchor = target.closest("a");
  if (!anchor) return;

  const href = anchor.getAttribute("href") ?? "";
  // In-page anchors stay inside the document; nothing to intercept.
  if (href.startsWith("#")) return;

  event.preventDefault();

  if (href.startsWith(OPEN_SCHEME)) {
    void invoke("follow_link", { target: href.slice(OPEN_SCHEME.length) });
    return;
  }

  let resolved: URL;
  try {
    resolved = new URL(href, location.href);
  } catch {
    return;
  }
  if (EXTERNAL_PROTOCOLS.includes(resolved.protocol)) {
    void invoke("open_external", { url: resolved.href });
  }
  // Anything else is deliberately inert.
});

async function init() {
  try {
    // Subscribe before asking for the content, not after.
    //
    // The watcher stores new content before it emits, so with this order a save
    // arriving mid-startup is either already included in what the command
    // returns, or delivered as an event — never dropped for want of a listener,
    // and never overwritten by an older snapshot. Reversing these two calls
    // silently reopens that gap.
    await listen<FileChangedPayload>("file-changed", (e) => {
      status.textContent = "";
      render(e.payload.html);
    });
    await listen("file-missing", () => {
      status.textContent = "⚠ ファイルが削除されました";
    });

    const doc = await invoke<Doc>("get_initial_content");
    filename.textContent = doc.file_name;
    render(doc.html);
  } catch (err) {
    showError(String(err));
  }
}

init();
