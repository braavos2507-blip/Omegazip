# Устанавливает пункты контекстного меню проводника для **текущего пользователя** (HKCU),
# без прав администратора. Аналогично по смыслу `scripts/context-menu-windows.reg.example`.
#
# Использование:
#   Установка:
#     powershell -ExecutionPolicy Bypass -File .\scripts\install-context-menu-windows.ps1 -OmegaZipExe "C:\Path\To\omegazip.exe"
#   Удаление:
#     powershell -ExecutionPolicy Bypass -File .\scripts\install-context-menu-windows.ps1 -Uninstall

param(
    [Parameter(Mandatory = $false)]
    [string] $OmegaZipExe,
    [switch] $Uninstall
)

$shell = "Registry::HKEY_CURRENT_USER\Software\Classes\*\shell"

function Set-OmegaZipMenu {
    param([string]$Name, [string]$Label, [string]$Arguments)

    $key = Join-Path $shell $Name
    New-Item -Path $key -Force | Out-Null
    Set-ItemProperty -Path $key -Name "(default)" -Value $Label
    Set-ItemProperty -Path $key -Name "NoWorkingDirectory" -Value ""

    $cmdKey = Join-Path $key "command"
    New-Item -Path $cmdKey -Force | Out-Null
    Set-ItemProperty -Path $cmdKey -Name "(default)" -Value $Arguments
}

function Remove-OmegaZipMenu {
    param([string]$Name)
    $key = Join-Path $shell $Name
    if (Test-Path -LiteralPath $key) {
        Remove-Item -LiteralPath $key -Recurse -Force
    }
}

if ($Uninstall) {
    Remove-OmegaZipMenu -Name "OmegaZipCompressOz"
    Remove-OmegaZipMenu -Name "OmegaZipCompressZip"
    Remove-OmegaZipMenu -Name "OmegaZipExtractHere"
    Write-Host "Удалено: HKCU\Software\Classes\*\shell\OmegaZipCompressOz|Zip|ExtractHere"
    exit 0
}

if ([string]::IsNullOrWhiteSpace($OmegaZipExe)) {
    Write-Error "Укажите -OmegaZipExe \"C:\Path\To\omegazip.exe\" или запустите с -Uninstall."
    exit 1
}

$exe = (Resolve-Path -LiteralPath $OmegaZipExe).Path
if (-not (Test-Path -LiteralPath $exe)) {
    Write-Error "Файл не найден: $exe"
    exit 1
}

# Кавычки как в `context-menu-windows.reg.example`: cmd.exe /c ""<exe>" compress "%1" "%~dpn1.oz""
$cmdOz = 'cmd.exe /c ""' + $exe + '" compress "%1" "%~dpn1.oz"'
$cmdZip = 'cmd.exe /c ""' + $exe + '" compress "%1" "%~dpn1.zip"'
$cmdExtract = 'cmd.exe /c ""' + $exe + '" decompress "%1" "%~dpn1_распаковано"'

Set-OmegaZipMenu -Name "OmegaZipCompressOz" -Label "Сжать в .oz (OmegaZip)" -Arguments $cmdOz
Set-OmegaZipMenu -Name "OmegaZipCompressZip" -Label "Сжать в .zip (OmegaZip)" -Arguments $cmdZip
Set-OmegaZipMenu -Name "OmegaZipExtractHere" -Label "Распаковать (OmegaZip)" -Arguments $cmdExtract

Write-Host "Готово: HKCU\Software\Classes\*\shell\OmegaZipCompressOz|Zip|ExtractHere"
Write-Host "Удаление: powershell -ExecutionPolicy Bypass -File .\scripts\install-context-menu-windows.ps1 -Uninstall"
