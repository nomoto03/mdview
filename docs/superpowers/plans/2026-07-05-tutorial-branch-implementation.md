# Rust/TypeScript/Tauri 学習教材化ブランチ Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `mdview`の実コードを教材に、Rust/TypeScript/Tauriが初めてのエンジニア向けに、解説コメント・設定ファイル解説・`LEARNING.md`・`exercises/`(基礎kata+発展課題+解答例)を追加した`tutorial`ブランチ(独立worktree)を作る。

**Architecture:** 新規worktree(`../markdown-viewer-tutorial`, `tutorial`ブランチ)上で、既存の実ファイル(`main.rs`/`markdown.rs`/`watcher.rs`/`main.ts`/`style.css`/設定ファイル)にコメント・解説を追加し、`exercises/`配下に独立したRust kataパッケージとTypeScript kata、発展課題の課題文・解答パッチを追加する。アプリの実行時の挙動は一切変えない。

**Tech Stack:** Rust 1.96 / cargo, TypeScript 5.6 / Vite 6 / Node 22, Tauri v2, pulldown-cmark, notify

**設計spec:** `docs/superpowers/specs/2026-07-05-tutorial-branch-design.md`

## Global Constraints

- アプリの挙動・ロジックは一切変更しない(コメント・ドキュメント・`exercises/`の追加のみ)。挙動が変わっていないことは各タスクで`cargo test`/`npm run build`の成功で確認する
- `tutorial`ブランチは`master`へのマージを前提としない恒久的な学習用ブランチ
- 新規の実行時依存ライブラリは追加しない。教材(`exercises/`)側も`cargo test`と素の`node`実行のみで完結させ、テストフレームワーク等は追加しない
- 対象読者は他言語でのプログラミング経験はあるが、Rust/TypeScript/Tauriが初めてのエンジニア。「変数とは」レベルの説明は書かず、各言語・フレームワークならではの考え方に絞る
- コメント方針: 同じ概念が複数ファイルに登場する場合、最初に登場した箇所で手厚く解説し、以降は簡潔な参照コメントに留める
- JSON設定ファイル(`tauri.conf.json`, `package.json`, `tsconfig.json`, `src-tauri/capabilities/default.json`)はコメントを書けないため、`<元のファイル名>.explained.md`という対になる解説Markdownを同じ場所に置く。TOML(`Cargo.toml`)は`#`コメントを直接書く
- **以降のすべてのタスクのファイルパスは、Task 1で作成する`../markdown-viewer-tutorial`(worktree)を作業ディレクトリの基準とする。**

---

### Task 1: worktreeの作成とカリキュラム索引の設置

**Files:**
- Create (worktree): `../markdown-viewer-tutorial`(`tutorial`ブランチ)
- Create: `LEARNING.md`
- Create: `exercises/README.md`

**Interfaces:**
- Produces: `tutorial`ブランチのworktree。以降の全タスクはこのディレクトリ内で作業する
- Produces: `LEARNING.md`が参照する構造(`exercises/basic/rust`, `exercises/basic/typescript`, `exercises/advanced/01〜05`, `*.explained.md`)は後続タスクで実際に作成される

- [ ] **Step 1: worktreeとブランチを作成する**

現在の作業ディレクトリ(`master`ブランチの`markdown-viewer`)から実行する。

```bash
git worktree add ../markdown-viewer-tutorial -b tutorial
```

- [ ] **Step 2: worktreeが作成されたことを確認する**

```bash
git worktree list
```

Expected: `../markdown-viewer-tutorial`が`tutorial`ブランチとして一覧に表示される。

- [ ] **Step 3: 以降の作業ディレクトリを移動する**

```bash
cd ../markdown-viewer-tutorial
```

- [ ] **Step 4: `LEARNING.md`を作成する**

`LEARNING.md`(worktreeのルート)に以下の内容で新規作成する。

```markdown
# LEARNING.md — mdviewで学ぶ Rust / TypeScript / Tauri

このブランチ(`tutorial`)は、`mdview`(Windows向け軽量Markdownビューア)の
実際に動くコードを教材にして、Rust・TypeScript・Tauriを学ぶための
学習用ブランチです。アプリの挙動は`master`ブランチと同じで、
コードへの解説コメントと`exercises/`フォルダだけが追加されています。

対象読者: 他言語でのプログラミング経験はあるが、Rust/TypeScript/Tauriは
これから、という方。「変数とは」レベルの説明は省き、この3つの技術
ならではの考え方(所有権、`Result`/`Option`、非同期処理、Tauriの
コマンド/イベントモデルなど)に絞って解説しています。

## 読み進める順番

### 0. 環境構築

`README.md`の「前提条件」「開発ビルド」を参照し、以下を確認してください。

- [ ] `rustc --version` / `cargo --version` が表示される(rustup経由でインストール)
- [ ] `node --version` が18以上
- [ ] `npm install` が通る
- [ ] `npm run tauri dev -- -- -- README.md` でウィンドウが開き、このファイルが表示される

### 1. アーキテクチャの全体像

`mdview`は3層構造です。

```
CLI引数 (main.rs)
   ↓
ファイル読み込み・検証 (main.rs: read_utf8)
   ↓
Markdown→HTML変換 (markdown.rs)
   ↓
Tauriのコマンド機構でフロントエンドへ (main.rs: get_initial_content)
   ↓
画面に描画 (main.ts)

並行して: ファイル監視 (watcher.rs) → 変更があればHTML再生成 →
          イベントでフロントエンドへ通知 (main.ts: listen)
```

「Rust側で全部HTMLにしてしまい、フロントエンドは表示するだけ」という
のがこのアプリの設計上のポイントです。フロントエンドにMarkdownパーサを
持たせない分、JSの依存が減り起動が軽くなります。

### 2. Rustのコードを読む(この順番がおすすめ)

1. `src-tauri/src/markdown.rs` — 一番シンプルで依存もないので最初に。
   Rustの関数・借用(`&str`)・`Options`のビルダー的な使い方・
   テストの書き方が学べる
2. `src-tauri/src/main.rs` — `Result`と`?`演算子、パターンマッチ、
   Tauriの`State`・`#[tauri::command]`・`setup`フックが学べる
3. `src-tauri/src/watcher.rs` — スレッド・チャネル(`mpsc`)・所有権が
   スレッド境界をどう移動するか(`move`クロージャ)が学べる

### 3. TypeScript / フロントエンドを読む

- `src/main.ts` — `async`/`await`、Tauriの`invoke`/`listen`、DOM操作
- `src/style.css` — 各ルールに「何のためか」のコメントを添えています

### 4. 設定ファイルを読む

JSONはコメントが書けないので、対になる`*.explained.md`を用意しています。

| 設定ファイル | 解説 |
| --- | --- |
| `src-tauri/Cargo.toml` | ファイル内に直接`#`コメントで解説(TOMLはコメント可) |
| `src-tauri/tauri.conf.json` | `src-tauri/tauri.conf.json.explained.md` |
| `package.json` | `package.json.explained.md` |
| `tsconfig.json` | `tsconfig.json.explained.md` |
| `src-tauri/capabilities/default.json` | `src-tauri/capabilities/default.json.explained.md` |

### 5. テストの考え方(TDD)

このリポジトリの実コードは「失敗するテストを先に書く→実装する→通す」
というTDD(テスト駆動開発)で作られました。`markdown.rs`と`main.rs`の
`#[cfg(test)] mod tests`にその考え方のコメントを添えています。

読んだら、`exercises/basic/rust`で自分でもRED→GREENのループを
体験してみてください。

### 6. 課題に挑戦する

`exercises/README.md`を参照してください。

## このブランチについて

- `master`ブランチとアプリの挙動は完全に同一です(コメント・ドキュメント・
  `exercises/`の追加のみ)
- `master`へのマージは想定していません。学習用に独立して育てていく
  ブランチです
```

- [ ] **Step 5: `exercises/README.md`を作成する**

`exercises/`ディレクトリを作り、その中に`README.md`を以下の内容で作成する。

```markdown
# 課題

`mdview`のコードを読んだら、実際に手を動かして理解を定着させましょう。

## 進め方

1. `basic/`から順に、Rust→TypeScriptの順で解く(基礎の穴埋め)
2. 余裕があれば`advanced/`の発展課題に挑戦する(実アプリの拡張)
3. 詰まったら`answers/`を見る(先に見ずに、まず自分で考えてみてください)

## 難易度表

| フォルダ | 内容 | 難易度 | 前提 |
| --- | --- | --- | --- |
| `basic/rust/` | Rustの基礎文法を穴埋めで習得(4問) | ★☆☆ | なし |
| `basic/typescript/` | TypeScriptの基礎文法を穴埋めで習得(2問) | ★☆☆ | なし |
| `advanced/01-line-numbers` | コードブロックに行番号を表示 | ★★☆ | `markdown.rs`を読んでいること |
| `advanced/02-dark-mode` | ダークモード対応 | ★☆☆ | `style.css`を読んでいること |
| `advanced/03-print` | 印刷対応 | ★★☆ | `main.ts`/`style.css`を読んでいること |
| `advanced/04-html-export` | HTML出力機能の追加 | ★★★ | `main.rs`のTauriコマンドを理解していること |
| `advanced/05-context-menu` | エクスプローラー右クリック連携 | ★★☆ | Windowsレジストリの基礎知識(課題内で解説) |

## basic の解き方

### Rust

```powershell
cd exercises/basic/rust
cargo test
```

最初は`todo!()`があるテストが失敗します(RED)。`src/kataNN_*.rs`内の
`todo!()`を実装で置き換え、`cargo test`が全て通れば(GREEN)クリアです。

### TypeScript

```powershell
node --experimental-strip-types exercises/basic/typescript/kata01_async.ts
```

Node.js 22は`--experimental-strip-types`フラグで`.ts`ファイルの型注釈を
読み飛ばして直接実行できます。ファイル内の`// TODO`や`throw new Error("not implemented")`
の部分を実装し、コメントに書かれた期待する出力とコンソール出力が
一致すればクリアです。

## advanced の解き方

各`advanced/0X-xxx/README.md`に課題文とヒントがあります。
`../../src`や`../../src-tauri/src`の実ファイルを直接編集して機能を
追加してください。

## 解答例

`answers/basic/`に穴埋めの完成版、`answers/advanced/`に発展課題の
参考実装(パッチファイル形式)があります。自分の実装と見比べる、
あるいは詰まったときのヒントとして使ってください。
```

- [ ] **Step 6: コミットする**

```bash
git add LEARNING.md exercises/README.md
git commit -m "docs: LEARNING.mdとexercises/README.mdを追加"
```

---

### Task 2: `markdown.rs`へのコメント追加

**Files:**
- Modify: `src-tauri/src/markdown.rs`

**Interfaces:**
- Consumes: なし(このファイルは他の実装に依存しない)
- Produces: `to_html(source: &str) -> String`(シグネチャ・挙動は変更なし)

- [ ] **Step 1: ファイル全体を以下の内容に置き換える**

```rust
//! Markdown → HTML 変換ロジック。
//!
//! ファイル冒頭の `//!` はモジュール(このファイル自体)に対するドキュメン
//! テーションコメントです。関数やstructの直前に書く `///` とは違い、
//! 「このファイル全体が何をするものか」を説明するのに使います。

use pulldown_cmark::{html, Options, Parser};

// GFM bare-URL autolinking is not supported by pulldown-cmark;
// CommonMark angle-bracket autolinks (<https://…>) still work.

/// Markdownのソース文字列をHTML文字列に変換する。
///
/// 引数`source`の型は`&str`(文字列スライスへの参照)であって`String`
/// (所有権を持つ文字列)ではない点に注目してください。Rustでは
/// 「読むだけで書き換えない」引数は`&str`で受け取るのが基本です。
/// こうすると、呼び出し側は文字列の所有権を手放さずに済み(コピーも
/// 発生しない)、この関数はただ「借りて」読むだけになります。
/// C++でいう`const std::string&`、Javaなら単なる`String`引数に近い
/// 感覚ですが、Rustではこの「借用(borrow)」がコンパイラによって
/// 静的にチェックされるのが大きな違いです。
pub fn to_html(source: &str) -> String {
    // `Options`はビットフラグの集まりです。`empty()`で「何も有効化
    // しない」状態を作り、`insert`で1つずつオンにしていきます。これは
    // C言語でいう`flags |= FLAG_A | FLAG_B`をタイプセーフにしたもの、
    // と考えるとイメージしやすいです。
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    // `Parser`はMarkdownを「イベントの列」(見出し開始、テキスト、
    // 見出し終了…)として解釈するイテレータです。すぐには文字列を
    // 生成せず、後段の`html::push_html`が呼ばれて初めて実際に処理が
    // 走ります(Rustのイテレータは遅延評価)。
    let parser = Parser::new_ext(source, opts);

    // `String::with_capacity`で最初からある程度の容量を確保しておくと、
    // 文字列を追記するたびに内部バッファを再確保するコストを減らせ
    // ます。HTMLはMarkdown原文よりおおよそ長くなる傾向があるので
    // `* 3 / 2`(1.5倍)を目安に確保しています。これはあくまで
    // パフォーマンス上の最適化であり、無くても正しく動きます。
    let mut out = String::with_capacity(source.len() * 3 / 2);

    // `push_html`はイベント列を消費しながら`out`に書き込んでいきます。
    // 引数`out`は`&mut String`(可変の借用)として渡されるので、
    // この関数を抜けたあとも`out`の所有権はここ(`to_html`)のまま
    // であり続けます。
    html::push_html(&mut out, parser);
    out
}

