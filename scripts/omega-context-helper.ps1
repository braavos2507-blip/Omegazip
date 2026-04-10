# Общая логика контекстного меню OmegaZip для Windows (паритет с macOS: pick_ext_auto, stem, пресеты).
# Вызывается из install-context-menu-windows.ps1; не запускайте руками без параметров.
# Синхронизировать расширения pick_ext с scripts/install-context-menu.sh (pick_ext_auto).

param(
    [Parameter(Mandatory)]
    [string]$OmegaZipExe,
    [Parameter(Mandatory, Position = 0)]
    [ValidateSet('CompressAuto', 'CompressOz', 'CompressZip', 'Extract')]
    [string]$Mode,
    [Parameter(Mandatory, Position = 1)]
    [string]$LiteralPath
)

$ErrorActionPreference = 'Stop'
$OmegaZipExe = $OmegaZipExe.Trim('"').Trim()
$LiteralPath = $LiteralPath.Trim('"').Trim()

if (-not (Test-Path -LiteralPath $OmegaZipExe)) {
    Write-Error "OmegaZip не найден: $OmegaZipExe"
    exit 1
}
if (-not (Test-Path -LiteralPath $LiteralPath)) {
    Write-Error "Путь не найден: $LiteralPath"
    exit 1
}

# Последний сегмент расширения (как ${base##*.} в bash) — см. install-context-menu.sh
$ZipLikeExt = @(
    'zip', '7z', 'rar', 'tar', 'gz', 'tgz', 'bz2', 'xz', 'zst', 'lz4', 'lzma', 'cab', 'ar', 'cpio',
    'xpi', 'crx', 'jar', 'war', 'ear', 'apk', 'ipa', 'msix',
    'jpg', 'jpeg', 'pjpeg', 'png', 'gif', 'bmp', 'webp', 'tif', 'tiff', 'heic', 'heif', 'avif', 'ico', 'jxl',
    'psd', 'dds', 'exr', 'dng', 'cr2', 'nef', 'orf', 'srw', 'svgz',
    'mp4', 'm4v', 'mkv', 'avi', 'mov', 'webm', 'mpeg', 'mpg', 'm2v', 'wmv', 'flv', '3gp', 'ogv', 'ts', 'mts', 'm2ts', 'vob', 'asf', 'f4v',
    'mp3', 'flac', 'wav', 'aac', 'm4a', 'm4b', 'ogg', 'opus', 'wma', 'aiff', 'aif', 'mpc', 'wv', 'ape', 'caf',
    'woff', 'woff2', 'otf', 'ttf', 'eot',
    'exe', 'dll', 'dylib', 'so', 'bin', 'com', 'msi', 'pyc', 'pyo', 'o', 'a', 'lib', 'class', 'dex', 'pak', 'nib', 'wasm', 'pdb',
    'dmg', 'iso', 'img', 'vmdk', 'vdi', 'qcow2', 'hdd', 'vhd', 'sparseimage', 'sqlite', 'db-shm', 'db-wal'
)

function Get-OmegaZipStem {
    param([string]$FullPath)
    $item = Get-Item -LiteralPath $FullPath -Force
    $leaf = $item.Name
    if ($item.PSIsContainer) {
        return $leaf
    }
    $lower = $leaf.ToLowerInvariant()
    $pairs = @(
        @('.tar.gz', 7), @('.tar.bz2', 8), @('.tar.xz', 7), @('.tar.zst', 8),
        @('.tgz', 4), @('.tbz2', 5), @('.txz', 4), @('.tzst', 5)
    )
    foreach ($p in $pairs) {
        $suf = $p[0]; $len = $p[1]
        if ($lower.EndsWith($suf)) {
            return $leaf.Substring(0, $leaf.Length - $len)
        }
    }
    return [System.IO.Path]::GetFileNameWithoutExtension($leaf)
}

function Get-PickExtAuto {
    param([string]$FullPath)
    $item = Get-Item -LiteralPath $FullPath -Force
    if ($item.PSIsContainer) {
        return 'oz'
    }
    $base = $item.Name
    if ($base -notmatch '\.([^.]+)$') {
        return 'oz'
    }
    $ext = $Matches[1].ToLowerInvariant()
    if ($ZipLikeExt -contains $ext) {
        return 'zip'
    }
    return 'oz'
}

