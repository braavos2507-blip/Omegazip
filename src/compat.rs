//! Совместимые архивы: ZIP/tar/gzip/xz/bzip2/zstd/CAB в Rust; 7z/RAR/ISO/WIM/MSI и др. через 7-Zip (7z|7zz|7za в PATH).

use std::env;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use walkdir::WalkDir;
use xz2::read::XzDecoder;
use zip::result::ZipError;
use zip::write::ZipWriter;
use zstd::stream::read::Decoder as ZstdDecoder;

use cab::Cabinet;

/// Подсказка, если нет 7-Zip в PATH (нужен для 7z, RAR, ISO и многих «системных» контейнеров).
pub const SEVENZIP_PATH_HINT: &str =
    "Установите 7-Zip (https://www.7-zip.org/) или p7zip: в PATH должны быть 7z, 7zz или 7za. На Windows при установке в стандартную папку OmegaZip часто находит 7z.exe без PATH. Подробности: команда «omegazip deps» или блок статуса в начале окна приложения.";

const ZSTD_MAGIC: [u8; 4] = [0x28, 0xb5, 0x2f, 0xfd];
const SIG_7Z: [u8; 6] = [0x37, 0x7a, 0xbc, 0xaf, 0x27, 0x1c];
const SIG_RAR: [u8; 7] = [0x52, 0x61, 0x72, 0x21, 0x1a, 0x07, 0x00];
const SIG_RAR_ALT: [u8; 6] = [0x52, 0x61, 0x72, 0x21, 0x1a, 0x07];
const SIG_CAB: [u8; 4] = [0x4d, 0x53, 0x43, 0x46];

fn map_zip_error(e: ZipError) -> Box<dyn std::error::Error + Send + Sync> {
    match e {
        ZipError::UnsupportedArchive(msg) if msg == ZipError::PASSWORD_REQUIRED => {
            "ZIP зашифрован паролем: OmegaZip распаковывает только незашифрованные ZIP. Архивы .oz с паролем поддерживаются отдельно.".into()
        }
        other => other.into(),
    }
}

fn safe_join(base: &Path, rel: &Path) -> io::Result<PathBuf> {
    if rel.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "absolute path inside archive",
        ));
    }
    for c in rel.components() {
        if matches!(c, Component::ParentDir) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path traversal in archive",
            ));
        }
    }
    Ok(base.join(rel))
}

fn read_head(path: &Path, n: usize) -> io::Result<Vec<u8>> {
    let mut f = File::open(path)?;
    let mut buf = vec![0u8; n];
    let got = f.read(&mut buf)?;
    buf.truncate(got);
    Ok(buf)
}

