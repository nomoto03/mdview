# mdview

Windows向けの超軽量Markdownビューア。閲覧に特化し、起動速度を最優先しています。

```
mdview README.md
```

## 特徴

- **閲覧専用** — 編集・検索・ワークスペース機能なし
- **GitHub Flavored Markdown** — テーブル、チェックボックス、打ち消し線、コードブロック
- **自動再読み込み** — 表示中のファイルが更新されると即座に反映（スクロール位置は保持）
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
├── src/                  # フロントエンド
│   ├── main.ts           # 初期コンテンツ取得・更新イベント受信・スクロール保持
│   └── style.css         # GitHub風ライトテーマ + 日本語フォント
└── src-tauri/            # Rustバックエンド
    ├── Cargo.toml
    ├── build.rs
    ├── tauri.conf.json   # ウィンドウ設定・CSP・バンドル設定
    ├── capabilities/
    │   └── default.json  # 権限（core:defaultのみ）
    ├── icons/            # 生成済みアイコン（icon.ico含む）
    └── src/
        ├── main.rs       # CLI引数処理・ファイル読み込み・Tauriアプリ本体
        ├── markdown.rs   # Markdown→HTML変換（pulldown-cmark, GFM拡張）
        └── watcher.rs    # ファイル監視（親ディレクトリ監視 + デバウンス）
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
- 引数なし → ウィンドウ内にUsageを表示
- ファイルが存在しない → ウィンドウ内にエラーを表示
- UTF-8以外のファイル → ウィンドウ内にエラーを表示（UTF-8のみサポート、BOM付きUTF-8は可）
- 拡張子は不問（.md以外のテキストファイルも表示可能）
- ファイル更新は親ディレクトリ監視で検知（VS Code等のアトミック保存にも対応）、300msデバウンス
- Markdown内の生HTMLはそのまま表示されるが、スクリプト実行はCSPで遮断

## オプション機能: エクスプローラー右クリック連携

Windowsエクスプローラーで`.md`ファイルを右クリックしたときに「mdviewで開く」
を表示できるようにする、オプションのスクリプトを用意しています。管理者権限は
不要です。詳細は [`windows/context-menu/README.md`](windows/context-menu/README.md)
を参照してください。