#[cfg(test)]
mod tests {
    use super::to_html;

    // それぞれのテストは「GFM(GitHub Flavored Markdown)のこの機能が
    // 正しくHTMLに変換されるか」を1つずつ確認しています。実装時は
    // これらを先に書いて失敗させ(RED)、そのあとpulldown-cmarkの
    // オプションを1つずつ有効にして通す(GREEN)、というTDDの
    // 順番で作られました。

    #[test]
    fn table_renders() {
        let html = to_html("|a|b|\n|-|-|\n|1|2|");
        assert!(html.contains("<table>"), "output: {html}");
        assert!(html.contains("<th>a</th>"), "output: {html}");
        assert!(html.contains("<td>1</td>"), "output: {html}");
    }

    #[test]
    fn task_list_renders_checkboxes() {
        let html = to_html("- [x] done\n- [ ] todo");
        assert_eq!(html.matches("type=\"checkbox\"").count(), 2, "output: {html}");
        assert_eq!(html.matches("checked").count(), 1, "output: {html}");
        assert!(html.contains("disabled"), "output: {html}");
    }

    #[test]
    fn code_block_plain() {
        // シンタックスハイライトは意図的に非対応。<span class="...">が
        // 出てこないことまで確認し、「将来ハイライトを足したくなったら
        // このテストが真っ先に落ちる」ようにしています。
        let html = to_html("```rust\nfn main() {}\n```");
        assert!(html.contains("<pre><code"), "output: {html}");
        assert!(!html.contains("<span class="), "output: {html}");
    }

    #[test]
    fn strikethrough_renders() {
        let html = to_html("~~gone~~");
        assert!(html.contains("<del>gone</del>"), "output: {html}");
    }

    #[test]
    fn japanese_passthrough() {
        // Rustの文字列はUTF-8前提なので、日本語を含む文字列も特別な
        // 変換なしにそのまま扱えます。
        let html = to_html("# 見出し\n\n日本語の段落です。");
        assert!(html.contains("<h1>見出し</h1>"), "output: {html}");
        assert!(html.contains("日本語の段落です。"), "output: {html}");
    }

    // Raw HTML passes through untouched: input files are trusted local
    // documents and script execution is blocked by the WebView CSP instead.
    #[test]
    fn raw_html_passthrough() {
        let html = to_html("<b>bold</b>");
        assert!(html.contains("<b>bold</b>"), "output: {html}");
    }
}
```

- [ ] **Step 2: テストを実行して挙動が変わっていないことを確認する**

```bash
cargo test --manifest-path src-tauri/Cargo.toml markdown::
```

Expected: `test result: ok. 6 passed`

- [ ] **Step 3: コミットする**

```bash
git add src-tauri/src/markdown.rs
git commit -m "docs: markdown.rsに解説コメントを追加"
```

---

### Task 3: `main.rs`へのコメント追加

**Files:**
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Consumes: `markdown::to_html(&str) -> String`(Task 2)
- Produces: `read_utf8(&Path) -> Result<String, String>`, `AppState`, `get_initial_content`(シグネチャ・挙動は変更なし)

- [ ] **Step 1: ファイル全体を以下の内容に置き換える**

```rust
//! mdviewのエントリポイント。
//!
//! CLI引数を読み取り → ファイルを検証・読み込み → Markdownを変換 →
//! Tauriアプリを起動、という一連の流れをこのファイルが担います。
//! 読む前に`markdown.rs`を先に読んでおくことを推奨します(このファイル
//! で使っている`Result`・パターンマッチ・Tauri特有の概念を、より
//! シンプルなコードで先に慣らしておくためです)。

// `windows_subsystem = "windows"`を指定すると、Windows上でこの実行
// ファイルを起動したときにコンソールウィンドウ(黒い画面)が付随しなく
// なります。ただし開発中(デバッグビルド)はこれを付けると`println!`
// によるデバッグ出力が見えなくなり不便なので、`cfg_attr`で「リリース
// ビルドのときだけ」この属性を有効にしています。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// `mod`宣言は「同じディレクトリの`markdown.rs`/`watcher.rs`という
// ファイルを、このクレートの一部として読み込む」という意味です。
// JavaScriptの`import`やPythonの`import`に近いですが、Rustでは
// ファイルパスではなく「モジュールツリー」の構築として扱われます。
mod markdown;
mod watcher;

use std::path::{Path, PathBuf};
use tauri::Manager;

// `#[derive(...)]`は、指定したトレイト(≒インターフェース)の実装を
// コンパイラに自動生成させる仕組みです。
// - `Clone`: このstructの値を複製(`.clone()`)できるようにする
// - `serde::Serialize`: このstructをJSONなどにシリアライズできる
//   ようにする(TauriがフロントエンドへRustの値を渡すときに使う)
/// フロントエンドに渡す「表示するドキュメント」を表す構造体。
#[derive(Clone, serde::Serialize)]
struct Document {
    file_name: String,
    html: String,
}

/// アプリ全体で共有する状態。Tauriの`.manage()`に渡すことで、
/// どのコマンド・イベントハンドラからも`tauri::State<AppState>`
/// という形で参照できるようになる(依存性注入に近い仕組み)。
struct AppState {
    // `Result<Document, String>`は「成功したら`Document`を、失敗したら
    // 理由を表す`String`を持つ」という意味の型です。Rustには例外
    // (exception)がなく、失敗しうる処理の戻り値は`Result`で表現する
    // のが基本です。ここでは「起動時にファイルの読み込みに成功した
    // か」をそのままウィンドウが表示すべき状態として保持しています。
    doc: Result<Document, String>,
    // `Option<PathBuf>`は「値があるかもしれないし、ないかもしれない」
    // ことを表す型です。`null`や`None`相当の概念を型システムに
    // 組み込んだもので、「うっかりnullを踏む」バグをコンパイル時に
    // 防げます。ここでは「監視すべきファイルパスがあるなら`Some`、
    // 読み込みに失敗して監視のしようがないなら`None`」を表します。
    path: Option<PathBuf>,
}

/// 指定したパスのファイルをUTF-8テキストとして読み込む。
///
/// 「ファイルが存在しない」と「存在するがUTF-8として不正」を区別して
/// エラーメッセージを返す。`env::args()`(コマンドライン引数)に依存しない
/// 純粋関数として切り出しているので、`cargo test`から直接呼び出して
/// 検証できる。
fn read_utf8(path: &Path) -> Result<String, String> {
    let display = path.display();

    // `match`はパターンマッチで、値の「形」に応じて分岐します。
    // if/elseの強化版のようなものですが、Rustの`match`は「すべての
    // パターンを網羅しているか」をコンパイラがチェックする点が
    // 大きく違います(1つでも書き漏らすとコンパイルエラー)。
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        // `if`付きのパターン(マッチガード)で、「NotFoundエラーの
        // 場合だけ」この腕にマッチさせています。
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(format!("File not found: {display}"));
        }
        Err(e) => return Err(format!("Cannot read {display}: {e}")),
    };

    // `String::from_utf8`はバイト列(`Vec<u8>`)をUTF-8文字列として
    // 検証しながら変換します。不正なバイト列なら`Err`を返す。
    // `.map_err(...)`は「`Err`の中身だけを変換する」ためのメソッドで、
    // ここでは`FromUtf8Error`という専用のエラー型を、アプリ全体で
    // 統一して使っている`String`エラーに変換しています。
    //
    // 行末の`?`演算子は、「この式が`Err`だったら、その`Err`を
    // このままこの関数の戻り値として即`return`する。`Ok`だったら
    // 中身を取り出して続行する」という糖衣構文(シンタックスシュガー)
    // です。`match`で毎回書くと長くなる「成功したら続ける、失敗したら
    // 打ち切って呼び出し元に伝播する」という定型パターンを1文字で
    // 表現できます。
    let text =
        String::from_utf8(bytes).map_err(|_| format!("{display} is not valid UTF-8."))?;

    // Windows editors (Notepad, PowerShell) often prepend a UTF-8 BOM,
    // which would otherwise break "#" heading detection at file start.
    Ok(text.strip_prefix('\u{feff}').map(str::to_owned).unwrap_or(text))
}

/// CLI引数からファイルを読み込み、`Document`と監視すべきパスを組み立てる。
///
/// 戻り値がタプル`(Result<Document, String>, Option<PathBuf>)`に
/// なっているのは、「表示すべき内容(またはエラー)」と「監視できる
/// パスがあるか」が独立した情報だからです。読み込みに失敗した場合は
/// `path`は`None`になり、`watcher::spawn`は呼ばれません(存在しない
/// ファイルを監視することはできないため)。
fn load_document() -> (Result<Document, String>, Option<PathBuf>) {
    // `let ... else`は「パターンにマッチしなかったら即座に関数を
    // 抜ける」ための構文です。ここでは「2番目の引数(`nth(1)`。
    // 0番目は実行ファイル自身のパス)が存在すれば`arg`に束縛し、
    // 存在しなければUsageエラーを返して打ち切る」という意味になり
    // ます。
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
        // `.map(...)`は`Option`の中身がある場合だけクロージャを適用する
        // メソッドです(値がなければ何もせず`None`のまま)。
        .map(|n| n.to_string_lossy().into_owned())
        // `.unwrap_or_else(...)`は`None`だった場合のフォールバック値を
        // 計算します(引数なしの`unwrap_or`と違い、値を「必要になった
        // ときだけ」計算する遅延評価版です)。
        .unwrap_or_else(|| arg.clone());

    let doc = Document {
        file_name,
        html: markdown::to_html(&text),
    };

    // `canonicalize()`はシンボリックリンクや`..`を解決した絶対パスを
    // 返します。ファイル監視(`watcher.rs`)には解決済みの実パスを渡す
    // 必要があるため、ここで変換しています。失敗する可能性がある処理
    // (`Result`を返す)ですが、監視できなくてもアプリ自体は動作を続けて
    // よいので`.ok()`で`Result`を`Option`に変換し、エラーは握りつぶして
    // います。
    (Ok(doc), path.canonicalize().ok())
}

// `#[tauri::command]`を付けた関数は、フロントエンド(TypeScript側)から
// `invoke("get_initial_content")`という形で呼び出せるようになります。
// Tauriがこのマクロを展開して、JSON⇄Rustの値の変換やスレッドをまたぐ
// 呼び出しの配線を自動生成してくれます。
/// フロントエンドが起動直後に呼ぶコマンド。読み込み済みの`Document`
/// (またはエラー)をそのまま返す「pull型」の設計。
///
/// setupフックの中で`app.emit(...)`する「push型」にしなかったのは、
/// フロントエンドのイベントリスナーが登録される前にemitが実行される
/// と、そのイベントが失われてしまう競合(レース)があるためです。
/// フロントエンドが「用意ができてから自分で取りに行く」pull型なら、
/// この種の取りこぼしが原理的に起こりません。
#[tauri::command]
fn get_initial_content(state: tauri::State<AppState>) -> Result<Document, String> {
    // `state`は`&AppState`(共有参照)なので中身を直接moveできません。
    // `Document`が`Clone`を実装しているおかげで`.clone()`で複製を
    // 作り、それを返すことで所有権の問題を回避しています。
    state.doc.clone()
}