pub fn resolve_7z_executable() -> Option<PathBuf> {
    #[cfg(target_os = "android")]
    {
        return None;
    }

    const NAMES_UNIX: &[&str] = &["7zz", "7z", "7za"];
    const NAMES_WIN: &[&str] = &["7z.exe", "7zz.exe", "7za.exe"];
    let names: &[&str] = if cfg!(windows) {
        NAMES_WIN
    } else {
        NAMES_UNIX
    };

    if let Some(path_var) = env::var_os("PATH") {
        for dir in env::split_paths(&path_var) {
            for name in names {
                let p = dir.join(name);
                if p.is_file() {
                    return Some(p);
                }
            }
        }
    }

    #[cfg(windows)]
    {
        if let Some(pf) = env::var_os("ProgramFiles") {
            for sub in ["7-Zip\\7z.exe", "7-Zip\\7zz.exe"] {
                let p = PathBuf::from(&pf).join(sub);
                if p.is_file() {
                    return Some(p);
                }
            }
        }
        if let Some(pf) = env::var_os("ProgramFiles(x86)") {
            for sub in ["7-Zip\\7z.exe", "7-Zip\\7zz.exe"] {
                let p = PathBuf::from(&pf).join(sub);
                if p.is_file() {
                    return Some(p);
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Homebrew p7zip: /opt/homebrew/bin или /usr/local/bin — обычно уже в PATH;
        // на всякий случай проверяем типичные префиксы.
        for prefix in ["/opt/homebrew/bin", "/usr/local/bin", "/opt/local/bin"] {
            for name in NAMES_UNIX {
                let p = PathBuf::from(prefix).join(name);
                if p.is_file() {
                    return Some(p);
                }
            }
        }
    }

    None
}

/// Заметка о формате RAR для UI и справки (создание не поддерживается по лицензии/спецификации).
pub const RAR_FORMAT_NOTE: &str = "Архивы RAR можно только распаковывать (через 7-Zip). Создание .rar как в WinRAR не поддерживается — формат проприетарный.";

/// Состояние внешнего 7-Zip / p7zip для GUI, CLI и документации.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SevenZipStatus {
    pub available: bool,
    pub executable: Option<String>,
    pub command_name: Option<String>,
    pub path_hint: String,
    pub install_howto: String,
    pub rar_note: String,
}

/// Как установить 7-Zip / p7zip на текущей платформе (текст для подсказки).
pub fn seven_zip_install_howto() -> String {
    if cfg!(target_os = "macos") {
        "macOS\n\
         • Установка: brew install p7zip — в PATH появится обычно 7zz\n\
         • Либо скачайте 7-Zip с https://www.7-zip.org/ и добавьте каталог с бинарником в PATH\n\
         • После установки перезапустите OmegaZip (или терминал), чтобы обновился PATH"
            .to_string()
    } else if cfg!(target_os = "windows") {
        "Windows\n\
         • Скачайте установщик: https://www.7-zip.org/\n\
         • При установке в стандартную папку OmegaZip находит 7z.exe в Program Files даже без PATH\n\
         • При переносном варианте добавьте каталог с 7z.exe в переменную PATH"
            .to_string()
    } else if cfg!(target_os = "linux") {
        "Linux\n\
         • Debian/Ubuntu: sudo apt install p7zip-full\n\
         • Fedora: sudo dnf install p7zip p7zip-plugins\n\
         • Arch: sudo pacman -S p7zip\n\
         • Нужна команда 7z или 7zz в PATH (проверка: 7z —)"
            .to_string()
    } else if cfg!(target_os = "android") {
        "OmegaZip Android\n\
         • Внешний процесс 7-Zip на устройстве не используется.\n\
         • Нативно: .oz, ZIP, tar (включая .gz/.xz/.bz2/.zst), одиночные сжатые потоки, CAB.\n\
         • RAR, .7z, ISO, MSI — откройте на ПК (OmegaZip + 7-Zip) или другим приложением из магазина."
            .to_string()
    } else {
        "Установите пакет p7zip или 7-Zip для вашей ОС и добавьте 7z / 7zz / 7za в PATH.\n\
         Сайт: https://www.7-zip.org/"
            .to_string()
    }
}

/// Полный статус 7-Zip для отображения пользователю.
pub fn seven_zip_status() -> SevenZipStatus {
    let exe = resolve_7z_executable();
    let (path_hint, rar_note) = if cfg!(target_os = "android") {
        (
            "На Android OmegaZip Android не подключает внешний 7-Zip — только встроенные форматы (см. текст ниже)."
                .to_string(),
            "RAR и архивы 7z на этом устройстве здесь не открываются. Создание .rar не поддерживается и на ПК. Для RAR/7z используйте десктоп OmegaZip с 7-Zip или приложение из Google Play."
                .to_string(),
        )
    } else {
        (SEVENZIP_PATH_HINT.to_string(), RAR_FORMAT_NOTE.to_string())
    };
    SevenZipStatus {
        available: exe.is_some(),
        executable: exe.as_ref().map(|p| p.to_string_lossy().into_owned()),
        command_name: exe
            .as_ref()
            .and_then(|p| p.file_name().map(|s| s.to_string_lossy().into_owned())),
        path_hint,
        install_howto: seven_zip_install_howto(),
        rar_note,
    }
}

fn sevenz_missing_error() -> Box<dyn std::error::Error + Send + Sync> {
    format!("{SEVENZIP_PATH_HINT}\n\n{}", seven_zip_install_howto()).into()
}

fn sevenz_o_switch(out_dir: &Path) -> io::Result<String> {
    let abs = fs::canonicalize(out_dir).unwrap_or_else(|_| out_dir.to_path_buf());
    let mut s = String::from("-o");
    let p = abs.to_string_lossy();
    s.push_str(&p);
    if !p.ends_with('/') && !p.ends_with('\\') {
        s.push(std::path::MAIN_SEPARATOR);
    }
    Ok(s)
}

fn parse_7z_slt_listing(stdout: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut pending: Option<String> = None;
    let mut is_folder = false;
    for line in stdout.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("Path = ") {
            if let Some(p) = pending.take() {
                if !is_folder && !p.is_empty() && !p.ends_with('/') {
                    out.push(p.replace('\\', "/"));
                }
            }
            pending = Some(rest.trim().to_string());
            is_folder = false;
            continue;
        }
        if t == "Folder = +" {
            is_folder = true;
        }
    }
    if let Some(p) = pending {
        if !is_folder && !p.is_empty() && !p.ends_with('/') {
            out.push(p.replace('\\', "/"));
        }
    }
    out
}

fn run_7z_list(
    archive: &Path,
    password: Option<&str>,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let exe = resolve_7z_executable().ok_or_else(sevenz_missing_error)?;
    let mut cmd = Command::new(&exe);
    cmd.arg("l").arg("-slt").arg("-bd").arg(archive);
    if let Some(p) = password {
        cmd.arg(format!("-p{p}"));
    }
    let output = cmd.output()?;
    if !output.status.success() {
        let e = String::from_utf8_lossy(&output.stderr);
        return Err(if e.trim().is_empty() {
            format!(
                "7-Zip: не удалось открыть архив (код {}). {}\n\n{}",
                output.status.code().unwrap_or(-1),
                SEVENZIP_PATH_HINT,
                seven_zip_install_howto()
            )
            .into()
        } else {
            e.trim().into()
        });
    }
    let s = String::from_utf8_lossy(&output.stdout);
    Ok(parse_7z_slt_listing(&s))
}

fn run_7z_extract(
    archive: &Path,
    out_dir: &Path,
    password: Option<&str>,
) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let exe = resolve_7z_executable().ok_or_else(sevenz_missing_error)?;
    let o_switch = sevenz_o_switch(out_dir)?;
    let mut cmd = Command::new(&exe);
    cmd.arg("x").arg("-y").arg("-bd").arg(&o_switch).arg(archive);
    if let Some(p) = password {
        cmd.arg(format!("-p{p}"));
    }
    let output = cmd.output()?;
    if !output.status.success() {
        let e = String::from_utf8_lossy(&output.stderr);
        return Err(if e.trim().is_empty() {
            format!("7-Zip: ошибка распаковки (код {})", output.status.code().unwrap_or(-1)).into()
        } else {
            e.trim().into()
        });
    }
    let listed = run_7z_list(archive, password)?;
    Ok(listed.len() as u32)
}

