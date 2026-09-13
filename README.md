# mdview

Windows向けの超軽量Markdownビューア。閲覧に特化し、起動速度を最優先しています。

```
mdview README.md
```

## 特徴

- **閲覧専用** — 編集・検索・ワークスペース機能なし
- **GitHub Flavored Markdown** — テーブル、チェックボックス、打ち消し線、コードブロック
- **自動再読み込み** — 表示中のファイルが更新されると即座に反映（スクロール位置は保持）
- **リンクを辿れる** — 別のMarkdownは新しいウィンドウで、外部URLは既定ブラウザで開く
- **ローカル画像** — Markdownからの相対パスで表示（遅延読み込み）
- **ダークモード** — OSのテーマに追従
- **日本語対応** — UTF-8、日本語フォントスタック
- **ポータブル** — 単一exe、インストーラ不要（WebView2はOS同梱を利用）

## 技術構成

| レイヤ | 技術 |
| ------ | ---- |
| シェル | Tauri v2（WebView2） |
| バックエンド | Rust（pulldown-cmark, notify） |
| フロントエンド | バニラTypeScript + Vite（フレームワークなし） |

Markdown→HTML変換はRust側で行い、フロントエンドは受け取ったHTMLを表示するだけの構成です。

## ディレクトリ構成

```
markdown-viewer/
├── package.json          # npmスクリプトとJS依存
├── tsconfig.json         # TypeScript設定（型チェックのみ）
├── vite.config.ts        # Vite設定
├── index.html            # 1画面のシェル
├── app-icon.png          # アイコン元画像
├── CONTEXT.md            # 用語集
├── docs/adr/             # 設計判断の記録
├── .github/workflows/    # CI（cargo test + tsc）
├── src/                  # フロントエンド
│   ├── main.ts           # 初期取得・更新イベント受信・スクロール保持・リンク処理
│   └── style.css         # GitHub風テーマ（ライト/ダーク） + 日本語フォント
└── src-tauri/            # Rustバックエンド
    ├── Cargo.toml
    ├── build.rs
    ├── tauri.conf.json   # ウィンドウ設定・CSP・バンドル設定
    ├── capabilities/
    │   └── default.json  # 権限（core:defaultのみ）
    ├── icons/            # 生成済みアイコン（icon.ico含む）
    └── src/
        ├── main.rs       # CLI引数処理・ファイル読み込み・画像配信・Tauriアプリ本体
        ├── links.rs      # リンク/画像の参照先解決（相対パス・UNC判定）
        ├── markdown.rs   # Markdown→HTML変換（pulldown-cmark, GFM拡張）
        └── watcher.rs    # ファイル監視（親ディレクトリ監視 + デバウンス + 削除検知）
```

## ビルド手順

### 前提条件

1. **Rust** — [rustup](https://rustup.rs/) でインストール（`x86_64-pc-windows-msvc` ツールチェーン）
2. **Visual Studio Build Tools** — 「C++によるデスクトップ開発」ワークロード
3. **Node.js 18以上** + npm
4. **WebView2ランタイム** — Windows 11 / 最新のWindows 10には標準搭載

### 開発ビルド（ホットリロード付き）

```powershell
npm install

# 引数渡しは三重の -- が必要（npm → tauri CLI → cargo run）
npm run tauri dev -- -- -- "C:\path\to\README.md"
```

### テスト

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

### リリースビルド（exe生成）

```powershell
npm install
npm run tauri build
```

成果物: `src-tauri\target\release\mdview.exe`

単一のポータブルexeです。PATHの通ったフォルダにコピーすれば、どこからでも `mdview <ファイル>` で起動できます。

```powershell
copy src-tauri\target\release\mdview.exe C:\bin\
mdview README.md
```

### アイコンを変更する場合

`app-icon.png`（1024x1024）を差し替えて再生成します:

```powershell
npm run tauri icon app-icon.png
```

## 動作仕様

- `mdview <ファイル>` — 指定ファイルを表示。実行のたびに新しいウィンドウが開く（マルチインスタンス）
- `mdview --help` / `mdview --version`
- 引数なし → ウィンドウ内にUsageを表示
- 拡張子は不問（.md以外のテキストファイルも表示可能）
- ファイル更新は親ディレクトリ監視で検知（VS Code等のアトミック保存にも対応）、300msデバウンス
- 表示中のファイルが削除されると、1秒の猶予後にヘッダで通知（本文はそのまま残る）
- 配色はOSのテーマに追従（`prefers-color-scheme`）
- Markdown内の生HTMLはそのまま表示されるが、スクリプト実行はCSPで遮断

### 読み込めないファイル

いずれもウィンドウ内にエラーを表示します。区別して表示するのは、リンクを辿ると
画像やPDFがmdviewで開かれるためです。

| 状態 | 表示 |
| ---- | ---- |
| ファイルが存在しない | File not found |
| バイナリ（先頭8KBにNULバイト） | バイナリファイルのため表示不可 |
| UTF-16（BOMあり） | UTF-16である旨 |
| その他UTF-8以外（Shift_JIS等） | UTF-8として不正である旨 |

UTF-8のみサポートします（BOM付きUTF-8は可）。

### リンクと画像

| リンク先 | 動作 |
| -------- | ---- |
| ローカルファイル | **新しいmdviewウィンドウ**で開く（拡張子は問わない） |
| `http` / `https` / `mailto` | OSの既定ハンドラ（ブラウザ等）で開く |
| `#見出し` | ページ内移動（見出しにIDは振っていないため通常は何も起きない） |
| ネットワークパス（`\\server\...`） | 何もしない |

- ローカルファイルは**常にmdview自身**に渡されます。OSの既定アプリには渡しません
- 画像はMarkdownからの相対パスで解決され、表示領域に入った時点で読み込まれます
- 画像はキャッシュされるため、Markdownを変えずに画像だけ差し替えても再描画されません
- リンクのクリック**以外**の遷移（`<meta http-equiv="refresh">`、フォーム送信）は
  すべて拒否されます

## 設計判断

なぜそうなっているのかは [`docs/adr/`](docs/adr/) に記録しています。用語は
[`CONTEXT.md`](CONTEXT.md) を参照してください。

- [ドキュメント参照は新しいウィンドウで開き、OSのシェルには渡さない](docs/adr/0001-links-open-new-window-never-the-shell.md)
- [画像は独自プロトコルで遅延配信し、鮮度よりも軽さを取る](docs/adr/0002-images-served-lazily-over-a-private-protocol.md)
- [リンクのクリックはJSで扱い、それ以外の遷移はRustで一律に拒否する](docs/adr/0003-clicks-in-js-navigation-denied-in-rust.md)
- [mdviewは状態を持たない](docs/adr/0004-mdview-holds-no-state.md)

### 意図的に持たない機能

履歴と「戻る」、OSの既定アプリ連携、画像の更新追跡、ウィンドウサイズ・位置の記憶、
ドラッグ＆ドロップ、設定ファイル、Shift_JIS対応、見出しIDの自動採番、数式、
シンタックスハイライト。それぞれの理由は上のADRに書いてあります。

## オプション機能: エクスプローラー右クリック連携

Windowsエクスプローラーで`.md`ファイルを右クリックしたときに「mdviewで開く」
を表示できるようにする、オプションのスクリプトを用意しています。管理者権限は
不要です。詳細は [`windows/context-menu/README.md`](windows/context-menu/README.md)
を参照してください。