fn main() {
    let (doc, path) = load_document();

    // Tauriアプリはビルダーパターンで組み立てます。`.manage(...)`で
    // 状態を登録し、`.invoke_handler(...)`でフロントエンドから呼べる
    // コマンドを登録し、`.setup(...)`で起動直後に一度だけ実行する
    // 処理を登録して、最後に`.run(...)`で実際にイベントループを
    // 開始する、という流れです。
    tauri::Builder::default()
        .manage(AppState {
            doc: doc.clone(),
            path: path.clone(),
        })
        .invoke_handler(tauri::generate_handler![get_initial_content])
        // `setup`に渡しているのはクロージャです。`move`を付けているのは、
        // このクロージャが外側の`doc`と`path`の所有権を「奪って」
        // 自分の中に持ち込むという意味です。`move`がないと、クロージャは
        // 変数を借用しようとしますが、`doc`と`path`は`main`関数の中で
        // このあと使われないため、所有権ごと移動させる方が素直です。
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

    // テスト用の一時ファイルを作るヘルパー。`std::env::temp_dir()`が
    // 返すOS標準の一時フォルダ(WindowsならAppData\Local\Temp)に書き
    // 込むことで、実際のファイルシステムを相手にした「本物の」I/O
    // テストにしています。
    fn temp_file(name: &str, bytes: &[u8]) -> PathBuf {
        let path = std::env::temp_dir().join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    }

    // 「存在しないファイル」ケース。TDDではこのテストを最初に書いて
    // 失敗させ、そのあと本実装を書いて通しました。
    #[test]
    fn missing_file_error() {
        let path = std::env::temp_dir().join("mdview-nonexistent-xyz.md");
        let err = read_utf8(&path).unwrap_err();
        assert!(err.contains("not found"), "error: {err}");
    }

    // 「存在はするがUTF-8として不正なバイト列」ケース。
    #[test]
    fn invalid_utf8_error() {
        let path = temp_file("mdview-test-invalid.md", &[0xFF, 0xFE, 0x80]);
        let err = read_utf8(&path).unwrap_err();
        assert!(err.contains("UTF-8"), "error: {err}");
        let _ = std::fs::remove_file(&path);
    }

    // 実機での動作確認中に見つかった不具合の再発防止テスト。
    // PowerShellやメモ帳が付与するUTF-8 BOM(先頭の3バイト)が残った
    // ままだと、"# 見出し"のようなMarkdown見出しの`#`が認識されない
    // 問題があった。
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
```

- [ ] **Step 2: テストを実行して挙動が変わっていないことを確認する**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: `test result: ok. 10 passed`

- [ ] **Step 3: コミットする**

```bash
git add src-tauri/src/main.rs
git commit -m "docs: main.rsに解説コメントを追加"
```

---

### Task 4: `watcher.rs`へのコメント追加

**Files:**
- Modify: `src-tauri/src/watcher.rs`

**Interfaces:**
- Consumes: `crate::read_utf8`(Task 3), `crate::markdown::to_html`(Task 2)
- Produces: `spawn(app: tauri::AppHandle, path: PathBuf)`(シグネチャ・挙動は変更なし)

- [ ] **Step 1: ファイル全体を以下の内容に置き換える**

```rust
//! ファイル監視ロジック。
//!
//! 表示中のファイルが外部から書き換えられたら検知し、変換し直した
//! HTMLをイベントでフロントエンドに届ける。監視はメインスレッドを
//! ブロックしないよう別スレッドで行う。

use std::{path::PathBuf, sync::mpsc, time::Duration};

use notify::{RecursiveMode, Watcher};
use tauri::Emitter;

/// フロントエンドへ送るイベントのペイロード(中身)。
#[derive(Clone, serde::Serialize)]
struct FileChangedPayload {
    html: String,
}

// Watches the parent directory (not the file itself) so that editors
// doing atomic saves (write temp file + rename over the original) are
// still detected after the original inode is replaced.

/// ファイル監視を別スレッドで開始する。呼び出し元(`main.rs`)は
/// この関数からすぐに戻り、監視はバックグラウンドで動き続ける。
pub fn spawn(app: tauri::AppHandle, path: PathBuf) {
    // `std::thread::spawn`はOSスレッドを1つ新たに起動し、渡した
    // クロージャをそのスレッド上で実行します。`move`が付いているのは、
    // このクロージャが`app`と`path`の所有権を新しいスレッドに完全に
    // 移動させるためです。Rustでは「同じ値を複数のスレッドが同時に
    // 書き換えられる」状態を型システムが許さないので、こうして
    // 所有権ごと渡すか、あとで出てくる`Arc`/`Mutex`のような共有の
    // 仕組みを使うかのどちらかになります。ここでは片方向にしか
    // 値を渡さないので、シンプルに`move`で十分です。
    std::thread::spawn(move || {
        let parent = match path.parent() {
            Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
            _ => PathBuf::from("."),
        };
        let target_name = match path.file_name() {
            Some(n) => n.to_os_string(),
            None => return,
        };

        // `mpsc::channel()`は「Multi-Producer, Single-Consumer」
        // (複数の送信者・単一の受信者)のキューを作ります。`tx`
        // (transmitter、送信側)はファイル監視ライブラリ(`notify`)に
        // 渡し、`rx`(receiver、受信側)はこのスレッド自身が使って
        // イベントを1つずつ受け取ります。スレッド間でデータを
        // やり取りする、Rustでは定番の方法です。
        let (tx, rx) = mpsc::channel();

        let mut watcher = match notify::recommended_watcher(tx) {
            Ok(w) => w,
            Err(_) => return,
        };
        if watcher.watch(&parent, RecursiveMode::NonRecursive).is_err() {
            return;
        }

        // 無限ループで、ファイルシステムのイベントが届くたびに処理を
        // 行います。`rx.recv()`はイベントが来るまでこのスレッドを
        // ブロックする(待機する)ので、CPUを無駄に消費しません。
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
                    //
                    // エディタの保存は「一時ファイルを書く」「リネーム
                    // する」など複数のファイルシステムイベントを短時間に
                    // 連続して発生させることがあります。そのたびに
                    // 再描画すると無駄が多いので、300ms以内に次の
                    // イベントが来る限り待ち続け、イベントが止んで
                    // から初めて再読み込みします。
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
                // `rx.recv()`が`Err`を返すのは、送信側(`tx`、および
                // それを内部に持つ`watcher`)が破棄されてチャネルが
                // 閉じられたときです。ここに来ることは通常ないはず
                // ですが、来た場合はループを終了してスレッドを
                // きれいに終わらせます。
                Err(_) => break,
            }
        }
    });
}
```

- [ ] **Step 2: ビルドが通ることを確認する**

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

Expected: エラーなくビルドが完了する(`watcher.rs`にはユニットテストがないため、ここではビルド成功のみ確認する)。

- [ ] **Step 3: コミットする**

```bash
git add src-tauri/src/watcher.rs
git commit -m "docs: watcher.rsに解説コメントを追加"
```

---

### Task 5: `main.ts`へのコメント追加

**Files:**
- Modify: `src/main.ts`

**Interfaces:**
- Consumes: `get_initial_content`コマンド、`file-changed`イベント(Task 3, 4のRust側と対応)
- Produces: `render`, `showError`, `init`(シグネチャ・挙動は変更なし)

- [ ] **Step 1: ファイル全体を以下の内容に置き換える**

```ts
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/**
 * Rust側の`Document`構造体(main.rs参照)にそのまま対応する型。
 * TauriはRustの`serde::Serialize`とTypeScriptのこの型を、JSON経由で
 * 橋渡ししてくれる。フィールド名(`file_name`)はRust側の命名(snake_case)
 * のままにしている点に注意(TSの慣習ならcamelCaseにしたいところだが、
 * JSONのキー名を変換する層を追加するのはこの規模のアプリではやり
 * すぎなので、そのまま受け取っている)。
 */
interface Doc {
  file_name: string;
  html: string;
}

/** watcher.rsが送ってくる`file-changed`イベントの中身。 */
interface FileChangedPayload {
  html: string;
}

// `document.getElementById`の戻り値の型は`HTMLElement | null`です。
// 末尾の`!`(非nullアサーション演算子)は、「このIDの要素は
// `index.html`に必ず存在するとTypeScriptに約束する」という意味の
// 記法です。実行時に本当に存在するかはチェックしないので、
// 存在が確実な場合にだけ使うべき機能です。
const header = document.getElementById("filename")!;
const content = document.getElementById("content")!;

/** HTMLを描画し、再描画後もスクロール位置を保つ。 */
function render(html: string) {
  const y = window.scrollY;
  // Trusted: HTML is produced by our own Rust converter from a local file.
  content.innerHTML = html;
  // `requestAnimationFrame`は「次の描画(ペイント)の直前」に渡した
  // 関数を実行するようブラウザに予約する仕組みです。`innerHTML`を
  // 差し替えた直後はまだレイアウト(高さなど)が確定していないことが
  // あるため、1フレーム待ってからスクロール位置を復元することで、
  // 正しい位置に戻せるようにしています。
  requestAnimationFrame(() => window.scrollTo(0, y));
}

/** エラーメッセージを表示する。 */
function showError(message: string) {
  header.textContent = "Error";
  document.title = "mdview — error";
  content.innerHTML = "";
  const pre = document.createElement("pre");
  pre.className = "error";
  // `innerHTML`ではなく`textContent`を使っている点が重要。エラー
  // メッセージには読み込もうとしたファイルパスなどが含まれ得るが、
  // それを`innerHTML`で挿入すると万一メッセージにHTMLタグが混ざって
  // いた場合にそのまま解釈されてしまう(XSSの原因になり得る)。
  // 「プレーンテキストとして扱いたいものは`textContent`、信頼できる
  // 形で生成されたHTMLだけ`innerHTML`」という使い分けを徹底している。
  pre.textContent = message;
  content.appendChild(pre);
}

// `async function`は「実行すると即座にPromiseを返す関数」を定義する
// 構文です。関数の中で`await`を使うと、その式が解決する(値が確定
// する)までそこで一時停止し、他の処理をブロックせずに待てます。
// JavaScriptはシングルスレッドですが、`await`中は他のイベント処理
// (画面描画やクリックなど)が動き続けられるのがポイントです。
async function init() {
  try {
    // `invoke`はTauriのRust側`#[tauri::command]`関数を呼び出す関数
    // です。ここでは戻り値の型を`<Doc>`として明示することで、
    // TypeScript側は「Rustの`get_initial_content`が返すJSONは
    // この形をしている」という前提で型チェックできます(実行時に
    // 型が保証されるわけではなく、あくまで開発者が正しく宣言する
    // 必要がある点に注意)。
    const doc = await invoke<Doc>("get_initial_content");
    header.textContent = doc.file_name;
    render(doc.html);

    // `listen`はRust側の`app.emit("file-changed", ...)`(watcher.rs
    // 参照)に対応するイベントリスナーを登録する。コールバック内の
    // `e.payload`がRustから送られてきたペイロードそのもの。
    await listen<FileChangedPayload>("file-changed", (e) => render(e.payload.html));
  } catch (err) {
    // `invoke`が失敗する(Rust側が`Err`を返す)と、TypeScript側では
    // Promiseの`reject`として現れ、`try/catch`で捕まえられる。
    // `err`の型は`unknown`なので、表示前に`String(err)`で文字列化
    // している。
    showError(String(err));
  }
}

init();
```

- [ ] **Step 2: 型チェックとビルドを確認する**

```bash
npm run build
```

Expected: エラーなく完了し、`dist/`にファイルが生成される。

- [ ] **Step 3: コミットする**

```bash
git add src/main.ts
git commit -m "docs: main.tsに解説コメントを追加"
```

---

### Task 6: `style.css`へのコメント追加

**Files:**
- Modify: `src/style.css`

**Interfaces:**
- Consumes: なし
- Produces: なし(スタイルの見た目は変更しない)

- [ ] **Step 1: ファイル全体を以下の内容に置き換える**

```css
/* すべての要素でborder/paddingをwidth/heightに含めて計算する
   (box-sizingのborder-box)。paddingを足すたびに横幅がずれるのを防ぐ、
   モダンCSSでの定番リセット。 */
* {
  box-sizing: border-box;
}

/* 本文全体のベーススタイル。日本語環境のWindowsで見栄えが良いように、
   欧文フォントのあとに日本語フォント(Yu Gothic UI / Meiryo)を
   フォールバックとして並べている。 */
body {
  font-family: -apple-system, "Segoe UI", "Helvetica Neue", Arial,
    "Yu Gothic UI", Meiryo, sans-serif;
  font-size: 16px;
  line-height: 1.6;
  color: #1f2328;
  background: #ffffff;
  max-width: 860px;
  margin: 0 auto;
  padding: 0 24px 48px;
  word-wrap: break-word;
}

/* ファイル名を表示するヘッダー。position: stickyでスクロールしても
   画面上部に張り付いたままにしている。 */
header#filename {
  position: sticky;
  top: 0;
  background: #ffffff;
  border-bottom: 1px solid #d1d9e0;
  padding: 10px 0;
  font-weight: 600;
  font-size: 14px;
}

/* h1/h2だけ下線を引いてセクションの区切りを目立たせる(GitHubの
   READMEプレビューに寄せたスタイル)。 */
h1,
h2 {
  border-bottom: 1px solid #d1d9e0;
  padding-bottom: 0.3em;
}

/* 見出し全般の余白・太さ・行間。 */
h1,
h2,
h3,
h4,
h5,
h6 {
  margin-top: 24px;
  margin-bottom: 16px;
  font-weight: 600;
  line-height: 1.25;
}

/* リンクの色と、ホバー時だけ下線を出す挙動。 */
a {
  color: #0969da;
  text-decoration: none;
}

a:hover {
  text-decoration: underline;
}

/* コードブロック(<pre>)の見た目。overflow-xで横に長いコードでも
   レイアウトを壊さずスクロールできるようにしている。 */
pre {
  background: #f6f8fa;
  border-radius: 6px;
  padding: 16px;
  overflow-x: auto;
  line-height: 1.45;
}

/* インラインコード(`code`)の見た目。等幅フォントのフォールバックに
   日本語環境向けの"MS Gothic"を含めている。 */