fn sevenz_can_open(archive: &Path) -> bool {
    let Some(exe) = resolve_7z_executable() else {
        return false;
    };
    Command::new(exe)
        .arg("t")
        .arg("-bd")
        .arg(archive)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn extension_prefers_7z(name_lower: &str) -> bool {
    name_lower.ends_with(".7z")
        || name_lower.ends_with(".rar")
        || name_lower.ends_with(".cbr")
        || name_lower.ends_with(".iso")
        || name_lower.ends_with(".img")
        || name_lower.ends_with(".wim")
        || name_lower.ends_with(".swm")
        || name_lower.ends_with(".esd")
        || name_lower.ends_with(".arj")
        || name_lower.ends_with(".lzh")
        || name_lower.ends_with(".lha")
        || name_lower.ends_with(".rpm")
        || name_lower.ends_with(".deb")
        || name_lower.ends_with(".msi")
        || name_lower.ends_with(".dmg")
        || name_lower.ends_with(".xar")
        || name_lower.ends_with(".pkg")
        || name_lower.ends_with(".chm")
        || name_lower.ends_with(".lzma")
        || name_lower.ends_with(".cpio")
        || name_lower.ends_with(".vhd")
        || name_lower.ends_with(".vhdx")
        || name_lower.ends_with(".ntfs")
        || name_lower.ends_with(".fat")
        || name_lower.ends_with(".squashfs")
        || name_lower.ends_with(".lz")
}

fn magic_prefers_7z(head: &[u8]) -> bool {
    (head.len() >= 6 && head[..6] == SIG_7Z)
        || (head.len() >= 7 && head[..7] == SIG_RAR)
        || (head.len() >= 6 && head[..6] == SIG_RAR_ALT)
}

fn looks_like_zip(name: &str) -> bool {
    name.ends_with(".zip")
        || name.ends_with(".jar")
        || name.ends_with(".war")
        || name.ends_with(".apk")
        || name.ends_with(".ipa")
        || name.ends_with(".docx")
        || name.ends_with(".xlsx")
        || name.ends_with(".pptx")
        || name.ends_with(".odt")
        || name.ends_with(".ods")
        || name.ends_with(".odp")
        || name.ends_with(".cbz")
}

fn is_ustar_file(path: &Path) -> io::Result<bool> {
    let mut f = File::open(path)?;
    let mut b = [0u8; 264];
    let n = f.read(&mut b)?;
    if n < 263 {
        return Ok(false);
    }
    Ok(&b[257..263] == b"ustar\0" || &b[257..263] == b"ustar ")
}

fn list_zip(archive: &Path) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let file = File::open(archive)?;
    let mut z = zip::ZipArchive::new(file)?;
    let mut out = Vec::with_capacity(z.len());
    for i in 0..z.len() {
        let n = z.by_index(i).map_err(map_zip_error)?.name().to_string();
        if !n.ends_with('/') {
            out.push(n);
        }
    }
    Ok(out)
}

