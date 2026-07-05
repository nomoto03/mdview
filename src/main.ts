import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface Doc {
  file_name: string;
  html: string;
}

interface FileChangedPayload {
  html: string;
}

const header = document.getElementById("filename")!;
const content = document.getElementById("content")!;

function render(html: string) {
  const y = window.scrollY;
  // Trusted: HTML is produced by our own Rust converter from a local file.
  content.innerHTML = html;
  requestAnimationFrame(() => window.scrollTo(0, y));
}

function showError(message: string) {
  header.textContent = "Error";
  document.title = "mdview — error";
  content.innerHTML = "";
  const pre = document.createElement("pre");
  pre.className = "error";
  pre.textContent = message;
  content.appendChild(pre);
}

async function init() {
  try {
    const doc = await invoke<Doc>("get_initial_content");
    header.textContent = doc.file_name;
    render(doc.html);
    await listen<FileChangedPayload>("file-changed", (e) => render(e.payload.html));
  } catch (err) {
    showError(String(err));
  }
}

init();