code {
  font-family: ui-monospace, Consolas, "Courier New", "MS Gothic", monospace;
  font-size: 85%;
  background: #f6f8fa;
  padding: 0.2em 0.4em;
  border-radius: 4px;
}

/* <pre><code>のように入れ子になっている場合は、codeの背景・パディング
   を打ち消してpre側のスタイルだけが効くようにする。 */
pre code {
  background: none;
  padding: 0;
  font-size: 100%;
}

/* テーブルは横に長くなりがちなので、display: block +
   overflow-x: autoで画面幅を超えても横スクロールで収まるようにして
   いる(縦方向のcollapseはborder-collapseで維持)。 */
table {
  border-collapse: collapse;
  display: block;
  max-width: 100%;
  overflow-x: auto;
  margin-bottom: 16px;
}

th,
td {
  border: 1px solid #d1d9e0;
  padding: 6px 13px;
}

th {
  font-weight: 600;
  background: #f6f8fa;
}

/* 偶数行に薄い背景色を付け、行を目で追いやすくする(ゼブラストライプ)。 */
tr:nth-child(2n) {
  background: #f6f8fa;
}

/* 引用(blockquote)は左側に縦線を引き、本文よりやや薄い色にする。 */
blockquote {
  border-left: 4px solid #d1d9e0;
  color: #59636e;
  margin-left: 0;
  padding-left: 16px;
}

ul,
ol {
  padding-left: 2em;
}

/* タスクリスト(- [ ]/- [x])はpulldown-cmarkが<li class="task-list-item">
   を出力する。通常のリストマーカー(・や数字)を消して、代わりに
   チェックボックスだけが見えるように左マージンを調整している。 */
li.task-list-item {
  list-style: none;
  margin-left: -1.4em;
}

input[type="checkbox"] {
  margin-right: 0.5em;
  vertical-align: middle;
}

hr {
  border: 0;
  border-top: 1px solid #d1d9e0;
  margin: 24px 0;
}

/* 画像が本文の最大幅を超えてはみ出さないようにする。 */
img {
  max-width: 100%;
}

/* main.tsのshowError()が生成する要素用。エラーであることが一目で
   わかるよう赤系の配色にしている。 */
pre.error {
  background: #fff1f0;
  border: 1px solid #ffa39e;
  color: #a8071a;
  white-space: pre-wrap;
}
```

- [ ] **Step 2: ビルドを確認する**

```bash
npm run build
```

Expected: エラーなく完了する。

- [ ] **Step 3: コミットする**

```bash
git add src/style.css
git commit -m "docs: style.cssに解説コメントを追加"
```

---

### Task 7: `Cargo.toml`へのコメント追加

**Files:**
- Modify: `src-tauri/Cargo.toml`

**Interfaces:**
- Consumes: なし
- Produces: なし(依存関係・ビルド設定は変更しない)

- [ ] **Step 1: ファイル全体を以下の内容に置き換える**

```toml
[package]
name = "mdview"
version = "0.1.0"
edition = "2021"

# ビルド時にだけ必要な依存(実行時のバイナリには含まれない)。
# tauri-buildはbuild.rsの中でアイコンやWindowsのリソース情報を
# 実行ファイルに埋め込む処理を行う。
[build-dependencies]
tauri-build = { version = "2", features = [] }

# 実行時に必要な依存。`features = []`は「そのクレートのオプション
# 機能を何も有効にしない」という意味で、不要な機能(と、それに伴う
# 依存クレートやコンパイル時間)を削るために明示的に空にしている。
[dependencies]
# TauriのRust側ランタイム本体。ウィンドウ管理・IPC(invoke/emit)・
# アプリのライフサイクル管理を提供する。
tauri = { version = "2", features = [] }
# RustのstructをJSON等にシリアライズ/デシリアライズするための
# デファクトスタンダードなライブラリ。`derive`機能を有効にすることで
# `#[derive(Serialize)]`のようにマクロで自動実装できるようになる。
serde = { version = "1", features = ["derive"] }
# Markdown→HTML変換の本体(markdown.rs参照)。`default-features = false`
# でCLIツール向けの余分な機能(getopts依存など)を外し、HTML出力に
# 必要な`features = ["html"]`だけを有効にしている。
pulldown-cmark = { version = "0.13", default-features = false, features = ["html"] }
# ファイル監視ライブラリ(watcher.rs参照)。OS標準のファイル変更通知
# APIをクロスプラットフォームな1つのインターフェースにまとめている。
notify = "8"

# `cargo build --release`(= `npm run tauri build`が使う)でだけ有効な
# 最適化設定。「起動速度を最優先し、不要なものは削る」という設計
# 方針を、そのままビルド設定にも反映している。
[profile.release]
# "s"はサイズ優先の最適化レベル("3"のような速度優先の最大値に比べ、
# バイナリサイズを削ることを優先する)。ビューアとしての処理は軽い
# ため、多少の実行速度より軽量な実行ファイルを優先している。
opt-level = "s"
# LTO(Link Time Optimization)。クレートをまたいだ最適化(不要な
# コードの除去やインライン化)を行い、実行ファイルを小さく・速く
# する代わりにビルド時間が伸びる。
lto = true
# コンパイル単位を1つにまとめることで、LTOと組み合わせたときの
# 最適化の効果を最大化する(引き換えにビルドの並列度が下がり、
# ビルド時間はさらに伸びる)。
codegen-units = 1
# デバッグシンボルを実行ファイルから取り除き、サイズを削減する。
strip = true
# panicが起きたときに「スタックを巻き戻して後片付けする」処理を
# 省略し、即座にプロセスを終了させる。巻き戻し用のテーブルが
# 不要になる分バイナリが小さくなる代わりに、`catch_unwind`のような
# 「panicを捕まえて継続する」機能は使えなくなる(このアプリでは
# panicから復旧する必要がないため、このトレードオフを取っている)。
panic = "abort"
```

- [ ] **Step 2: `Cargo.lock`が変化しないことを確認する**

```bash
git diff --stat src-tauri/Cargo.lock
```

Expected: 出力なし(依存バージョンは一切変えていないため)。

- [ ] **Step 3: ビルドを確認する**

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

Expected: エラーなくビルドが完了する。

- [ ] **Step 4: コミットする**

```bash
git add src-tauri/Cargo.toml
git commit -m "docs: Cargo.tomlに解説コメントを追加"
```

---

### Task 8: `tauri.conf.json`の解説ドキュメント作成

**Files:**
- Create: `src-tauri/tauri.conf.json.explained.md`

**Interfaces:**
- Consumes: `src-tauri/tauri.conf.json`(内容は変更しない、読むだけ)
- Produces: なし

- [ ] **Step 1: `src-tauri/tauri.conf.json.explained.md`を作成する**

```markdown
# tauri.conf.json 解説

Tauri v2アプリの設定ファイル。JSONはコメントを書けないため、この
ファイルで1つずつ解説する。

## トップレベル

| キー | 役割 |
| --- | --- |
| `$schema` | エディタの補完・検証用。Tauriが公開しているJSON Schemaを指す |
| `productName` | 実行ファイル名や表示名のもとになる文字列(`mdview`) |
| `version` | アプリのバージョン。`package.json`と揃えている |
| `identifier` | アプリを一意に識別するID(逆ドメイン記法)。Windowsのショートカット統合やアップデータなどTauriの各機能が内部的に使う |

## `build`

開発時・ビルド時にTauriがどうフロントエンドを扱うかの設定。

| キー | 役割 |
| --- | --- |
| `devUrl` | `npm run tauri dev`のとき、Tauriのウィンドウがどこを読み込むか。`vite.config.ts`の`server.port`(1420)と一致させる必要がある |
| `frontendDist` | `npm run tauri build`のとき、ビルド済みの静的ファイルがどこにあるか(`vite build`の出力先である`dist/`) |
| `beforeDevCommand` / `beforeBuildCommand` | Tauriがdev/buildを実行する前に自動で走らせるnpmスクリプト |

## `app.windows`

起動するウィンドウの設定を配列で指定する(複数ウィンドウを持つ
アプリなら複数書ける)。

| キー | 役割 |
| --- | --- |
| `label` | ウィンドウの識別子。Rust側で`app.get_webview_window("main")`のように参照する |
| `title` | 初期タイトル(main.rsの`setup`でファイル名に上書きされる) |
| `width` / `height` | 初期サイズ。前回のサイズを記憶する仕組みは意図的に実装していない(要件: 固定サイズでよい、参照: README.md) |
| `resizable` | ユーザーがウィンドウサイズを変更できるか |

## `app.security.csp`

CSP(Content Security Policy)。ブラウザ/WebViewに対して「どこから
何を読み込んでいいか」を制限するルール。

```
default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:
```

- `default-src 'self'`: 特に指定のないリソース(スクリプトなど)は
  同一オリジン(アプリ自身が提供するファイル)からしか読み込めない。
  Markdown内に埋め込まれた`<script>`や`onclick=`属性は、このCSPに
  よって実行されない(main.tsが`innerHTML`で信頼できないHTMLを挿入
  していても、実行そのものはブロックされる多層防御)
- `style-src 'self' 'unsafe-inline'`: スタイルシートは同一オリジンに
  加えてインラインスタイル(`<style>`タグや`style=`属性)も許可。
  READMEなどのMarkdownファイルにインラインスタイルが含まれることは
  珍しくないため
- `img-src 'self' data:`: 画像は同一オリジンに加えて`data:`URI
  (Base64埋め込み画像)も許可

## `bundle`

| キー | 役割 |
| --- | --- |
| `active` | `false`にすると、`tauri build`はインストーラ(MSI/NSISなど)を作らず、実行ファイル(exe)のビルドだけを行う。「ポータブル単一exe、インストーラ不要」という要件に合わせた設定 |
| `icon` | 実行ファイルに埋め込むアイコン。`active: false`でも、Windows実行ファイルのリソースとしてicoファイルは必要 |
```

- [ ] **Step 2: `tauri.conf.json`自体が変更されていないことを確認する**

```bash
git diff --stat src-tauri/tauri.conf.json
```

Expected: 出力なし。

- [ ] **Step 3: コミットする**

```bash
git add src-tauri/tauri.conf.json.explained.md
git commit -m "docs: tauri.conf.jsonの解説ドキュメントを追加"
```

---

### Task 9: `package.json`/`tsconfig.json`/`capabilities/default.json`の解説ドキュメント作成

**Files:**
- Create: `package.json.explained.md`
- Create: `tsconfig.json.explained.md`
- Create: `src-tauri/capabilities/default.json.explained.md`

**Interfaces:**
- Consumes: 対応する3つのJSONファイル(内容は変更しない、読むだけ)
- Produces: なし

- [ ] **Step 1: `package.json.explained.md`を作成する**

```markdown
# package.json 解説

Node.js/npmプロジェクトの設定ファイル。

## `scripts`

| スクリプト | 役割 |
| --- | --- |
| `dev` | Viteの開発サーバーを起動する(`npm run tauri dev`から間接的に呼ばれる。`tauri.conf.json`の`beforeDevCommand`参照) |
| `build` | `tsc --noEmit`でTypeScriptの型チェックだけを行い(JSファイルは出力しない)、そのあと`vite build`で実際のバンドルを生成する。型エラーがあればここで気づける |
| `tauri` | Tauri CLI(`@tauri-apps/cli`)を呼び出す入り口。`npm run tauri dev`や`npm run tauri build`はこのスクリプト経由でTauri CLIに引数を渡している |

## `dependencies`

| パッケージ | 役割 |
| --- | --- |
| `@tauri-apps/api` | フロントエンド(TypeScript)からTauriの機能(`invoke`でRustのコマンドを呼ぶ、`listen`でRustからのイベントを受け取るなど)を使うためのAPI |

## `devDependencies`

| パッケージ | 役割 |
| --- | --- |
| `@tauri-apps/cli` | `tauri dev` / `tauri build` / `tauri icon`などのCLIコマンド本体 |
| `typescript` | `tsc`(型チェッカー)本体 |
| `vite` | フロントエンドのビルドツール。開発サーバーと本番バンドルの両方を担う |

## なぜReactを使っていないか

表示内容が「ファイル名 + Markdownを変換したHTML」のほぼ静的な画面
なため、仮想DOMや状態管理を持つフレームワークは過剰と判断し、
`main.ts`で直接DOM操作をする構成にしている(起動速度と依存の
少なさを優先する設計方針、README.md参照)。
```

- [ ] **Step 2: `tsconfig.json.explained.md`を作成する**

```markdown
# tsconfig.json 解説

TypeScriptコンパイラ(`tsc`)の設定。このプロジェクトでは`tsc`は
型チェック専用(`--noEmit`)で、実際のJS出力はViteが行う。