fn list_tar_stream<R: Read>(r: R) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let mut ar = tar::Archive::new(r);
    let mut out = Vec::new();
    for e in ar.entries()? {
        let e = e?;
        if e.header().entry_type().is_dir() {
            continue;
        }
        let p = e.path()?;
        out.push(p.to_string_lossy().into_owned());
    }
    Ok(out)
}

fn list_cab(archive: &Path) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let file = File::open(archive)?;
    let cab = Cabinet::new(file)?;
    let mut out = Vec::new();
    for folder in cab.folder_entries() {
        for file in folder.file_entries() {
            out.push(file.name().to_string());
        }
    }
    Ok(out)
}

fn extract_cab(archive: &Path, out_dir: &Path) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let file = File::open(archive)?;
    let mut cab = Cabinet::new(file)?;
    let names: Vec<String> = {
        let mut n = Vec::new();
        for folder in cab.folder_entries() {
            for fe in folder.file_entries() {
                n.push(fe.name().to_string());
            }
        }
        n
    };
    let mut count = 0u32;
    for name in names {
        let target = safe_join(out_dir, Path::new(&name))?;
        if let Some(p) = target.parent() {
            fs::create_dir_all(p)?;
        }
        let mut r = cab.read_file(&name)?;
        let mut out = File::create(&target)?;
        io::copy(&mut r, &mut out)?;
        count += 1;
    }
    Ok(count)
}

/// Список имён (файлов) в архиве; `password` передаётся в 7-Zip при необходимости.
pub fn list_foreign(archive: &Path) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    list_foreign_with_password(archive, None)
}

