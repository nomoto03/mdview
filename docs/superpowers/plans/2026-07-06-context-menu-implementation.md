# エクスプローラー右クリック連携 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Windowsエクスプローラーで`.md`ファイルを右クリックしたときに「mdviewで開く」を選べるようにする、PowerShellベースのオプション導入/削除スクリプトを追加する。

**Architecture:** `windows/context-menu/`配下に`install.ps1`（レジストリ登録）と`uninstall.ps1`（登録解除）を新設する。両スクリプトはユーザースコープの`HKEY_CURRENT_USER\Software\Classes\SystemFileAssociations\.md\shell\mdview`のみを操作し、管理者権限を必要としない。アプリ本体（`src/`, `src-tauri/`）のコードは一切変更しない。

**Tech Stack:** PowerShell 5.1（Windows標準搭載）、Windowsレジストリ（`HKCU:`プロバイダ）。新規の実行時依存なし。

## Global Constraints

- 登録先レジストリキーは `HKEY_CURRENT_USER\Software\Classes\SystemFileAssociations\.md\shell\mdview` のみ（`HKEY_CLASSES_ROOT`直下は使わない。管理者権限不要という設計方針のため）
- 対象拡張子は`.md`のみ（`.markdown`等は対象外）
- メニュー表示名は正確に `mdviewで開く`
- `command`サブキーの既定値は正確に `"<ExePathの絶対パス>" "%1"`（exeパスをダブルクォートで囲む）
- `install.ps1`は`-ExePath`パラメータを**必須**とする。exeパスの自動探索は行わない
- アプリ本体（`src/`, `src-tauri/`）のコード・ロジックは一切変更しない
- 自動テスト（Pester等）は追加しない。`windows/context-menu/README.md`に手動検証手順を明記することで代替する
- ファイル配置は `windows/context-menu/install.ps1` / `uninstall.ps1` / `README.md` の3ファイル

---

### Task 1: install.ps1 — レジストリ登録スクリプト

**Files:**
- Create: `windows/context-menu/install.ps1`

**Interfaces:**
- Consumes: なし（最初のタスク）
- Produces: CLI引数 `-ExePath <string>`（必須）。成功時にレジストリキー
  `HKCU:\Software\Classes\SystemFileAssociations\.md\shell\mdview`
  （既定値 `mdviewで開く`）と、そのサブキー`command`
  （既定値 `"<ExePath>" "%1"`）を作成する。Task 2（uninstall.ps1）は
  このキーパスを削除対象として参照する。

- [ ] **Step 1: install.ps1を作成する**

`windows/context-menu/install.ps1`:

```powershell
<#
.SYNOPSIS
    mdviewをWindowsエクスプローラーの右クリックメニュー（.mdファイル）に登録する。
.DESCRIPTION
    HKEY_CURRENT_USER配下のみを操作するため、管理者権限は不要。
    実行したユーザーのエクスプローラーにのみ反映される。
.PARAMETER ExePath
    mdview.exeの絶対パス（自動探索はしない。明示的に指定すること）。
.EXAMPLE
    .\install.ps1 -ExePath C:\Users\you\.local\bin\mdview.exe
#>
param(
    [Parameter(Mandatory = $true)]
    [string]$ExePath
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $ExePath -PathType Leaf)) {
    Write-Error "指定されたパスにファイルが見つかりません: $ExePath"
    exit 1
}

$fileName = [System.IO.Path]::GetFileName($ExePath)
if ($fileName -ne "mdview.exe") {
    Write-Error "指定されたファイルはmdview.exeではありません（ファイル名: $fileName）: $ExePath"
    exit 1
}

$exeFullPath = (Resolve-Path -LiteralPath $ExePath).ProviderPath
$keyPath = "HKCU:\Software\Classes\SystemFileAssociations\.md\shell\mdview"
$commandKeyPath = "$keyPath\command"
$commandValue = "`"$exeFullPath`" `"%1`""

try {
    New-Item -Path $keyPath -Force | Out-Null
    Set-Item -Path $keyPath -Value "mdviewで開く"

    New-Item -Path $commandKeyPath -Force | Out-Null
    Set-Item -Path $commandKeyPath -Value $commandValue
}
catch {
    Write-Error "レジストリの登録に失敗しました: $_"
    exit 1
}

Write-Host "登録しました: $keyPath"
Write-Host "  コマンド: $commandValue"
Write-Host "エクスプローラーで.mdファイルを右クリックし、「mdviewで開く」が表示されることを確認してください。"
```

- [ ] **Step 2: 存在しないパスでの検証エラーを確認する**

Run:
```powershell
.\windows\context-menu\install.ps1 -ExePath C:\nonexistent\mdview.exe
echo "exit code: $LASTEXITCODE"
```
Expected: `指定されたパスにファイルが見つかりません: C:\nonexistent\mdview.exe` が表示され、`exit code: 1`

- [ ] **Step 3: ファイル名不一致での検証エラーを確認する**

まず検証用のダミーファイルを用意してから実行する:
```powershell
New-Item -ItemType File -Path "$env:TEMP\notmdview.exe" -Force | Out-Null
.\windows\context-menu\install.ps1 -ExePath "$env:TEMP\notmdview.exe"
echo "exit code: $LASTEXITCODE"
Remove-Item "$env:TEMP\notmdview.exe"
```
Expected: `指定されたファイルはmdview.exeではありません（ファイル名: notmdview.exe）` が表示され、`exit code: 1`

- [ ] **Step 4: 実際のmdview.exeで登録し、レジストリを確認する**

このリポジトリのビルド成果物または既にPATHに配置済みの`mdview.exe`（例:
`C:\Users\<user>\.local\bin\mdview.exe`）を使う。実際にこのマシンの
`HKCU`レジストリ（ユーザースコープ）を書き換える点に注意。

```powershell
.\windows\context-menu\install.ps1 -ExePath C:\Users\nomok\.local\bin\mdview.exe