| キー | 役割 |
| --- | --- |
| `target` | 出力(相当)するJavaScriptの構文レベル。`ES2021`はモダンなNode.js/ブラウザなら問題なく動く水準 |
| `module` / `moduleResolution` | `ESNext` + `bundler`は「Viteのようなバンドラーがモジュール解決を行う」ことを前提にした設定。Node.js独自の解決ルールに縛られない |
| `lib` | 型チェック時に「存在するもの」として扱うAPI群。`DOM`を含めることで`document.getElementById`などブラウザAPIの型が使える |
| `strict` | Nullチェックの厳格化など、TypeScriptの型チェックを最も厳しいモードにする。`main.ts`で`document.getElementById("filename")!`のように非nullアサーション(`!`)が必要になっているのも、このstrictモードのおかげで「nullかもしれない」ことが型上も見える化されているため |
| `noEmit` | `.js`ファイルを出力しない(型チェックのみ行う)。実際のビルドはVite側の責務 |
| `noUnusedLocals` / `noUnusedParameters` | 使っていないローカル変数・引数があるとエラーにする。デッドコードの検出 |
| `skipLibCheck` | 依存ライブラリ(`node_modules`)内の型定義ファイルまではチェックしない。ビルド時間の短縮のため |
```

- [ ] **Step 3: `src-tauri/capabilities/default.json.explained.md`を作成する**

```markdown
# capabilities/default.json 解説

Tauri v2の「capabilities(権限)」設定。Tauriはv2から、アプリが
使えるAPIをこのファイルで明示的に許可する仕組みになっている
(既定で全部許可、ではなく最小権限が基本方針)。

| キー | 役割 |
| --- | --- |
| `$schema` | エディタ補完・検証用のJSON Schema参照 |
| `identifier` | このcapabilities定義自体の識別子 |
| `windows` | この権限セットを適用するウィンドウのラベル一覧。`tauri.conf.json`の`app.windows[].label`(`"main"`)と対応する |
| `permissions` | 許可する権限のリスト。`core:default`はTauriコアの基本的な権限セット(`invoke`でのコマンド呼び出し、`listen`でのイベント購読など)を指す。mdviewはファイルダイアログやシステムトレイなどの追加プラグインを使っていないため、これだけで足りている |
```

- [ ] **Step 4: 対応する3つのJSONファイルが変更されていないことを確認する**

```bash
git diff --stat package.json tsconfig.json src-tauri/capabilities/default.json
```

Expected: 出力なし。

- [ ] **Step 5: コミットする**

```bash
git add package.json.explained.md tsconfig.json.explained.md src-tauri/capabilities/default.json.explained.md
git commit -m "docs: package.json/tsconfig.json/capabilitiesの解説ドキュメントを追加"
```

---

### Task 10: Rust基礎kata 01 — `Result`と`?`演算子

**Files:**
- Create: `exercises/basic/rust/Cargo.toml`
- Create: `exercises/basic/rust/src/lib.rs`
- Create: `exercises/basic/rust/src/kata01_result.rs`
- Create: `exercises/answers/basic/rust/kata01_result.rs`
- Modify: `.gitignore`

**Interfaces:**
- Produces: `pub fn parse_positive(s: &str) -> Result<u32, String>`

- [ ] **Step 1: 独立したCargoパッケージを作成する**

`exercises/basic/rust/Cargo.toml`を新規作成する。

```toml
[package]
name = "rust-katas"
version = "0.1.0"
edition = "2021"

# このパッケージはsrc-tauri(本体アプリ)から完全に独立している。
# 依存や設定に手を加えても本体アプリには一切影響しない。
[dependencies]
```

- [ ] **Step 1b: `.gitignore`に`exercises/basic/rust`のビルド成果物を追加する**

`cargo test`を実行すると`exercises/basic/rust/target/`が生成される。`.gitignore`の末尾に以下の行を追加する。

```
exercises/basic/rust/target/
```

- [ ] **Step 2: `lib.rs`を作成する**

`exercises/basic/rust/src/lib.rs`を新規作成する。

```rust
//! 基礎kata置き場。各`kataNN_*`モジュールに1問ずつ入っている。
//! `cargo test`で全問まとめて採点できる。

pub mod kata01_result;
```

(kata02〜04は後続タスクでこのファイルに追記する)

- [ ] **Step 3: kata01(未実装版)を作成する**

`exercises/basic/rust/src/kata01_result.rs`を新規作成する。

```rust
//! kata01: Result<T, E> と `?` 演算子
//!
//! `main.rs`の`read_utf8`で使われている「Resultを返す関数を`?`で
//! つなぐ」パターンを、小さな例で練習する。
//!
//! `s`を数値としてパースし、0より大きければその値を返す。
//! - 数値としてパースできない場合: `Err("not a number".to_string())`
//! - パースはできるが0以下の場合: `Err("must be positive".to_string())`
//!
//! ヒント:
//! - `s.parse::<i64>()`は`Result<i64, std::num::ParseIntError>`を返す
//! - `Result`には`.map_err(|_| ...)`でErrの中身を変換するメソッドがある
//! - `?`演算子は「Errなら即returnし、Okなら中身を取り出す」
pub fn parse_positive(s: &str) -> Result<u32, String> {
    todo!("s をi64としてパースし、0より大きければu32として返す実装をしてください")
}

#[cfg(test)]
mod tests {
    use super::parse_positive;

    #[test]
    fn parses_positive_number() {
        assert_eq!(parse_positive("42"), Ok(42));
    }

    #[test]
    fn rejects_non_number() {
        assert_eq!(parse_positive("abc"), Err("not a number".to_string()));
    }

    #[test]
    fn rejects_zero_or_negative() {
        assert_eq!(parse_positive("0"), Err("must be positive".to_string()));
        assert_eq!(parse_positive("-5"), Err("must be positive".to_string()));
    }
}
```

- [ ] **Step 4: 失敗することを確認する(RED)**

```bash
cd exercises/basic/rust
cargo test
```

Expected: `todo!()`のパニックにより3件とも`FAILED`。

- [ ] **Step 5: 実装して通す(GREEN、確認用に一時的に書く)**

`kata01_result.rs`の`todo!(...)`を一時的に以下へ置き換える。

```rust
pub fn parse_positive(s: &str) -> Result<u32, String> {
    let n: i64 = s.parse().map_err(|_| "not a number".to_string())?;
    if n <= 0 {
        return Err("must be positive".to_string());
    }
    Ok(n as u32)
}
```

```bash
cargo test
```

Expected: `test result: ok. 3 passed`

- [ ] **Step 6: 解答例として保存する**

実装した`kata01_result.rs`全体(ヒントコメント込み、`todo!`部分を上記実装に置き換えたもの)を`../../answers/basic/rust/kata01_result.rs`としてコピーする(パスは`exercises/`ルートから見て`exercises/answers/basic/rust/kata01_result.rs`)。

- [ ] **Step 7: kata本体を未実装版に戻す**

`exercises/basic/rust/src/kata01_result.rs`の`parse_positive`本体を、Step 3の`todo!(...)`版に戻す(学習者が最初に解く状態に戻す)。

```bash
cargo test
```

Expected: 再び3件とも`FAILED`(RED状態で配布する)。

- [ ] **Step 8: worktreeルートに戻ってコミットする**

```bash
cd ../../..
git add exercises/basic/rust exercises/answers/basic/rust/kata01_result.rs .gitignore
git commit -m "docs: Rust基礎kata01(Result/?演算子)を追加"
```

---

### Task 11: Rust基礎kata 02 — 所有権と借用

**Files:**
- Modify: `exercises/basic/rust/src/lib.rs`
- Create: `exercises/basic/rust/src/kata02_ownership.rs`
- Create: `exercises/answers/basic/rust/kata02_ownership.rs`

**Interfaces:**
- Produces: `pub fn join_with_comma(items: &[String]) -> String`, `pub fn take_ownership_demo(items: Vec<String>) -> Option<String>`

- [ ] **Step 1: `lib.rs`にモジュールを追加する**

`exercises/basic/rust/src/lib.rs`に以下の行を追記する。

```rust
pub mod kata02_ownership;
```

- [ ] **Step 2: kata02(未実装版)を作成する**

`exercises/basic/rust/src/kata02_ownership.rs`を新規作成する。

```rust
//! kata02: 所有権(ownership)と借用(borrow)
//!
//! Rustの値は「誰か1人の所有者」を持ち、関数に値そのもの(例:
//! `Vec<String>`)を渡すと所有権が移動(move)し、渡した側はもう
//! その値を使えなくなる。一方、参照(`&`)を渡すと「借用」になり、
//! 所有権は移動せず、貸した側は引き続きその値を使える。
//!
//! `join_with_comma`は、文字列のスライス`&[String]`を「借用」で受け
//! 取り、それらをカンマ区切りで1つの`String`に結合して返してください。
//! 引数の所有権は奪わないので、呼び出し側は関数呼び出し後も`items`
//! を使い続けられます。
//!
//! ヒント: `items.join(", ")`は`&[String]`に対して使える標準メソッド
//! ですが、今回は`for`ループで手を動かして書いてみてください。
pub fn join_with_comma(items: &[String]) -> String {
    todo!("items を \", \" 区切りで1つのStringに結合してください")
}

/// `take_ownership_demo`は逆に所有権を「奪う」例です。引数`items`は
/// `Vec<String>`(参照ではない)なので、この関数を呼んだ時点で
/// 呼び出し元は`items`を手放します。ここでは、最後の要素だけを
/// 取り出して返し、残りは(戻り値に含めず)破棄してください。
///
/// ヒント: `Vec`には`.pop()`という「末尾の要素を`Option<T>`として
/// 取り出す」メソッドがあります。
pub fn take_ownership_demo(mut items: Vec<String>) -> Option<String> {
    todo!("items の最後の要素を取り出して返してください")
}

#[cfg(test)]
mod tests {
    use super::{join_with_comma, take_ownership_demo};

    #[test]
    fn joins_strings_with_comma() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(join_with_comma(&items), "a, b, c");
        // items はここでもまだ使える(所有権を奪われていないため)。
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn joins_empty_slice() {
        let items: Vec<String> = vec![];
        assert_eq!(join_with_comma(&items), "");
    }

    #[test]
    fn takes_last_item() {
        let items = vec!["x".to_string(), "y".to_string()];
        assert_eq!(take_ownership_demo(items), Some("y".to_string()));
    }

    #[test]
    fn empty_vec_returns_none() {
        assert_eq!(take_ownership_demo(vec![]), None);
    }
}
```

- [ ] **Step 3: 失敗することを確認する(RED)**

```bash
cd exercises/basic/rust
cargo test kata02
```

Expected: `todo!()`のパニックにより4件とも`FAILED`。

- [ ] **Step 4: 実装して通す(GREEN、確認用に一時的に書く)**

```rust
pub fn join_with_comma(items: &[String]) -> String {
    let mut out = String::new();
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(item);
    }
    out
}

pub fn take_ownership_demo(mut items: Vec<String>) -> Option<String> {
    items.pop()
}
```

```bash
cargo test kata02
```

Expected: `test result: ok. 4 passed`

- [ ] **Step 5: 解答例として保存し、kata本体は未実装版に戻す**

Task 10のStep 6-7と同じ手順で、実装版を`exercises/answers/basic/rust/kata02_ownership.rs`として保存してから、`exercises/basic/rust/src/kata02_ownership.rs`の関数本体を`todo!(...)`に戻す。

```bash
cargo test kata02
```

Expected: 再び4件とも`FAILED`。

- [ ] **Step 6: コミットする**

```bash
cd ../../..
git add exercises/basic/rust exercises/answers/basic/rust/kata02_ownership.rs
git commit -m "docs: Rust基礎kata02(所有権/借用)を追加"
```

---

### Task 12: Rust基礎kata 03 — パターンマッチとenum

**Files:**
- Modify: `exercises/basic/rust/src/lib.rs`
- Create: `exercises/basic/rust/src/kata03_pattern_matching.rs`
- Create: `exercises/answers/basic/rust/kata03_pattern_matching.rs`

**Interfaces:**
- Produces: `pub enum Shape`, `pub fn area(shape: &Shape) -> f64`, `pub fn find_square_side(shapes: &[Shape]) -> Option<f64>`

- [ ] **Step 1: `lib.rs`にモジュールを追加する**

`exercises/basic/rust/src/lib.rs`に以下の行を追記する。

```rust
pub mod kata03_pattern_matching;
```

- [ ] **Step 2: kata03(未実装版)を作成する**

`exercises/basic/rust/src/kata03_pattern_matching.rs`を新規作成する。

```rust
//! kata03: パターンマッチ(match)とenum
//!
//! Rustのenumは「複数の種類のうちどれか1つ」を表現でき、それぞれの
//! 種類(バリアント)にデータを持たせられます(タグ付きunionに近い)。
//! `match`で分岐すると、コンパイラが「すべてのバリアントを網羅して
//! いるか」をチェックしてくれます。

pub enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
}

/// `shape`の面積を計算してください。
///
/// ヒント:
/// - `match shape { Shape::Circle { radius } => ..., Shape::Rectangle { width, height } => ... }`
/// - 円の面積は `std::f64::consts::PI * radius * radius`
pub fn area(shape: &Shape) -> f64 {
    todo!("shape の種類に応じて面積を計算してください")
}