pub fn list_foreign_with_password(
    archive: &Path,
    password: Option<&str>,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let name = archive
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    if looks_like_zip(&name) {
        return list_zip(archive);
    }
    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        return list_tar_stream(GzDecoder::new(File::open(archive)?));
    }
    if name.ends_with(".tar.xz") || name.ends_with(".txz") {
        return list_tar_stream(XzDecoder::new(File::open(archive)?));
    }
    if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") || name.ends_with(".tbz") {
        return list_tar_stream(BzDecoder::new(File::open(archive)?));
    }
    if name.ends_with(".tar.zst") || name.ends_with(".tzst") {
        return list_tar_stream(ZstdDecoder::new(File::open(archive)?)?);
    }
    if name.ends_with(".tar") {
        return list_tar_stream(File::open(archive)?);
    }
    if name.ends_with(".cab") {
        return list_cab(archive);
    }
    if name.ends_with(".zst") {
        return Ok(vec![
            archive
                .file_stem()
                .and_then(|s| s.to_str())
                .filter(|s| !s.is_empty())
                .unwrap_or("out")
                .to_string(),
        ]);
    }

    if extension_prefers_7z(&name) {
        return run_7z_list(archive, password);
    }

    let head = read_head(archive, 8)?;
    if head.len() >= 4 && head.starts_with(&SIG_CAB) {
        return list_cab(archive);
    }
    if head.len() >= 4 && head[..4] == ZSTD_MAGIC {
        if name.ends_with(".tar.zst") || name.ends_with(".tzst") {
            return list_tar_stream(ZstdDecoder::new(File::open(archive)?)?);
        }
        return Ok(vec![archive.file_name().and_then(|s| s.to_str()).unwrap_or("out").to_string()]);
    }
    if head.len() >= 4 && head[0] == 0x50 && head[1] == 0x4b {
        return list_zip(archive);
    }
    if magic_prefers_7z(&head) {
        return run_7z_list(archive, password);
    }
    if is_ustar_file(archive)? {
        return list_tar_stream(File::open(archive)?);
    }
    if sevenz_can_open(archive) {
        return run_7z_list(archive, password);
    }

    let mut err = format!(
        "Формат не распознан. Нативно: ZIP, tar, tar.gz/xz/bz2/zst, gz, xz, bz2, CAB, zst. Остальное — через 7-Zip ({}).",
        SEVENZIP_PATH_HINT
    );
    if resolve_7z_executable().is_none() {
        err.push_str("\n\n");
        err.push_str(&seven_zip_install_howto());
    }
    Err(err.into())
}

/// Распаковка «чужого» архива.
pub fn extract_foreign(archive: &Path, out_dir: &Path) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    extract_foreign_with_password(archive, out_dir, None)
}

