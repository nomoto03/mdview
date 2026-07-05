# mdviewを教材にしたRust/TypeScript/Tauri学習教材化 設計

## 背景・目的

`mdview`(このリポジトリのWindows向け軽量Markdownビューア)を教材として、Rust・TypeScript・Tauriのスタックが初めてのエンジニアに、環境構築からアーキテクチャ、テストの考え方までを実際に動くコードを通じて学んでもらう。

## 対象読者

- プログラミング自体の経験はある(他言語での開発経験あり)
- Rust / TypeScript / Tauri のスタックは初めて
- 「変数とは何か」レベルの説明は不要。Rust/TypeScript/Tauriならではの考え方(所有権、`Result`/`Option`、非同期処理、Tauriのコマンド/イベントモデルなど)にフォーカスする

## 非目標 / 前提

- アプリの挙動・ロジックは一切変更しない。コメント追加・ドキュメント追加・`exercises/`フォルダの新設のみを行う
- このブランチは恒久的な学習用ブランチとして扱い、`master`へのマージバックは前提としない
- 新規の実行時依存ライブラリは追加しない(教材用の依存も最小限。テストフレームワーク等は追加せず`cargo test`と素の`node`実行で完結させる)

## 全体構成

- `git worktree add ../markdown-viewer-tutorial -b tutorial` で新規worktree + `tutorial`ブランチを作成し、`master`の作業ディレクトリには影響を与えない
- ルートに `LEARNING.md` を新設し、学習の進め方(カリキュラム索引)を示す
  - 想定の読む順序: 環境構築 → Rust基礎の勘所 → アーキテクチャ概観 → `main.rs` → `markdown.rs` → `watcher.rs` → `main.ts` → `style.css` → 設定ファイル群 → テストの考え方(TDD) → `exercises/`

## コメントの書き方の方針

- **初出箇所で丁寧に、以降は簡潔に**: 同じ概念(例: `Result<T, E>`)が複数ファイルに登場する場合、最初に登場した箇所(`main.rs`の`read_utf8`)で踏み込んで解説し、以降は一言程度の参照コメントに留める。冗長な繰り返しを避ける
- **Rust**: `///`(ドキュメンテーションコメント)で関数・構造体の役割を説明し、関数内は`//`で「なぜそう書くか」(所有権、`?`演算子、パターンマッチ、スレッド間のデータのやり取りなど)を解説する
- **TypeScript**: TSDoc(`/** */`)で関数の役割、`//`で`async/await`・型注釈・DOM操作などその都度の考え方を解説する
- **CSS**: 各ルールブロックの直前に「これは何のためのスタイルか」を一言添える
- コメントを追加する対象は次の実コードファイル: `src-tauri/src/main.rs`, `src-tauri/src/markdown.rs`, `src-tauri/src/watcher.rs`, `src/main.ts`, `src/style.css`

## 設定ファイルの解説方式

- JSON(`tauri.conf.json`, `package.json`, `tsconfig.json`, `src-tauri/capabilities/default.json`)はコメントを書けない仕様のため、それぞれに対になる解説Markdownを同じ場所に配置する(例: `tauri.conf.json` → `tauri.conf.explained.md`)。主要キーを1つずつ「これは何のためか・変えるとどうなるか」で解説する
- TOML(`src-tauri/Cargo.toml`)は`#`コメントが書けるため、ファイル内に直接解説を追加する(`[dependencies]`の各クレートが何のためか、`[profile.release]`の各最適化フラグの意味など)

## テストの解説方針

- `markdown.rs`と`main.rs`の既存テストに「なぜこのケースをテストするのか」を解説するコメントを追加する
- `LEARNING.md`に、このリポジトリで実際に使ったTDD(RED→GREenのループ)の考え方を簡潔にまとめ、テストコードへの導線とする

## `exercises/`フォルダの構成

```
exercises/
├── README.md                    # 進め方・難易度表・一覧
├── basic/                       # 穴埋め形式(構文の基礎を体で覚える)
│   ├── rust/                    # 本体に影響しない独立Cargoパッケージ
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── kata01_result.rs      # todo!()を埋めてcargo testを通す
│   │       ├── kata02_ownership.rs
│   │       ├── kata03_pattern_matching.rs
│   │       └── kata04_iterator.rs
│   └── typescript/
│       ├── kata01_async.ts      # TODOコメントを埋め、nodeで直接実行して出力確認
│       └── kata02_types.ts
├── advanced/                    # 実アプリを拡張する発展課題
│   ├── 01-line-numbers/README.md
│   ├── 02-dark-mode/README.md
│   ├── 03-print/README.md
│   ├── 04-html-export/README.md
│   └── 05-context-menu/README.md
└── answers/
    ├── basic/                   # kataの完成版
    └── advanced/                # 各発展課題の参考実装
```

- `exercises/basic/rust`は`src-tauri`とは独立した単体のCargoパッケージとする(workspaceにはしない)。`src-tauri/Cargo.toml`や依存関係には一切影響しない
- **basic**: `cargo test`(Rust)・`node`(TypeScript、Node 22の型ストリッピング機能により追加ツールなしで直接実行)で自己採点でき、実装で実践したTDDのRED→GREENループをそのまま体験できる
- **advanced**: README.mdに元々あった「将来オプション」(行番号表示・ダークモード・印刷・HTML出力・右クリック連携)を課題化。実アプリのどのファイルを触るかのヒントを各課題README.mdに記載
- `answers/`に解答例(basic: kataの完成版、advanced: 各課題の参考実装)を用意する

## 完了の定義 / 検証方法

- `../markdown-viewer-tutorial`(tutorialブランチ)で `cargo test --manifest-path src-tauri/Cargo.toml` と `npm run build` が引き続き成功する(アプリの挙動が変わっていないことの確認)
- `exercises/basic/rust` で `cargo test` を実行すると、未実装のkataが失敗し(RED)、`answers/basic`の内容を反映すると全て成功する(GREEN)ことを確認する
- `exercises/basic/typescript`の各kataが`node`コマンドで実行できることを確認する
- 主要ファイル(`main.rs`, `markdown.rs`, `watcher.rs`, `main.ts`, `style.css`, 各設定ファイルと対になるexplained.md)にコメント・解説が入っていることを目視確認する