/// 図形のリストから、最初に見つかった「正方形(width == heightの
/// Rectangle)」の一辺の長さを返してください。見つからなければ`None`。
///
/// ヒント: `Vec::iter()`と`for`ループ、または`Iterator::find_map`が
/// 使えます。`if let Shape::Rectangle { width, height } = shape`で
/// enumの中身を取り出せます。
pub fn find_square_side(shapes: &[Shape]) -> Option<f64> {
    todo!("最初の正方形の一辺の長さを探してください")
}

#[cfg(test)]
mod tests {
    use super::{area, find_square_side, Shape};

    #[test]
    fn circle_area() {
        let shape = Shape::Circle { radius: 2.0 };
        assert!((area(&shape) - 12.566370614359172).abs() < 1e-9);
    }

    #[test]
    fn rectangle_area() {
        let shape = Shape::Rectangle { width: 3.0, height: 4.0 };
        assert_eq!(area(&shape), 12.0);
    }

    #[test]
    fn finds_square() {
        let shapes = vec![
            Shape::Rectangle { width: 2.0, height: 3.0 },
            Shape::Rectangle { width: 5.0, height: 5.0 },
        ];
        assert_eq!(find_square_side(&shapes), Some(5.0));
    }

    #[test]
    fn no_square_returns_none() {
        let shapes = vec![Shape::Rectangle { width: 2.0, height: 3.0 }];
        assert_eq!(find_square_side(&shapes), None);
    }
}
```

- [ ] **Step 3: 失敗することを確認する(RED)**

```bash
cd exercises/basic/rust
cargo test kata03
```

Expected: `todo!()`のパニックにより4件とも`FAILED`。

- [ ] **Step 4: 実装して通す(GREEN、確認用に一時的に書く)**

```rust
pub fn area(shape: &Shape) -> f64 {
    match shape {
        Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
        Shape::Rectangle { width, height } => width * height,
    }
}

pub fn find_square_side(shapes: &[Shape]) -> Option<f64> {
    for shape in shapes {
        if let Shape::Rectangle { width, height } = shape {
            if width == height {
                return Some(*width);
            }
        }
    }
    None
}
```

```bash
cargo test kata03
```

Expected: `test result: ok. 4 passed`

- [ ] **Step 5: 解答例として保存し、kata本体は未実装版に戻す**

実装版を`exercises/answers/basic/rust/kata03_pattern_matching.rs`として保存してから、`exercises/basic/rust/src/kata03_pattern_matching.rs`の関数本体を`todo!(...)`に戻す。

```bash
cargo test kata03
```

Expected: 再び4件とも`FAILED`。

- [ ] **Step 6: コミットする**

```bash
cd ../../..
git add exercises/basic/rust exercises/answers/basic/rust/kata03_pattern_matching.rs
git commit -m "docs: Rust基礎kata03(パターンマッチ/enum)を追加"
```

---

### Task 13: Rust基礎kata 04 — イテレータ

**Files:**
- Modify: `exercises/basic/rust/src/lib.rs`
- Create: `exercises/basic/rust/src/kata04_iterator.rs`
- Create: `exercises/answers/basic/rust/kata04_iterator.rs`

**Interfaces:**
- Produces: `pub fn long_names_uppercase(names: &[String]) -> Vec<String>`, `pub fn contains(numbers: &[i32], target: i32) -> bool`

- [ ] **Step 1: `lib.rs`にモジュールを追加する**

`exercises/basic/rust/src/lib.rs`に以下の行を追記する。

```rust
pub mod kata04_iterator;
```

- [ ] **Step 2: kata04(未実装版)を作成する**

`exercises/basic/rust/src/kata04_iterator.rs`を新規作成する。

```rust
//! kata04: イテレータ(map/filter/collect)
//!
//! `watcher.rs`の`event.paths.iter().any(|p| ...)`のように、Rustでは
//! ループを書く代わりにイテレータのメソッドを連鎖させてデータを
//! 変換・絞り込みするのが定番のスタイルです。

/// `names`の中から、3文字以上の名前だけを大文字に変換したリストを
/// 返してください。
///
/// ヒント: `.iter()`→`.filter(|s| ...)`→`.map(|s| s.to_uppercase())`
/// →`.collect()`という連鎖で書けます。
pub fn long_names_uppercase(names: &[String]) -> Vec<String> {
    todo!("3文字以上の名前だけ大文字にして集めてください")
}

/// `numbers`の中に、指定した`target`と一致する要素が「1つでも」
/// あるかどうかを返してください(watcher.rsの`.any(...)`と同じ考え方)。
pub fn contains(numbers: &[i32], target: i32) -> bool {
    todo!("numbers の中に target と一致する要素があるか判定してください")
}

#[cfg(test)]
mod tests {
    use super::{contains, long_names_uppercase};

    #[test]
    fn filters_and_uppercases() {
        let names = vec!["Al".to_string(), "Bob".to_string(), "Eve".to_string()];
        assert_eq!(long_names_uppercase(&names), vec!["BOB".to_string(), "EVE".to_string()]);
    }

    #[test]
    fn empty_input() {
        let names: Vec<String> = vec![];
        assert_eq!(long_names_uppercase(&names), Vec::<String>::new());
    }

    #[test]
    fn contains_found() {
        assert!(contains(&[1, 2, 3], 2));
    }

    #[test]
    fn contains_not_found() {
        assert!(!contains(&[1, 2, 3], 9));
    }
}
```

- [ ] **Step 3: 失敗することを確認する(RED)**

```bash
cd exercises/basic/rust
cargo test kata04
```

Expected: `todo!()`のパニックにより4件とも`FAILED`。

- [ ] **Step 4: 実装して通す(GREEN、確認用に一時的に書く)**

```rust
pub fn long_names_uppercase(names: &[String]) -> Vec<String> {
    names
        .iter()
        .filter(|s| s.len() >= 3)
        .map(|s| s.to_uppercase())
        .collect()
}

pub fn contains(numbers: &[i32], target: i32) -> bool {
    numbers.iter().any(|&n| n == target)
}
```

```bash
cargo test kata04
```

Expected: `test result: ok. 4 passed`

- [ ] **Step 5: 解答例として保存し、kata本体は未実装版に戻す**

実装版を`exercises/answers/basic/rust/kata04_iterator.rs`として保存してから、`exercises/basic/rust/src/kata04_iterator.rs`の関数本体を`todo!(...)`に戻す。

- [ ] **Step 6: 4問すべてがRED状態であることを最終確認する**

```bash
cargo test
```

Expected: `kata01`〜`kata04`の全15件が`FAILED`(`todo!()`によるパニック)。これが学習者に配布する初期状態。

- [ ] **Step 7: コミットする**

```bash
cd ../../..
git add exercises/basic/rust exercises/answers/basic/rust/kata04_iterator.rs
git commit -m "docs: Rust基礎kata04(イテレータ)を追加"
```

---

### Task 14: TypeScript基礎kata 01 — async/await

**Files:**
- Create: `exercises/basic/typescript/kata01_async.ts`
- Create: `exercises/answers/basic/typescript/kata01_async.ts`

**Interfaces:**
- Produces: なし(スクリプトとして直接実行するのみ)

- [ ] **Step 1: kata01(未実装版)を作成する**

`exercises/basic/typescript/kata01_async.ts`を新規作成する。

```ts
// kata01: async/await
//
// main.ts の `init()` では、Tauriの`invoke`が返すPromiseを`await`で
// 待ってから次の処理に進んでいる。ここでは同じパターンを、Tauriに
// 依存しない小さな例で練習する。
//
// 「1秒待ってから受け取った数値を2倍にする」という非同期関数を
// 実装してください。
//
// ヒント:
// - `new Promise<number>((resolve) => setTimeout(() => resolve(value * 2), 1000))`
//   のように、Promiseの中で`setTimeout`を使うと「時間がかかる処理」を
//   模擬できます
// - この関数自体は`async function`にし、内部で作った`Promise`を
//   `await`して返してください
async function doubleAfterDelay(value: number): Promise<number> {
  // TODO: 1秒待ってから value * 2 を返す実装をしてください
  throw new Error("not implemented");
}

// --- 動作確認用(このファイルを直接 node で実行すると下が動きます) ---
// 期待する出力: "result: 10"
async function main() {
  const result = await doubleAfterDelay(5);
  console.log(`result: ${result}`);
}

main();
```

- [ ] **Step 2: 失敗することを確認する(RED)**

```bash
node --experimental-strip-types exercises/basic/typescript/kata01_async.ts
```

Expected: `Error: not implemented`がスローされ、`Uncaught`として表示される。

- [ ] **Step 3: 実装して通す(GREEN、確認用に一時的に書く)**

```ts
async function doubleAfterDelay(value: number): Promise<number> {
  return await new Promise<number>((resolve) => {
    setTimeout(() => resolve(value * 2), 1000);
  });
}
```

```bash
node --experimental-strip-types exercises/basic/typescript/kata01_async.ts
```

Expected: `result: 10`と出力される(約1秒後)。

- [ ] **Step 4: 解答例として保存し、kata本体は未実装版に戻す**

実装版を`exercises/answers/basic/typescript/kata01_async.ts`として保存してから、`exercises/basic/typescript/kata01_async.ts`の`doubleAfterDelay`本体をStep 1の`throw new Error("not implemented")`版に戻す。

- [ ] **Step 5: コミットする**

```bash
git add exercises/basic/typescript/kata01_async.ts exercises/answers/basic/typescript/kata01_async.ts
git commit -m "docs: TypeScript基礎kata01(async/await)を追加"
```

---

### Task 15: TypeScript基礎kata 02 — 型注釈とinterface

**Files:**
- Create: `exercises/basic/typescript/kata02_types.ts`
- Create: `exercises/answers/basic/typescript/kata02_types.ts`

**Interfaces:**
- Produces: なし(スクリプトとして直接実行するのみ)

- [ ] **Step 1: kata02(未実装版)を作成する**

`exercises/basic/typescript/kata02_types.ts`を新規作成する。

```ts
// kata02: 型注釈とinterface
//
// main.ts では、Rust側の`Document`構造体に対応する`interface Doc`を
// 定義し、`invoke<Doc>(...)`のようにジェネリクスで「この関数が返す
// 値の形」を伝えている。ここでは同じ考え方を練習する。
//
// 「本(book)」を表すinterfaceを定義し、蔵書のリストから条件に合う
// ものを探す関数を実装してください。

interface Book {
  title: string;
  author: string;
  year: number;
}

// TODO: Bookの配列`books`から、`year`が`after`より新しい本のタイトル
// (`title`)だけを配列で返す関数を実装してください。
function titlesPublishedAfter(books: Book[], after: number): string[] {
  throw new Error("not implemented");
}

// --- 動作確認用 ---
// 期待する出力: '["Rust入門","TypeScript実践"]'
function main() {
  const books: Book[] = [
    { title: "Rust入門", author: "A", year: 2022 },
    { title: "古い本", author: "B", year: 1990 },
    { title: "TypeScript実践", author: "C", year: 2023 },
  ];
  const result = titlesPublishedAfter(books, 2000);
  console.log(JSON.stringify(result));
}

main();
```

- [ ] **Step 2: 失敗することを確認する(RED)**

```bash
node --experimental-strip-types exercises/basic/typescript/kata02_types.ts
```

Expected: `Error: not implemented`がスローされる。

- [ ] **Step 3: 実装して通す(GREEN、確認用に一時的に書く)**

```ts
function titlesPublishedAfter(books: Book[], after: number): string[] {
  return books.filter((b) => b.year > after).map((b) => b.title);
}
```

```bash
node --experimental-strip-types exercises/basic/typescript/kata02_types.ts
```

Expected: `["Rust入門","TypeScript実践"]`と出力される。

- [ ] **Step 4: 解答例として保存し、kata本体は未実装版に戻す**

実装版を`exercises/answers/basic/typescript/kata02_types.ts`として保存してから、`exercises/basic/typescript/kata02_types.ts`の`titlesPublishedAfter`本体をStep 1の`throw new Error("not implemented")`版に戻す。

- [ ] **Step 5: コミットする**

```bash
git add exercises/basic/typescript/kata02_types.ts exercises/answers/basic/typescript/kata02_types.ts
git commit -m "docs: TypeScript基礎kata02(型注釈/interface)を追加"
```

---

### Task 16: 発展課題のREADME(5本)を作成

**Files:**
- Create: `exercises/advanced/01-line-numbers/README.md`
- Create: `exercises/advanced/02-dark-mode/README.md`
- Create: `exercises/advanced/03-print/README.md`
- Create: `exercises/advanced/04-html-export/README.md`
- Create: `exercises/advanced/05-context-menu/README.md`

**Interfaces:**
- Consumes: なし(課題文のみ)
- Produces: なし

- [ ] **Step 1: `exercises/advanced/01-line-numbers/README.md`を作成する**

```markdown
# 発展課題01: コードブロックに行番号を表示する

## 課題