function Read-ContextPreset {
    $envPreset = $env:OMEGAZIP_CONTEXT_PRESET
    if (-not [string]::IsNullOrWhiteSpace($envPreset)) {
        return $envPreset.Trim()
    }
    $cf = Join-Path $env:USERPROFILE '.config\omegazip\context_preset'
    if (Test-Path -LiteralPath $cf) {
        $line = Get-Content -LiteralPath $cf -ErrorAction SilentlyContinue |
            Where-Object { $_ -notmatch '^\s*#' -and $_.Trim() -ne '' } |
            Select-Object -First 1
        if ($line) {
            return ($line -split '\s+')[0].Trim()
        }
    }
    return 'auto'
}

function Read-AutoUpgradeFolderMB {
    $v = $env:OMEGAZIP_AUTO_UPGRADE_FOLDER_MB
    if (-not [string]::IsNullOrWhiteSpace($v)) {
        return $v.Trim()
    }
    $cf = Join-Path $env:USERPROFILE '.config\omegazip\auto_upgrade_folder_mb'
    if (Test-Path -LiteralPath $cf) {
        $line = Get-Content -LiteralPath $cf -ErrorAction SilentlyContinue |
            Where-Object { $_ -notmatch '^\s*#' -and $_.Trim() -ne '' } |
            Select-Object -First 1
        if ($line) {
            return ($line -split '\s+')[0].Trim()
        }
    }
    return '200'
}

function Get-DirectorySizeMB {
    param([string]$Path)
    try {
        $sum = (Get-ChildItem -LiteralPath $Path -Recurse -File -Force -ErrorAction SilentlyContinue |
            Measure-Object -Property Length -Sum -ErrorAction SilentlyContinue).Sum
        if ($null -eq $sum) { return 0 }
        return [double]($sum / 1MB)
    }
    catch {
        return 0
    }
}

function Get-EffectivePreset {
    param([string]$Path, [string]$BasePreset)
    $mbStr = Read-AutoUpgradeFolderMB
    if ($mbStr -eq '0') {
        return $BasePreset
    }
    $mb = 200
    if (-not [int]::TryParse($mbStr, [ref]$mb)) {
        $mb = 200
    }
    $item = Get-Item -LiteralPath $Path -Force
    if ($BasePreset -ne 'auto' -or -not $item.PSIsContainer) {
        return $BasePreset
    }
    $sizeMb = Get-DirectorySizeMB -Path $Path
    if ($sizeMb -ge $mb) {
        return 'max'
    }
    return 'auto'
}

function Invoke-OmegaZipCompressOz {
    param([string]$Src, [string]$Out, [string]$Preset)
    switch -Regex ($Preset.ToLowerInvariant()) {
        '^(max|aggressive)$' { & $OmegaZipExe compress --preset max $Src $Out; break }
        '^ultra$' { & $OmegaZipExe compress --preset ultra $Src $Out; break }
        '^fast$' { & $OmegaZipExe compress --preset fast $Src $Out; break }
        '^balanced$' { & $OmegaZipExe compress --preset balanced $Src $Out; break }
        Default { & $OmegaZipExe compress --preset auto $Src $Out }
    }
}

$dir = Split-Path -Parent $LiteralPath
$stem = Get-OmegaZipStem -FullPath $LiteralPath
if ([string]::IsNullOrWhiteSpace($stem)) {
    $stem = 'archive'
}

switch ($Mode) {
    'CompressOz' {
        $out = Join-Path $dir "$stem.oz"
        $base = Read-ContextPreset
        $eff = Get-EffectivePreset -Path $LiteralPath -BasePreset $base
        Invoke-OmegaZipCompressOz -Src $LiteralPath -Out $out -Preset $eff
    }
    'CompressZip' {
        $out = Join-Path $dir "$stem.zip"
        & $OmegaZipExe compress $LiteralPath $out
    }
    'CompressAuto' {
        $ext = Get-PickExtAuto -FullPath $LiteralPath
        $out = Join-Path $dir "$stem.$ext"
        if ($ext -eq 'oz') {
            $base = Read-ContextPreset
            $eff = Get-EffectivePreset -Path $LiteralPath -BasePreset $base
            Invoke-OmegaZipCompressOz -Src $LiteralPath -Out $out -Preset $eff
        }
        else {
            & $OmegaZipExe compress $LiteralPath $out
        }
    }
    'Extract' {
        $item = Get-Item -LiteralPath $LiteralPath -Force
        if ($item.PSIsContainer) {
            Write-Error "Распаковка: выберите файл архива, а не папку."
            exit 1
        }
        $out = Join-Path $dir "${stem}_распаковано"
        & $OmegaZipExe decompress $LiteralPath $out
    }
}
