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