コードブロック(```` ``` ````で囲んだ部分)に、行番号を表示できるように
してください。

## ヒント

- `src-tauri/src/markdown.rs`の`to_html`は、今は`pulldown_cmark::html::push_html`
  にすべて任せている。行番号を差し込むには、`Parser`が生成する
  `Event`列を自分で処理する必要がある
- `pulldown_cmark::Event`には`Start(Tag::CodeBlock(_))`・`Text(_)`・
  `End(TagEnd::CodeBlock)`がある。コードブロックの中の`Text`イベントを
  受け取ったら、その文字列を`\n`で分割し、1行ずつ`<span class="line-number">`
  のような要素で包んでからHTMLに追記するとよい
- CSSでは`.line-number`のような専用クラスを用意し、幅を揃えて
  右寄せにすると数字が読みやすい
- 既存のテスト(`code_block_plain`など)を壊さないよう、まずは新しい
  テスト(行番号が表示されることを確認するテスト)を追加してから実装
  してください

## 完了の目安

- `cargo test --manifest-path src-tauri/Cargo.toml`が通る
- `npm run tauri dev -- -- -- <コードブロックを含むmdファイル>`で
  実際に行番号が表示される

## 参考実装

`exercises/answers/advanced/01-line-numbers.patch`(パッチファイル)。
自分で実装したあと、または詰まったときに参照してください。
```

- [ ] **Step 2: `exercises/advanced/02-dark-mode/README.md`を作成する**

```markdown
# 発展課題02: ダークモード対応

## 課題

OSのダークモード設定に連動して、見た目が暗い配色に切り替わるように
してください(切り替えボタンなどのUIは不要。OSの設定に自動で追従
すればOK)。

## ヒント

- `src/style.css`に`@media (prefers-color-scheme: dark)`ブロックを
  追加し、その中で背景色・文字色・罫線の色などを上書きする
- 既存の色(`#ffffff`, `#1f2328`, `#f6f8fa`, `#d1d9e0`など)がどこで
  使われているか`style.css`全体を確認してから、暗い配色のペアを
  考えるとよい
- Windowsでダークモードを試すには、設定 > 個人用設定 > 色 から
  「濃い」モードに切り替える

## 完了の目安

- OSをダークモードに切り替えた状態で`mdview`を起動すると、暗い
  配色で表示される
- ライトモードのときの見た目は変化しない

## 参考実装

`exercises/answers/advanced/02-dark-mode.patch`(パッチファイル)。
自分で実装したあと、または詰まったときに参照してください。
```

- [ ] **Step 3: `exercises/advanced/03-print/README.md`を作成する**

```markdown
# 発展課題03: 印刷対応

## 課題

`Ctrl+P`で現在表示中のMarkdownを印刷できるようにし、印刷時には
画面に固定されているヘッダーの背景色など、印刷に不要な装飾を
消してください。

## ヒント

- ブラウザ/WebViewの`window.print()`を呼べば印刷ダイアログを開ける。
  `src/main.ts`に`document.addEventListener("keydown", ...)`で
  `Ctrl+P`(`e.ctrlKey && e.key === "p"`)を検知する処理を追加する
- ブラウザの既定の印刷ショートカット動作と衝突しないよう、検知したら
  `e.preventDefault()`を呼んでから`window.print()`を呼ぶとよい
- `src/style.css`に`@media print { ... }`ブロックを追加し、
  `header#filename`の`position: sticky`や背景色など画面表示専用の
  スタイルを印刷時だけ打ち消す

## 完了の目安

- `mdview`表示中に`Ctrl+P`を押すと印刷プレビューが開く
- 印刷プレビューでヘッダーの背景色が付いていない(白紙に馴染む)

## 参考実装

`exercises/answers/advanced/03-print.patch`(パッチファイル)。
自分で実装したあと、または詰まったときに参照してください。
```

- [ ] **Step 4: `exercises/advanced/04-html-export/README.md`を作成する**

```markdown
# 発展課題04: HTML出力機能の追加

## 課題

表示中のMarkdownを、CSSを埋め込んだ単体のHTMLファイルとして書き出す
機能を追加してください(元のMarkdownファイルと同じフォルダに
`<元のファイル名>.html`として保存する想定)。

## ヒント

- `src-tauri/src/main.rs`を参考に、新しい`#[tauri::command]`関数
  (例: `export_html`)を追加する。引数でHTML文字列を受け取り、
  `AppState`が持っている`path`(監視対象のファイルパス)を使って
  保存先を決め、`std::fs::write`で書き込み、成功したら保存先パスを
  `Result<String, String>`で返す設計にするとよい
- 新しいコマンドは`tauri::generate_handler![get_initial_content, export_html]`
  のように`invoke_handler`に追加登録する必要がある(登録を忘れると
  フロントエンドから`invoke`で呼んでも見つからないエラーになる)
- CSSを埋め込むには、Rust側の`include_str!("../../src/style.css")`で
  ビルド時にファイルの中身を文字列として埋め込む方法が使える
- フロントエンド側は`Ctrl+E`などのショートカットで
  `invoke("export_html", { html })`を呼び出し、成功したら保存先パスを
  一時的に画面に表示するとよい

## 完了の目安

- `Ctrl+E`(など決めたショートカット)を押すと、元のMarkdownファイルと
  同じフォルダに単体で開けるHTMLファイルが生成される
- 生成されたHTMLファイルをブラウザで開くと、`mdview`で見た表示と
  同じ見た目になる

## 参考実装

`exercises/answers/advanced/04-html-export.patch`(パッチファイル)。
自分で実装したあと、または詰まったときに参照してください。
```

- [ ] **Step 5: `exercises/advanced/05-context-menu/README.md`を作成する**

```markdown
# 発展課題05: エクスプローラー右クリックからmdviewで開く

## 課題

Windowsのエクスプローラーで`.md`ファイルを右クリックしたときに、
「mdviewで開く」という項目が出るようにしてください。この課題は
アプリのコード変更ではなく、Windowsのレジストリ設定が対象です。

## 背景知識

Windowsでは、ファイルの右クリックメニューに項目を追加する方法の
1つに、レジストリの`HKEY_CLASSES_ROOT\SystemFileAssociations\.md\shell`
以下にキーを追加する方法があります。

```
HKEY_CLASSES_ROOT
└─ SystemFileAssociations
   └─ .md
      └─ shell
         └─ mdview                  ← メニューに表示される項目名の元
            ├─ (既定)  = "mdviewで開く"
            └─ command
               └─ (既定) = "C:\path\to\mdview.exe" "%1"
```

## ヒント

- レジストリの変更は`.reg`ファイル(テキストで書けるレジストリの
  インポート用ファイル)にまとめておくと、他の人のPCにも簡単に
  適用・削除できる
- `%1`はエクスプローラーが右クリックされたファイルのフルパスに
  置き換えて渡してくれるプレースホルダー。`main.rs`の
  `std::env::args().nth(1)`がこの`%1`を受け取ることになる
- レジストリ変更は元に戻せるようにしておくこと(削除用の`.reg`、
  またはPowerShellの`Remove-Item`を使ったアンインストールスクリプトを
  セットで用意する)
- 実際に試す前に、レジストリのバックアップを取ってから作業すること
  (この課題はシステム設定を変更するため、実施は自己責任で)

## 完了の目安

- 作成した`.reg`をダブルクリックしてインポートすると、`.md`ファイルの
  右クリックメニューに項目が現れる
- クリックすると`mdview`がそのファイルを開いた状態で起動する
- アンインストール用のスクリプト/`.reg`で元の状態に戻せる

## 参考実装

`exercises/answers/advanced/05-context-menu/`(`.reg`ファイルと
インストール/アンインストールスクリプト一式)。
```

- [ ] **Step 6: コミットする**

```bash
git add exercises/advanced
git commit -m "docs: 発展課題README(5本)を追加"
```

---

### Task 17: 発展課題の解答01 — コードブロックへの行番号表示

**Files:**
- Temporarily modify then revert: `src-tauri/src/markdown.rs`, `src/style.css`
- Create: `exercises/answers/advanced/01-line-numbers.patch`

**Interfaces:**
- Consumes: `markdown::to_html`(Task 2)の既存実装
- Produces: パッチファイル(実際のコードには残さない)

- [ ] **Step 1: `src-tauri/src/markdown.rs`の`use`宣言と`to_html`を以下に置き換える**

冒頭の`use`宣言を置き換える。

```rust
use pulldown_cmark::{html, Event, Options, Parser, Tag, TagEnd};
```

`to_html`関数全体(モジュールコメント・ドキュメンテーションコメントはそのまま)を以下に置き換える。

```rust
pub fn to_html(source: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(source, opts);

    let mut out = String::with_capacity(source.len() * 3 / 2);
    let mut in_code_block = false;

    // pulldown-cmarkの既定のHTML生成器(html::push_html)は、コードブロック
    // の中身を素通しでHTMLに変換するだけで行番号は挿入してくれない。
    // そこでイベント列を自前でたどり、コードブロックの`Text`イベントだけ
    // 特別扱いして1行ごとに<span>で包む。
    //
    // 簡略化のため「1つのコードブロックにつきTextイベントは1つ」という
    // pulldown-cmarkの一般的な挙動を前提にしている。複数のTextイベントに
    // 分割されるケースでは行番号が1から数え直されてしまう可能性がある
    // (さらに正確にするなら、コードブロックの開始から終了までの行番号
    // カウンタを外に持ち出して累積させるとよい)。
    let mut buffered = Vec::new();
    for event in parser {
        match &event {
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
                buffered.push(event);
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                buffered.push(event);
            }
            Event::Text(text) if in_code_block => {
                let numbered = number_lines(text);
                buffered.push(Event::Html(numbered.into()));
            }
            _ => buffered.push(event),
        }
    }

    html::push_html(&mut out, buffered.into_iter());
    out
}

/// コードブロックの中身(複数行の文字列)を、1行ごとに
/// `<span class="line-number"></span>`を先頭に付けたテキストへ変換する。
/// 出力はそのままHTMLとして埋め込む文字列なので、`<`, `>`, `&`は
/// エスケープする。
fn number_lines(text: &str) -> String {
    let mut out = String::new();
    let body = text.strip_suffix('\n').unwrap_or(text);
    for (i, line) in body.split('\n').enumerate() {
        out.push_str(&format!(
            "<span class=\"line-number\">{}</span>{}\n",
            i + 1,
            escape_html(line)
        ));
    }
    out
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
```

- [ ] **Step 2: 行番号のテストを追加する**

`#[cfg(test)] mod tests`の中(既存テストの末尾)に以下を追加する。

```rust
    #[test]
    fn code_block_has_line_numbers() {
        let html = to_html("```rust\nfn main() {}\nprintln!(\"hi\");\n```");
        assert!(html.contains("<span class=\"line-number\">1</span>"), "output: {html}");
        assert!(html.contains("<span class=\"line-number\">2</span>"), "output: {html}");
        assert!(!html.contains("<span class=\"line-number\">3</span>"), "output: {html}");
    }

    #[test]
    fn escapes_html_in_code() {
        let html = to_html("```\n<script>\n```");
        assert!(html.contains("&lt;script&gt;"), "output: {html}");
    }
```

- [ ] **Step 3: `src/style.css`の末尾に追加する**

```css

/* 発展課題01: コードブロックの行番号表示。 */
.line-number {
  display: inline-block;
  width: 2em;
  margin-right: 1em;
  text-align: right;
  color: #8c959f;
  user-select: none;
}
```

- [ ] **Step 4: テストが通ることを確認する(GREEN)**

```bash
cargo test --manifest-path src-tauri/Cargo.toml markdown::
```

Expected: 既存6件 + 新規2件の合計8件が`ok`。

- [ ] **Step 5: フロントエンドのビルドも通ることを確認する**

```bash
npm run build
```

Expected: エラーなく完了する。

- [ ] **Step 6: 差分をパッチファイルとして保存する**

```bash
mkdir -p exercises/answers/advanced
git diff -- src-tauri/src/markdown.rs src/style.css > exercises/answers/advanced/01-line-numbers.patch
```

- [ ] **Step 7: 変更を取り消し、実コードを元に戻す**

```bash
git checkout -- src-tauri/src/markdown.rs src/style.css
```

- [ ] **Step 8: 元に戻ったことを確認する**

```bash
cargo test --manifest-path src-tauri/Cargo.toml markdown::
git status --porcelain src-tauri/src/markdown.rs src/style.css
```

Expected: テストは元の6件のみ`ok`。`git status`は出力なし(変更なし)。

- [ ] **Step 9: パッチファイルをコミットする**

```bash
git add exercises/answers/advanced/01-line-numbers.patch
git commit -m "docs: 発展課題01(行番号表示)の参考実装パッチを追加"
```

---

### Task 18: 発展課題の解答02 — ダークモード対応

**Files:**
- Temporarily modify then revert: `src/style.css`
- Create: `exercises/answers/advanced/02-dark-mode.patch`

**Interfaces:**
- Consumes: `src/style.css`(Task 6の既存内容)
- Produces: パッチファイル(実際のコードには残さない)

- [ ] **Step 1: `src/style.css`の末尾に追加する**

```css