pub fn extract_foreign_with_password(
    archive: &Path,
    out_dir: &Path,
    password: Option<&str>,
) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    fs::create_dir_all(out_dir)?;
    let name = archive
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    if looks_like_zip(&name) {
        return extract_zip(archive, out_dir);
    }
    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        return extract_tar_stream(GzDecoder::new(File::open(archive)?), out_dir);
    }
    if name.ends_with(".tar.xz") || name.ends_with(".txz") {
        return extract_tar_stream(XzDecoder::new(File::open(archive)?), out_dir);
    }
    if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") || name.ends_with(".tbz") {
        return extract_tar_stream(BzDecoder::new(File::open(archive)?), out_dir);
    }
    if name.ends_with(".tar.zst") || name.ends_with(".tzst") {
        return extract_tar_stream(ZstdDecoder::new(File::open(archive)?)?, out_dir);
    }
    if name.ends_with(".tar") {
        return extract_tar_stream(File::open(archive)?, out_dir);
    }
    if name.ends_with(".gz") && !name.contains(".tar.") {
        return gunzip_one(archive, out_dir);
    }
    if name.ends_with(".xz") && !name.contains(".tar.") {
        return xz_one(archive, out_dir);
    }
    if name.ends_with(".bz2") && !name.contains(".tar.") {
        return bz2_one(archive, out_dir);
    }
    if name.ends_with(".zst") && !name.contains(".tar.") {
        return zstd_one(archive, out_dir);
    }
    if name.ends_with(".cab") {
        return extract_cab(archive, out_dir);
    }

    if extension_prefers_7z(&name) {
        return run_7z_extract(archive, out_dir, password);
    }

    let mut f = File::open(archive)?;
    let mut hdr = [0u8; 8];
    let n = f.read(&mut hdr)?;
    let hdr = &hdr[..n];

    if hdr.len() >= 4 && hdr.starts_with(&SIG_CAB) {
        return extract_cab(archive, out_dir);
    }
    if hdr.len() >= 4 && hdr[..4] == ZSTD_MAGIC {
        if name.ends_with(".tar.zst") || name.ends_with(".tzst") {
            return extract_tar_stream(ZstdDecoder::new(File::open(archive)?)?, out_dir);
        }
        return zstd_one(archive, out_dir);
    }
    if hdr.len() >= 4 && hdr[0] == 0x50 && hdr[1] == 0x4b {
        return extract_zip(archive, out_dir);
    }
    if hdr.len() >= 2 && hdr[0] == 0x1f && hdr[1] == 0x8b {
        return gunzip_one(archive, out_dir);
    }
    if hdr.len() >= 6
        && hdr[0] == 0xfd
        && hdr[1] == 0x37
        && hdr[2] == 0x7a
        && hdr[3] == 0x58
        && hdr[4] == 0x5a
    {
        return xz_one(archive, out_dir);
    }
    if hdr.len() >= 2 && hdr[0] == b'B' && hdr[1] == b'Z' {
        return bz2_one(archive, out_dir);
    }
    if magic_prefers_7z(hdr) {
        return run_7z_extract(archive, out_dir, password);
    }
    if is_ustar_file(archive)? {
        return extract_tar_stream(File::open(archive)?, out_dir);
    }
    if sevenz_can_open(archive) {
        return run_7z_extract(archive, out_dir, password);
    }

    let mut err = format!(
        "Формат не поддерживается без 7-Zip. Нативно: ZIP, tar*, gz, xz, bz2, zst, CAB. {}",
        SEVENZIP_PATH_HINT
    );
    if resolve_7z_executable().is_none() {
        err.push_str("\n\n");
        err.push_str(&seven_zip_install_howto());
    } else {
        err.push_str("\n\n7-Zip установлен, но архив не открылся — возможно, файл повреждён или редкий вариант контейнера.");
    }
    Err(err.into())
}

/// Расширения и суффиксы, при которых GUI переводит сценарий в «распаковку» (`ui/index.html` — `isExtractArchivePath`).
/// Синхронизировать при изменении списка в UI.
pub fn looks_like_supported_archive_path(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    if name.ends_with(".oz") {
        return true;
    }
    name.ends_with(".zip")
        || name.ends_with(".7z")
        || name.ends_with(".rar")
        || name.ends_with(".cbr")
        || name.ends_with(".iso")
        || name.ends_with(".wim")
        || name.ends_with(".msi")
        || name.ends_with(".dmg")
        || name.ends_with(".deb")
        || name.ends_with(".rpm")
        || name.ends_with(".cab")
        || name.ends_with(".zst")
        || name.ends_with(".tzst")
        || name.ends_with(".tar.zst")
        || name.ends_with(".jar")
        || name.ends_with(".war")
        || name.ends_with(".apk")
        || name.ends_with(".ipa")
        || name.ends_with(".tar")
        || name.ends_with(".tar.gz")
        || name.ends_with(".tgz")
        || name.ends_with(".tar.xz")
        || name.ends_with(".txz")
        || name.ends_with(".tar.bz2")
        || name.ends_with(".tbz2")
        || name.ends_with(".tbz")
        || name.ends_with(".gz")
        || name.ends_with(".xz")
        || name.ends_with(".bz2")
        || name.ends_with(".docx")
        || name.ends_with(".xlsx")
        || name.ends_with(".pptx")
        || name.ends_with(".odt")
        || name.ends_with(".ods")
        || name.ends_with(".odp")
        || name.ends_with(".cbz")
        || name.ends_with(".xar")
        || name.ends_with(".lzma")
        || name.ends_with(".arj")
        || name.ends_with(".lzh")
        || name.ends_with(".lha")
}