$keyPath = "HKCU:\Software\Classes\SystemFileAssociations\.md\shell\mdview"
(Get-Item -Path $keyPath).GetValue($null)
(Get-Item -Path "$keyPath\command").GetValue($null)
```
Expected:
- 1行目の出力: `mdviewで開く`
- 2行目の出力: `"C:\Users\nomok\.local\bin\mdview.exe" "%1"`

- [ ] **Step 5: 再実行しても失敗しない（冪等性）ことを確認する**

```powershell
.\windows\context-menu\install.ps1 -ExePath C:\Users\nomok\.local\bin\mdview.exe
echo "exit code: $LASTEXITCODE"
```
Expected: エラーなく完了し、`exit code: 0`。レジストリの内容もStep 4と同じ

- [ ] **Step 6: Commit**

```bash
git add windows/context-menu/install.ps1
git commit -m "feat: エクスプローラー右クリック連携のinstall.ps1を追加"
```

---

### Task 2: uninstall.ps1 — レジストリ削除スクリプト

**Files:**
- Create: `windows/context-menu/uninstall.ps1`

**Interfaces:**
- Consumes: Task 1が作成したレジストリキー
  `HKCU:\Software\Classes\SystemFileAssociations\.md\shell\mdview`（存在しなくても動作する）
- Produces: 上記キーを削除するCLIスクリプト（引数なし）

- [ ] **Step 1: uninstall.ps1を作成する**

`windows/context-menu/uninstall.ps1`:

```powershell
<#
.SYNOPSIS
    install.ps1で登録したエクスプローラー右クリックメニュー（.mdファイル）を削除する。
.EXAMPLE
    .\uninstall.ps1
#>

$ErrorActionPreference = "Stop"

$keyPath = "HKCU:\Software\Classes\SystemFileAssociations\.md\shell\mdview"

if (-not (Test-Path -LiteralPath $keyPath)) {
    Write-Host "既に未登録です: $keyPath"
    exit 0
}

try {
    Remove-Item -Path $keyPath -Recurse -Force
}
catch {
    Write-Error "レジストリの削除に失敗しました: $_"
    exit 1
}

Write-Host "削除しました: $keyPath"
```

- [ ] **Step 2: Task 1で登録した状態から削除できることを確認する**

Task 1のStep 4/5で`HKCU`に登録済みの状態から実行する:

```powershell
$keyPath = "HKCU:\Software\Classes\SystemFileAssociations\.md\shell\mdview"
Test-Path -LiteralPath $keyPath   # True であること（削除前の前提確認）

.\windows\context-menu\uninstall.ps1
echo "exit code: $LASTEXITCODE"

Test-Path -LiteralPath $keyPath   # False になっていること
```
Expected: `削除しました: HKCU:\Software\Classes\SystemFileAssociations\.md\shell\mdview` が表示され、`exit code: 0`。削除後の`Test-Path`は`False`

- [ ] **Step 3: 未登録状態での再実行を確認する**

```powershell
.\windows\context-menu\uninstall.ps1
echo "exit code: $LASTEXITCODE"
```
Expected: `既に未登録です: HKCU:\Software\Classes\SystemFileAssociations\.md\shell\mdview` が表示され、`exit code: 0`（エラー扱いにしない）

- [ ] **Step 4: Commit**

```bash
git add windows/context-menu/uninstall.ps1
git commit -m "feat: エクスプローラー右クリック連携のuninstall.ps1を追加"
```

---

### Task 3: ドキュメント整備

**Files:**
- Create: `windows/context-menu/README.md`
- Modify: `README.md`（ルート） — 100行目付近「## 動作仕様」セクションの後に新セクションを追記

**Interfaces:**
- Consumes: Task 1/2で作成した`install.ps1`/`uninstall.ps1`のCLI仕様
  （`-ExePath`必須パラメータ、exit code規約）
- Produces: なし（末端タスク）

- [ ] **Step 1: windows/context-menu/README.mdを作成する**

`windows/context-menu/README.md`:

```markdown
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
```

- [ ] **Step 2: ルートREADME.mdに導線を追加する**

`README.md`の「## 動作仕様」セクション（108行目、`- Markdown内の生HTMLは...`の後）
の直後に、以下のセクションを追記する:

```markdown

## オプション機能: エクスプローラー右クリック連携

Windowsエクスプローラーで`.md`ファイルを右クリックしたときに「mdviewで開く」
を表示できるようにする、オプションのスクリプトを用意しています。管理者権限は
不要です。詳細は [`windows/context-menu/README.md`](windows/context-menu/README.md)
を参照してください。
```

- [ ] **Step 3: 全体フローを通しで確認する**

```powershell
.\windows\context-menu\install.ps1 -ExePath C:\Users\nomok\.local\bin\mdview.exe
# エクスプローラーで任意の.mdファイルを右クリックし、「mdviewで開く」の表示とクリック起動を目視確認

.\windows\context-menu\uninstall.ps1
# 同じファイルを右クリックし、項目が消えていることを目視確認
```
Expected: 両スクリプトともエラーなく完了し、右クリックメニューの表示/非表示が目視で確認できる

- [ ] **Step 4: Commit**

```bash
git add windows/context-menu/README.md README.md
git commit -m "docs: エクスプローラー右クリック連携の使い方を追記"
```
