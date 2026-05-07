# Регистрация открытия .oz в OmegaZip для текущего пользователя (HKCU), без администратора.
# Двойной клик передаёт путь первому аргументу — см. RunEvent::Ready в src-tauri/src/lib.rs (Windows/Linux).
#
# Установка (GUI из сборки Tauri):
#   powershell -ExecutionPolicy Bypass -File .\scripts\install-oz-file-association-windows.ps1 -OmegaZipApp "C:\Path\To\OmegaZip.exe"
# Удаление:
#   powershell -ExecutionPolicy Bypass -File .\scripts\install-oz-file-association-windows.ps1 -Uninstall

param(
    [Parameter(Mandatory = $false)]
    [string] $OmegaZipApp,
    [switch] $Uninstall
)

$progId = "OmegaZip.oz"
$classesRoot = "Registry::HKEY_CURRENT_USER\Software\Classes"

if ($Uninstall) {
    Remove-Item -Path "$classesRoot\.oz" -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item -Path "$classesRoot\$progId" -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "Удалено: HKCU\Software\Classes\.oz и $progId"
    exit 0
}

if ([string]::IsNullOrWhiteSpace($OmegaZipApp)) {
    Write-Error "Укажите -OmegaZipApp `"C:\Path\To\OmegaZip.exe`" или -Uninstall."
    exit 1
}

$exe = (Resolve-Path -LiteralPath $OmegaZipApp).Path
if (-not (Test-Path -LiteralPath $exe)) {
    Write-Error "Файл не найден: $exe"
    exit 1
}

New-Item -Path "$classesRoot\.oz" -Force | Out-Null
Set-ItemProperty -Path "$classesRoot\.oz" -Name "(default)" -Value $progId
New-Item -Path "$classesRoot\.oz\OpenWithProgids" -Force | Out-Null
New-ItemProperty -Path "$classesRoot\.oz\OpenWithProgids" -Name $progId -Value "" -PropertyType String -Force | Out-Null

New-Item -Path "$classesRoot\$progId" -Force | Out-Null
Set-ItemProperty -Path "$classesRoot\$progId" -Name "(default)" -Value "OmegaZip Archive"
Set-ItemProperty -Path "$classesRoot\$progId" -Name "FriendlyTypeName" -Value "OmegaZip Archive"

$iconKey = Join-Path "$classesRoot\$progId" "DefaultIcon"
New-Item -Path $iconKey -Force | Out-Null
Set-ItemProperty -Path $iconKey -Name "(default)" -Value ("`"{0}`",0" -f $exe)

$openCmd = Join-Path "$classesRoot\$progId" "shell\open\command"
New-Item -Path $openCmd -Force | Out-Null
$quoted = "`"$exe`" `"%1`""
Set-ItemProperty -Path $openCmd -Name "(default)" -Value $quoted

Write-Host "Готово: .oz → $exe"
Write-Host ('Удаление: powershell -ExecutionPolicy Bypass -File "{0}" -Uninstall' -f $PSCommandPath)