fn extract_zip(archive: &Path, out_dir: &Path) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let file = File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;
    let mut count = 0u32;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(map_zip_error)?;
        let raw = entry.name().to_string();
        let rel = Path::new(raw.trim_start_matches('/'));
        let target = safe_join(out_dir, rel)?;
        if raw.ends_with('/') || entry.is_dir() {
            fs::create_dir_all(&target)?;
            continue;
        }
        if let Some(p) = target.parent() {
            fs::create_dir_all(p)?;
        }
        let mut out = File::create(&target)?;
        io::copy(&mut entry, &mut out)?;
        count += 1;
    }
    Ok(count)
}

fn extract_tar_stream<R: Read>(
    r: R,
    out_dir: &Path,
) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let mut ar = tar::Archive::new(r);
    let mut count = 0u32;
    for e in ar.entries()? {
        let mut e = e?;
        let path = e.path()?;
        let target = safe_join(out_dir, path.as_ref())?;
        if e.header().entry_type().is_dir() {
            fs::create_dir_all(&target)?;
            continue;
        }
        if let Some(p) = target.parent() {
            fs::create_dir_all(p)?;
        }
        let mut out = File::create(&target)?;
        io::copy(&mut e, &mut out)?;
        count += 1;
    }
    Ok(count)
}

fn gunzip_one(archive: &Path, out_dir: &Path) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let stem = archive
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("out");
    let out_path = out_dir.join(stem);
    let mut dec = GzDecoder::new(File::open(archive)?);
    let mut buf = Vec::new();
    dec.read_to_end(&mut buf)?;
    fs::write(&out_path, &buf)?;
    Ok(1)
}

fn xz_one(archive: &Path, out_dir: &Path) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let stem = archive
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("out");
    let out_path = out_dir.join(stem);
    let mut dec = XzDecoder::new(File::open(archive)?);
    let mut buf = Vec::new();
    dec.read_to_end(&mut buf)?;
    fs::write(&out_path, &buf)?;
    Ok(1)
}

fn bz2_one(archive: &Path, out_dir: &Path) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let stem = archive
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("out");
    let out_path = out_dir.join(stem);
    let mut dec = BzDecoder::new(File::open(archive)?);
    let mut buf = Vec::new();
    dec.read_to_end(&mut buf)?;
    fs::write(&out_path, &buf)?;
    Ok(1)
}

fn zstd_one(archive: &Path, out_dir: &Path) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let stem = archive
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("out");
    let out_path = out_dir.join(stem);
    let mut dec = ZstdDecoder::new(File::open(archive)?)?;
    let mut buf = Vec::new();
    dec.read_to_end(&mut buf)?;
    fs::write(&out_path, &buf)?;
    Ok(1)
}

/// Папка или один файл → ZIP (Deflate).
pub fn compress_to_zip(source: &Path, zip_path: &Path) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let file = File::create(zip_path)?;
    let mut zw = ZipWriter::new(file);
    let opts = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut count = 0u32;

    if source.is_dir() {
        let base = source.canonicalize().unwrap_or_else(|_| source.to_path_buf());
        for entry in WalkDir::new(&base).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let rel = path.strip_prefix(&base).unwrap_or(path);
            let name = rel.to_string_lossy().replace('\\', "/");
            zw.start_file(&name, opts)?;
            let mut f = File::open(path)?;
            io::copy(&mut f, &mut zw)?;
            count += 1;
        }
    } else if source.is_file() {
        let name = source
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "file".to_string());
        zw.start_file(&name, opts)?;
        let mut f = File::open(source)?;
        io::copy(&mut f, &mut zw)?;
        count += 1;
    } else {
        return Err("источник не найден".into());
    }
    zw.finish()?;
    Ok(count)
}

