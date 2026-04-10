# Устанавливает пункты контекстного меню проводника для **текущего пользователя** (HKCU),
# без прав администратора. Логика сжатия/имён — через omega-context-helper.ps1 (паритет с macOS).
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
$helper = Join-Path $PSScriptRoot "omega-context-helper.ps1"

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
    Remove-OmegaZipMenu -Name "OmegaZipCompressAuto"
    Remove-OmegaZipMenu -Name "OmegaZipCompressOz"
    Remove-OmegaZipMenu -Name "OmegaZipCompressZip"
    Remove-OmegaZipMenu -Name "OmegaZipExtractHere"
    Write-Host "Удалено: HKCU\Software\Classes\*\shell\OmegaZip*"
    exit 0
}

if ([string]::IsNullOrWhiteSpace($OmegaZipExe)) {
    Write-Error "Укажите -OmegaZipExe \"C:\Path\To\omegazip.exe\" или запустите с -Uninstall."
    exit 1
}

if (-not (Test-Path -LiteralPath $helper)) {
    Write-Error "Не найден helper: $helper (нужен omega-context-helper.ps1 рядом со скриптом)."
    exit 1
}

$exe = (Resolve-Path -LiteralPath $OmegaZipExe).Path
if (-not (Test-Path -LiteralPath $exe)) {
    Write-Error "Файл не найден: $exe"
    exit 1
}

function New-HelperCommand {
    param([string]$Mode)
    return "powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$helper`" -OmegaZipExe `"$exe`" $Mode -LiteralPath `"%1`""
}

$cmdAuto = New-HelperCommand -Mode "CompressAuto"
$cmdOz = New-HelperCommand -Mode "CompressOz"
$cmdZip = New-HelperCommand -Mode "CompressZip"
$cmdExtract = New-HelperCommand -Mode "Extract"

Set-OmegaZipMenu -Name "OmegaZipCompressAuto" -Label "Сжать OmegaZip (авто .oz/.zip)" -Arguments $cmdAuto
Set-OmegaZipMenu -Name "OmegaZipCompressOz" -Label "Сжать в .oz (OmegaZip)" -Arguments $cmdOz
Set-OmegaZipMenu -Name "OmegaZipCompressZip" -Label "Сжать в .zip (OmegaZip)" -Arguments $cmdZip
Set-OmegaZipMenu -Name "OmegaZipExtractHere" -Label "Распаковать (OmegaZip)" -Arguments $cmdExtract

Write-Host "Готово: HKCU\Software\Classes\*\shell\OmegaZip*"
Write-Host "  — Сжать OmegaZip (авто .oz/.zip) — как Services на macOS (pick_ext_auto + пресеты)."
$self = Join-Path $PSScriptRoot "install-context-menu-windows.ps1"
Write-Host "Удаление: powershell -ExecutionPolicy Bypass -File `"$self`" -Uninstall"