/* 発展課題02: OSのダークモード設定に連動する配色。 */
@media (prefers-color-scheme: dark) {
  body {
    color: #e6edf3;
    background: #0d1117;
  }

  header#filename {
    background: #0d1117;
    border-bottom-color: #30363d;
  }

  h1,
  h2 {
    border-bottom-color: #30363d;
  }

  a {
    color: #4493f8;
  }

  pre,
  code {
    background: #161b22;
  }

  th,
  td {
    border-color: #30363d;
  }

  th {
    background: #161b22;
  }

  tr:nth-child(2n) {
    background: #161b22;
  }

  blockquote {
    border-left-color: #30363d;
    color: #8b949e;
  }

  hr {
    border-top-color: #30363d;
  }

  pre.error {
    background: #2d0f0f;
    border-color: #f85149;
    color: #ffa198;
  }
}
```

- [ ] **Step 2: ビルドが通ることを確認する**

```bash
npm run build
```

Expected: エラーなく完了する。

- [ ] **Step 3: 差分をパッチファイルとして保存する**

```bash
git diff -- src/style.css > exercises/answers/advanced/02-dark-mode.patch
```

- [ ] **Step 4: 変更を取り消す**

```bash
git checkout -- src/style.css
```

- [ ] **Step 5: 元に戻ったことを確認する**

```bash
git status --porcelain src/style.css
```

Expected: 出力なし。

- [ ] **Step 6: パッチファイルをコミットする**

```bash
git add exercises/answers/advanced/02-dark-mode.patch
git commit -m "docs: 発展課題02(ダークモード)の参考実装パッチを追加"
```

---

### Task 19: 発展課題の解答03 — 印刷対応

**Files:**
- Temporarily modify then revert: `src/main.ts`, `src/style.css`
- Create: `exercises/answers/advanced/03-print.patch`

**Interfaces:**
- Consumes: `src/main.ts`(Task 5)、`src/style.css`(Task 6)の既存内容
- Produces: パッチファイル(実際のコードには残さない)

- [ ] **Step 1: `src/main.ts`の末尾(`init();`の後)に追加する**

```ts

// 発展課題03: Ctrl+Pで印刷する。ブラウザ既定のショートカットの
// 代わりに`window.print()`を明示的に呼ぶことで、印刷対象を
// このウィンドウの内容に限定する。
document.addEventListener("keydown", (e) => {
  if (e.ctrlKey && e.key === "p") {
    e.preventDefault();
    window.print();
  }
});
```

- [ ] **Step 2: `src/style.css`の末尾に追加する**

```css

/* 発展課題03: 印刷時は画面専用の装飾(固定ヘッダーの背景など)を消す。 */
@media print {
  header#filename {
    position: static;
    background: none;
    border-bottom: none;
  }

  pre,
  code {
    background: none;
  }

  body {
    max-width: none;
  }
}
```

- [ ] **Step 3: ビルドが通ることを確認する**

```bash
npm run build
```

Expected: エラーなく完了する。

- [ ] **Step 4: 差分をパッチファイルとして保存する**

```bash
git diff -- src/main.ts src/style.css > exercises/answers/advanced/03-print.patch
```

- [ ] **Step 5: 変更を取り消す**

```bash
git checkout -- src/main.ts src/style.css
```

- [ ] **Step 6: 元に戻ったことを確認する**

```bash
git status --porcelain src/main.ts src/style.css
```

Expected: 出力なし。

- [ ] **Step 7: パッチファイルをコミットする**

```bash
git add exercises/answers/advanced/03-print.patch
git commit -m "docs: 発展課題03(印刷対応)の参考実装パッチを追加"
```

---

### Task 20: 発展課題の解答04 — HTML出力機能

**Files:**
- Temporarily modify then revert: `src-tauri/src/main.rs`, `src/main.ts`
- Create: `exercises/answers/advanced/04-html-export.patch`

**Interfaces:**
- Consumes: `AppState`(Task 3)、`invoke`/`render`(Task 5)の既存実装
- Produces: パッチファイル(実際のコードには残さない)

- [ ] **Step 1: `src-tauri/src/main.rs`に新しいコマンドを追加する**

`get_initial_content`関数の直後に以下を追加する。

```rust

// 発展課題04: 表示中のHTMLをCSS埋め込みの単体ファイルとして書き出す。
// フロントエンドから渡された`html`と、AppStateが持つ元ファイルの
// パスを組み合わせて `<元のファイル名>.html` を同じフォルダに保存する。
#[tauri::command]
fn export_html(state: tauri::State<AppState>, html: String) -> Result<String, String> {
    let path = state
        .path
        .clone()
        .ok_or_else(|| "No source file to export from.".to_string())?;
    let out_path = path.with_extension("html");
    // include_str!はビルド時にファイルの中身をそのまま文字列として
    // 埋め込むマクロ。実行時にファイルを読みに行かないので、
    // 生成されるHTMLは実行環境に依存せず自己完結する。
    let css = include_str!("../../src/style.css");
    let document = format!(
        "<!doctype html>\n<html lang=\"ja\">\n<head>\n<meta charset=\"UTF-8\">\n<style>\n{css}\n</style>\n</head>\n<body>\n<main>\n{html}\n</main>\n</body>\n</html>\n"
    );
    std::fs::write(&out_path, &document)
        .map_err(|e| format!("Cannot write {}: {e}", out_path.display()))?;
    Ok(out_path.display().to_string())
}
```

`invoke_handler`の行を以下に置き換える。

```rust
        .invoke_handler(tauri::generate_handler![get_initial_content, export_html])
```

- [ ] **Step 2: `src/main.ts`を変更する**

`render`関数を以下に置き換える(最新のHTMLを覚えておくための変数を追加)。

```ts
let latestHtml = "";
let currentFileName = "";

function render(html: string) {
  latestHtml = html;
  const y = window.scrollY;
  content.innerHTML = html;
  requestAnimationFrame(() => window.scrollTo(0, y));
}
```

`init`関数内、`header.textContent = doc.file_name;`の行を以下に置き換える。

```ts
    currentFileName = doc.file_name;
    header.textContent = doc.file_name;
```

ファイルの末尾(`init();`の後)に以下を追加する。

```ts

// 発展課題04: Ctrl+EでHTML出力する。
document.addEventListener("keydown", async (e) => {
  if (e.ctrlKey && e.key === "e") {
    e.preventDefault();
    try {
      const path = await invoke<string>("export_html", { html: latestHtml });
      header.textContent = `保存しました: ${path}`;
      setTimeout(() => {
        header.textContent = currentFileName;
      }, 3000);
    } catch (err) {
      showError(String(err));
    }
  }
});
```

- [ ] **Step 3: テストとビルドが通ることを確認する**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
npm run build
```

Expected: 両方エラーなく完了する。

- [ ] **Step 4: 差分をパッチファイルとして保存する**

```bash
git diff -- src-tauri/src/main.rs src/main.ts > exercises/answers/advanced/04-html-export.patch
```

- [ ] **Step 5: 変更を取り消す**

```bash
git checkout -- src-tauri/src/main.rs src/main.ts
```

- [ ] **Step 6: 元に戻ったことを確認する**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
git status --porcelain src-tauri/src/main.rs src/main.ts
```

Expected: テストは元の10件のみ`ok`。`git status`は出力なし。

- [ ] **Step 7: パッチファイルをコミットする**

```bash
git add exercises/answers/advanced/04-html-export.patch
git commit -m "docs: 発展課題04(HTML出力)の参考実装パッチを追加"
```

---

### Task 21: 発展課題の解答05 — エクスプローラー右クリック連携

**Files:**
- Create: `exercises/answers/advanced/05-context-menu/install.reg`
- Create: `exercises/answers/advanced/05-context-menu/uninstall.reg`
- Create: `exercises/answers/advanced/05-context-menu/README.md`

**Interfaces:**
- Consumes: なし(アプリのコードは変更しない)
- Produces: なし

- [ ] **Step 1: `exercises/answers/advanced/05-context-menu/install.reg`を作成する**

```reg
Windows Registry Editor Version 5.00

[HKEY_CLASSES_ROOT\SystemFileAssociations\.md\shell\mdview]
@="mdviewで開く"
"Icon"="C:\\path\\to\\mdview.exe"

[HKEY_CLASSES_ROOT\SystemFileAssociations\.md\shell\mdview\command]
@="\"C:\\path\\to\\mdview.exe\" \"%1\""
```

- [ ] **Step 2: `exercises/answers/advanced/05-context-menu/uninstall.reg`を作成する**

```reg
Windows Registry Editor Version 5.00

[-HKEY_CLASSES_ROOT\SystemFileAssociations\.md\shell\mdview]
```

- [ ] **Step 3: `exercises/answers/advanced/05-context-menu/README.md`を作成する**

```markdown
# 発展課題05 参考実装: エクスプローラー右クリック連携

## 使い方

1. `install.reg`を開き、`C:\path\to\mdview.exe`の部分を実際に
   `mdview.exe`を置いた場所の絶対パスに書き換える
2. `install.reg`をダブルクリックしてレジストリにインポートする
   (ユーザーアカウント制御の確認が出たら許可する)
3. 任意の`.md`ファイルを右クリックし、「mdviewで開く」が表示されて
   クリックで起動できることを確認する

## 元に戻す

`uninstall.reg`をダブルクリックしてインポートすると、追加した
レジストリキーが削除される。

## 補足

- `[-キー名]`という記法(先頭に`-`)は、レジストリエディタに対して
  「このキーを削除する」という意味になる
- 実際にアプリとして配布する場合は、インストーラ(NSIS/WiXなど)の
  中でこれと同等のレジストリ操作を自動化するのが一般的。今回は
  「ポータブル単一exe、インストーラなし」という設計方針のため、
  手動でレジストリを操作する形の参考実装にしている
```

- [ ] **Step 4: コミットする**

```bash
git add exercises/answers/advanced/05-context-menu
git commit -m "docs: 発展課題05(右クリック連携)の参考実装を追加"
```

---

### Task 22: 最終検証と仕上げ

**Files:**
- なし(検証のみ)

**Interfaces:**
- Consumes: すべての前タスクの成果物

- [ ] **Step 1: 本体アプリのテストが通ることを確認する(挙動が変わっていないことの最終確認)**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: `test result: ok. 10 passed`(Task 2, 3でコメントを追加したのみで、テスト件数・内容は`master`と同一)。

- [ ] **Step 2: フロントエンドのビルドが通ることを確認する**

```bash
npm run build
```

Expected: エラーなく完了する。

- [ ] **Step 3: 基礎kata(Rust)が意図通りREDで配布されることを確認する**

```bash
cd exercises/basic/rust
cargo test
cd ../../..
```

Expected: 4問(kata01〜04)、計15件すべて`FAILED`(`todo!()`によるパニック)。学習者が最初に取り組む状態として正しい。

- [ ] **Step 4: 基礎kata(TypeScript)が意図通り例外を投げることを確認する**

```bash
node --experimental-strip-types exercises/basic/typescript/kata01_async.ts
node --experimental-strip-types exercises/basic/typescript/kata02_types.ts
```

Expected: 両方とも`Error: not implemented`で終了する。

- [ ] **Step 5: 発展課題の答えパッチが実コードにきれいに当たることを確認する**

```bash
git apply --check exercises/answers/advanced/01-line-numbers.patch
git apply --check exercises/answers/advanced/02-dark-mode.patch
git apply --check exercises/answers/advanced/03-print.patch
git apply --check exercises/answers/advanced/04-html-export.patch
```

Expected: 4件ともエラーなし(`--check`は実際には適用せず、適用可能かどうかだけ確認する)。

- [ ] **Step 6: worktreeの状態がクリーンであることを確認する**

```bash
git status
```

Expected: `nothing to commit, working tree clean`

- [ ] **Step 7: 全体のファイル一覧を目視確認する**

```bash
git ls-files exercises LEARNING.md
```

Expected: `LEARNING.md`、`exercises/README.md`、`exercises/basic/rust/*`、`exercises/basic/typescript/*`、`exercises/advanced/0[1-5]-*/README.md`、`exercises/answers/basic/rust/*`、`exercises/answers/basic/typescript/*`、`exercises/answers/advanced/*`が過不足なく一覧に含まれている。

## Self-Review(このプランの作成者によるチェック)

- **spec網羅性**: 設計spec(`docs/superpowers/specs/2026-07-05-tutorial-branch-design.md`)の各項目(全体構成/コメント方針/設定ファイル解説/テスト解説/exercises構成/完了の定義)はTask 1〜22でそれぞれ対応済み
- **プレースホルダ確認**: 全タスクに完全なコード・完全な文章を記載済み(「適切なコメントを追加」のような曖昧な指示はない)
- **型/シグネチャの一貫性**: `Document`/`AppState`/`read_utf8`/`get_initial_content`のシグネチャはTask 3で定義した後、Task 4・17・20で参照するもの(`crate::read_utf8`, `crate::markdown::to_html`, `state.path`)と一致している。kataの関数シグネチャ(`parse_positive`, `join_with_comma`など)は問題文とテストで一貫している
