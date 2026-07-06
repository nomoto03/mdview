# エクスプローラー右クリック連携（オプション機能）

Windowsエクスプローラーで`.md`ファイルを右クリックしたときに「mdviewで開く」
を表示できるようにする、オプションのインストール/アンインストールスクリプトです。

- `HKEY_CURRENT_USER`配下のみを操作するため、**管理者権限は不要**です
- 実行したユーザーのエクスプローラーにのみ反映されます（他ユーザーには影響しません）
- アプリ本体（`mdview.exe`）のコードやインストーラは変更しません。あくまで
  「ポータブルexeを配置した後に任意で実行するサポートスクリプト」です

## 導入

`mdview.exe`を配置した絶対パスを指定して実行します（自動探索はしません）:

```powershell
.\windows\context-menu\install.ps1 -ExePath C:\path\to\mdview.exe
```

## 削除

```powershell
.\windows\context-menu\uninstall.ps1
```

## 実行ポリシーでブロックされる場合

Windowsの既定設定では、ローカルの`.ps1`ファイルの実行がPowerShellの実行ポリシー
によりブロックされることがあります（`UnauthorizedAccess`エラー）。これは
mdview固有の問題ではなく、署名されていないスクリプト全般に適用されるWindowsの
既定動作です。

管理者権限やシステム全体の設定変更をせずに、その場限りで許可するには
`-ExecutionPolicy Bypass`を付けて実行します:

```powershell
powershell.exe -ExecutionPolicy Bypass -File .\windows\context-menu\install.ps1 -ExePath C:\path\to\mdview.exe
```

Git Bash等、PowerShell以外のシェルから`powershell.exe`経由で呼び出す場合も同様です。

## 手動検証手順

1. `install.ps1 -ExePath <mdview.exeの絶対パス>` を実行する
2. 任意の`.md`ファイルを右クリックし、「mdviewで開く」が表示されることを確認する
3. クリックしてmdviewが起動し、そのファイルが表示されることを確認する
4. `uninstall.ps1`を実行する
5. 同じ`.md`ファイルを右クリックし、「mdviewで開く」が消えていることを確認する

## 補足

- 対象拡張子は`.md`のみです（`.markdown`等は対象外）
- 登録先: `HKEY_CURRENT_USER\Software\Classes\SystemFileAssociations\.md\shell\mdview`
- `install.ps1`は再実行しても安全です（既存の登録を上書きします）