/// Папка или файл → tar.gz (как в типичных архиваторах).
pub fn compress_to_tar_gz(source: &Path, dest: &Path) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let enc = GzEncoder::new(File::create(dest)?, Compression::default());
    let mut builder = tar::Builder::new(enc);
    let mut count = 0u32;
    if source.is_dir() {
        let base = source.canonicalize().unwrap_or_else(|_| source.to_path_buf());
        for entry in WalkDir::new(&base).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let rel = path.strip_prefix(&base).unwrap_or(path);
            let name = rel.to_string_lossy().replace('\\', "/");
            builder.append_path_with_name(path, name)?;
            count += 1;
        }
    } else if source.is_file() {
        let name = source
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "file".to_string());
        builder.append_path_with_name(source, name)?;
        count = 1;
    } else {
        return Err("источник не найден".into());
    }
    builder.finish()?;
    let gz = builder.into_inner().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e:?}")))?;
    gz.finish()?;
    Ok(count)
}

/// Сжатие в .7z через 7-Zip (если установлен).
pub fn compress_to_7z(
    source: &Path,
    dest_7z: &Path,
    password: Option<&str>,
) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let exe = resolve_7z_executable().ok_or_else(sevenz_missing_error)?;
    let dest_abs = if dest_7z.is_absolute() {
        dest_7z.to_path_buf()
    } else {
        env::current_dir()?.join(dest_7z)
    };
    if let Some(p) = dest_abs.parent() {
        fs::create_dir_all(p)?;
    }
    let source = source.canonicalize().unwrap_or_else(|_| source.to_path_buf());

    let mut count = 0u32;
    let mut cmd = Command::new(&exe);
    cmd.arg("a")
        .arg("-t7z")
        .arg("-mx=5")
        .arg("-y")
        .arg("-bd")
        .arg(&dest_abs);
    if let Some(p) = password {
        cmd.arg(format!("-p{p}"));
    }
    if source.is_dir() {
        cmd.current_dir(&source);
        for entry in WalkDir::new(".")
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                cmd.arg(entry.path());
                count += 1;
            }
        }
        if count == 0 {
            return Err("в папке нет файлов".into());
        }
    } else if source.is_file() {
        cmd.arg(&source);
        count = 1;
    } else {
        return Err("источник не найден".into());
    }

    let out = cmd.output()?;
    if !out.status.success() {
        let e = String::from_utf8_lossy(&out.stderr);
        return Err(if e.trim().is_empty() {
            format!("7-Zip: не удалось создать архив (код {})", out.status.code().unwrap_or(-1)).into()
        } else {
            e.trim().into()
        });
    }
    Ok(count)
}

pub fn output_is_zip(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("zip"))
        .unwrap_or(false)
}

pub fn output_is_tar_gz(path: &Path) -> bool {
    let s = path.to_string_lossy().to_lowercase();
    s.ends_with(".tar.gz") || s.ends_with(".tgz")
}

pub fn output_is_7z(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("7z"))
        .unwrap_or(false)
}

#[cfg(test)]
mod looks_like_tests {
    use super::looks_like_supported_archive_path;
    use std::path::Path;

    #[test]
    fn known_suffixes() {
        assert!(looks_like_supported_archive_path(Path::new("a.zip")));
        assert!(looks_like_supported_archive_path(Path::new("/tmp/X.7Z")));
        assert!(looks_like_supported_archive_path(Path::new("b.tar.gz")));
        assert!(!looks_like_supported_archive_path(Path::new("readme.txt")));
    }
}
