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
