# エクスプローラー右クリック連携 設計

## 背景・目的

`mdview`は「ポータブル単一exe、インストーラなし」という設計方針を維持しつつ、Windowsエクスプローラーで`.md`ファイルを右クリックしたときに「mdviewで開く」を選べるようにしたい。当初の設計ではYAGNIとして見送っていた機能で、`tutorial`ブランチの`exercises/advanced/05-context-menu`に学習用の参考実装(`.reg`一式)があるが、これはあくまで課題の答えであり、本番の`mdview`リポジトリには含まれていない。今回、本番機能として正式に組み込む。

## 非目標 / 前提

- アプリ本体(`main.rs`/`markdown.rs`/`watcher.rs`/フロントエンド)のロジックは一切変更しない。`mdview.exe`は既に`std::env::args`でファイルパスを1つ受け取れる作りになっており、コンテキストメニューからの起動もこの既存の引数処理をそのまま利用する
- インストーラは作らない。あくまで「ポータブルexeを配置した後に任意で実行するサポートスクリプト」という位置づけ
- 自動テスト(Pester等)は追加しない。レジストリを実際に書き換える性質上、手動検証手順をREADMEに明記する形とする

## 登録先レジストリ

`HKEY_CURRENT_USER\Software\Classes\SystemFileAssociations\.md\shell\mdview` 以下に登録する。

- `HKCU`配下のため管理者権限は不要
- 実行したユーザーのExplorerにのみ反映される(他ユーザーには影響しない)
- `tutorial`ブランチの参考実装(`HKEY_CLASSES_ROOT`直下、管理者権限が必要)とは意図的に異なる方式にする。「インストーラなし・ポータブル」という本体の設計方針と整合させるため

```
HKEY_CURRENT_USER
└─ Software
   └─ Classes
      └─ SystemFileAssociations
         └─ .md
            └─ shell
               └─ mdview
                  ├─ (既定) = "mdviewで開く"
                  └─ command
                     └─ (既定) = "<ExePathの絶対パス>" "%1"
```

対象拡張子は`.md`のみ。`.markdown`等は対象外(必要になれば後から追加)。

## ディレクトリ構成

```
markdown-viewer/
└── windows/
    └── context-menu/
        ├── install.ps1     # レジストリ登録
        ├── uninstall.ps1   # レジストリ削除
        └── README.md       # 使い方・手動検証手順
```

`windows/`はアプリのソースコード(`src/`, `src-tauri/`)とは独立したサポートツール置き場として新設する。

## install.ps1 の仕様

- 必須パラメータ `-ExePath <string>`: `mdview.exe`の絶対パス。自動探索は行わない
  - 理由: `mdview.exe`はビルド場所(`src-tauri/target/release/`)にも、ユーザーが任意に配置した場所(例: `%USERPROFILE%\.local\bin`)にも置かれうる。誤った自動推測で見当違いのexeを登録する事故を避けるため、明示指定を必須にする
- 事前検証(境界での軽いチェック):
  - `-ExePath`で指定されたファイルが存在するか(`Test-Path`)
  - ファイル名が`mdview.exe`であるか(大文字小文字を区別しない完全一致)
  - どちらか満たさない場合はエラーメッセージを表示して`exit 1`
- レジストリ操作: `New-Item`/`Set-ItemProperty`で上記キーを作成
  - 既定値 = `"mdviewで開く"`
  - `command`の既定値 = `"<ExePath>" "%1"`
- 既存キーがあっても上書きするだけで失敗しない(再実行しても安全 = 冪等)
- 成功時は登録した内容(パス)を表示して終了

## uninstall.ps1 の仕様

- 引数なし
- `HKCU:\Software\Classes\SystemFileAssociations\.md\shell\mdview`キーが存在すれば`Remove-Item -Recurse`で削除
- キーが存在しない場合はエラーにせず「既に未登録です」と表示して正常終了(`exit 0`)

## エラーハンドリング方針

- レジストリ操作(`New-Item`/`Set-ItemProperty`/`Remove-Item`)は失敗しうる境界操作として`try/catch`で囲み、失敗時は日本語で分かりやすいメッセージを表示して非ゼロ終了する
- それ以外の内部ロジックは簡潔にする(過剰な防御的コードは書かない)

## テスト・検証方針

- 自動テストは追加しない(レジストリを実際に書き換えるスクリプトのため、YAGNIとしてPester等は見送る)
- `windows/context-menu/README.md`に手動検証手順を明記する:
  1. `install.ps1 -ExePath <mdview.exeの絶対パス>` を実行
  2. 任意の`.md`ファイルを右クリックし「mdviewで開く」が表示されることを確認
  3. クリックしてmdviewが起動することを確認
  4. `uninstall.ps1`を実行し、右クリックメニューから項目が消えたことを確認

## ドキュメント更新

- ルートの`README.md`に「オプション機能」として1段落追加し、`windows/context-menu/README.md`への導線を張る
