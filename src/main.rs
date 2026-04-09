//! CLI: конвейер + пресеты, recovery, info, list, пароль из env.

use omegazip::{
    archive_info, compress_advanced_dispatch, compress_dispatch, decompress_any_to_path,
    decompress_any_to_path_with_password, export_to_zip, list_any_archive,
    list_any_archive_with_password, repo_init, repo_backup, repo_restore, list_snapshots, repo_push,
    seven_zip_status, suggested_preset_for_path, CompressOptions, Preset, DEFAULT_CHUNK_SIZE,
};
use std::path::{Path, PathBuf};
use std::{env, fs};

/// Если выходной файл не указан: `<имя_как_у_источника>.oz` (у файла — без последнего расширения).
fn default_compress_output_from_input(input: &Path) -> PathBuf {
    let file_name = input
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("archive");
    let base = input
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(file_name);
    PathBuf::from(format!("{base}.oz"))
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "compress" => {
            let (input, output, opts) = parse_compress_args(&args)?;
            let n = if opts.chunk_size.is_some()
                || opts.solid
                || opts.password.is_some()
                || opts.recovery_parity > 0
                || opts.preset.is_some()
            {
                compress_advanced_dispatch(&input, &output, opts)?
            } else {
                compress_dispatch(&input, &output)?
            };
            println!("Written {} -> {} ({} files)", input.display(), output.display(), n);
        }
        "decompress" => {
            let (archive, out_dir, password) = parse_decompress_args(&args)?;
            let n = if let Some(p) = password {
                decompress_any_to_path_with_password(&archive, &out_dir, Some(&p))?
            } else {
                decompress_any_to_path(&archive, &out_dir)?
            };
            println!("Extracted {} files to {}", n, out_dir.display());
        }
        "export-zip" => {
            let (archive, zip_path, password) = parse_export_args(&args)?;
            let n = export_to_zip(&archive, &zip_path, password.as_deref())?;
            println!("Exported {} files to {}", n, zip_path.display());
        }
        "info" => {
            let archive = args.get(2).map(PathBuf::from).ok_or("info: укажите архив")?;
            if let Ok(info) = archive_info(&archive) {
                println!("Format: OmegaZip (.oz)");
                println!("Version: {}", info.version);
                println!("Files: {}", info.file_count);
                println!("Encrypted: {}", info.encrypted);
                println!("Chunked: {}", info.chunked);
                println!("Solid: {}", info.solid);
                println!("Recovery: {}", info.recovery);
            } else {
                let paths = list_any_archive(&archive)?;
                println!("Format: совместимый архив (не .oz)");
                println!("Files: {}", paths.len());
            }
        }
        "list" => {
            let mut archive = None;
            let mut password = None;
            let mut password_file = None;
            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--password" => {
                        i += 1;
                        if i < args.len() {
                            password = Some(args[i].clone());
                            i += 1;
                        }
                    }
                    "--password-file" => {
                        i += 1;
                        if i < args.len() {
                            password_file = Some(args[i].as_str());
                            i += 1;
                        }
                    }
                    _ if !args[i].starts_with('-') => {
                        archive = Some(PathBuf::from(&args[i]));
                        i += 1;
                    }
                    _ => i += 1,
                }
            }
            let password = read_password_from_env_or_file(password, password_file);
            let archive = archive.ok_or("list: укажите архив")?;
            let paths = list_any_archive_with_password(&archive, password.as_deref())?;
            for p in &paths {
                println!("{}", p);
            }
            eprintln!("Total: {} files", paths.len());
        }
        "deps" => {
            let s = seven_zip_status();
            println!("Внешние зависимости OmegaZip\n");
            println!(
                "7-Zip / p7zip: {}",
                if s.available {
                    "найден"
                } else {
                    "не найден (нужен для .7z, RAR, ISO, WIM, MSI и др.)"
                }
            );
            if let Some(ref p) = s.executable {
                println!("  Исполняемый файл: {}", p);
            }
            if let Some(ref n) = s.command_name {
                println!("  Имя команды: {}", n);
            }
            println!("\n{}", s.rar_note);
            println!("\n--- Как установить 7-Zip ---\n{}", s.install_howto);
            if !s.available {
                println!("\n{}", s.path_hint);
            }
        }
        "repo" => {
            let sub = args.get(2).map(String::as_str).ok_or("repo: укажите init|backup|restore|push|list")?;
            match sub {
                "init" => {
                    let path = args.get(3).map(PathBuf::from).ok_or("repo init: укажите путь")?;
                    repo_init(&path)?;
                    println!("Repository initialized at {}", path.display());
                }
                "backup" => {
                    let repo_path = args.get(3).map(PathBuf::from).ok_or("repo backup: укажите путь к репо")?;
                    let source = args.get(4).map(PathBuf::from).ok_or("repo backup: укажите источник")?;
                    let id = repo_backup(&repo_path, &source)?;
                    println!("Backup snapshot: {}", id);
                }
                "restore" => {
                    let repo_path = args.get(3).map(PathBuf::from).ok_or("repo restore: укажите путь к репо")?;
                    let id: u64 = args.get(4).ok_or("repo restore: укажите id снапшота")?.parse()?;
                    let dest = args.get(5).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("restored"));
                    let n = repo_restore(&repo_path, id, &dest)?;
                    println!("Restored {} files to {}", n, dest.display());
                }
                "push" => {
                    let repo_path = args.get(3).map(PathBuf::from).ok_or("repo push: укажите путь к репо")?;
                    let dest = args.get(4).map(PathBuf::from).ok_or("repo push: укажите назначение (папка или rclone mount)")?;
                    let n = repo_push(&repo_path, &dest)?;
                    println!("Pushed {} files to {}", n, dest.display());
                }
                "list" => {
                    let repo_path = args.get(3).map(PathBuf::from).ok_or("repo list: укажите путь к репо")?;
                    let ids = list_snapshots(&repo_path)?;
                    for id in &ids {
                        println!("{}", id);
                    }
                    eprintln!("Total: {} snapshots", ids.len());
                }
                _ => eprintln!("repo: init|backup|restore|push|list"),
            }
        }
        _ => print_usage(),
    }
    Ok(())
}

fn print_usage() {
    eprintln!("OmegaZip 0.3 — chunked dedup, solid, шифрование, recovery, repo push");
    eprintln!("  compress [OPTIONS] <file|dir> <output.oz|.zip|.tar.gz|.tgz|.7z>");
    eprintln!("    --preset fast|balanced|max|ultra|maxcompression|auto   auto = по типу файлов; ultra = макс. сжатие (XZ-9 + recovery)");
    eprintln!("    --chunked [SIZE]   чанковая дедупликация (default {} bytes)", DEFAULT_CHUNK_SIZE);
    eprintln!("    --solid            solid-сжатие");
    eprintln!("    --recovery [N]     восстановительные записи (0, 1 или 2)");
    eprintln!("    --password PASS    шифрование (или OMEGAZIP_PASSWORD, --password-file)");
    eprintln!("  decompress [--password PASS] <архив> <output_dir>  — .oz, ZIP/tar/zstd/CAB + 7z/RAR/ISO… (7-Zip в PATH)");
    eprintln!("  export-zip [--password PASS] <input.oz> <output.zip>");
    eprintln!("  info <архив>        — метаданные .oz или число файлов в ZIP/tar");
    eprintln!("  list [--password PASS] <архив>  — список путей (пароль для 7-Zip/RAR и т.д.)");
    eprintln!("  deps              — проверка 7-Zip/p7zip в системе и подсказки по установке");
    eprintln!("  repo init <path>    — создать репозиторий");
    eprintln!("  repo backup <repo> <source>  — бэкап в репо");
    eprintln!("  repo restore <repo> <id> [dest]  — восстановить снапшот");
    eprintln!("  repo push <repo> <dest>  — синхронизировать репо в папку/облако (rclone mount и т.д.)");
    eprintln!("  repo list <repo>    — список снапшотов");
}

fn read_password_from_env_or_file(
    explicit: Option<String>,
    password_file: Option<&str>,
) -> Option<String> {
    if let Some(p) = explicit {
        return if p.is_empty() { None } else { Some(p) };
    }
    if let Some(path) = password_file {
        if let Ok(s) = fs::read_to_string(path) {
            let p = s.lines().next().unwrap_or("").trim().to_string();
            if !p.is_empty() {
                return Some(p);
            }
        }
    }
    env::var("OMEGAZIP_PASSWORD").ok().filter(|s| !s.is_empty())
}

fn parse_compress_args(args: &[String]) -> Result<(PathBuf, PathBuf, CompressOptions), Box<dyn std::error::Error + Send + Sync>> {
    let mut input = None;
    let mut output = None;
    let mut chunk_size = None;
    let mut solid = false;
    let mut password = None;
    let mut password_file = None;
    let mut recovery = 0u32;
    let mut preset = None;
    let mut preset_auto = false;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--preset" => {
                i += 1;
                if i < args.len() {
                    if args[i].eq_ignore_ascii_case("auto") {
                        preset_auto = true;
                    } else {
                        preset = Preset::from_str(&args[i]);
                    }
                    i += 1;
                }
            }
            "--chunked" => {
                i += 1;
                chunk_size = Some(if i < args.len() && !args[i].starts_with('-') {
                    let v = args[i].parse::<usize>().unwrap_or(DEFAULT_CHUNK_SIZE);
                    i += 1;
                    v
                } else {
                    DEFAULT_CHUNK_SIZE
                });
            }
            "--solid" => {
                solid = true;
                i += 1;
            }
            "--recovery" => {
                i += 1;
                if i < args.len() && !args[i].starts_with('-') {
                    recovery = args[i].parse::<u32>().unwrap_or(2).min(2);
                    i += 1;
                } else {
                    recovery = 2;
                }
            }
            "--password" => {
                i += 1;
                if i < args.len() {
                    password = Some(args[i].clone());
                    i += 1;
                }
            }
            "--password-file" => {
                i += 1;
                if i < args.len() {
                    password_file = Some(args[i].as_str());
                    i += 1;
                }
            }
            _ if !args[i].starts_with('-') => {
                if input.is_none() {
                    input = Some(PathBuf::from(&args[i]));
                } else if output.is_none() {
                    output = Some(PathBuf::from(&args[i]));
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    let password = read_password_from_env_or_file(password, password_file);
    let input = input.ok_or("compress: укажите входной файл или папку")?;
    let mut preset = preset;
    if preset_auto {
        preset = Some(suggested_preset_for_path(&input));
    }
    let output = output.unwrap_or_else(|| default_compress_output_from_input(&input));
    let opts = CompressOptions {
        chunk_size,
        solid,
        password,
        recovery_parity: recovery,
        preset,
        parallel: true,
        progress: None,
    };
    Ok((input, output, opts))
}

fn parse_decompress_args(args: &[String]) -> Result<(PathBuf, PathBuf, Option<String>), Box<dyn std::error::Error + Send + Sync>> {
    let mut archive = None;
    let mut out_dir = None;
    let mut password = None;
    let mut password_file = None;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--password" => {
                i += 1;
                if i < args.len() {
                    password = Some(args[i].clone());
                    i += 1;
                }
            }
            "--password-file" => {
                i += 1;
                if i < args.len() {
                    password_file = Some(args[i].as_str());
                    i += 1;
                }
            }
            _ if !args[i].starts_with('-') => {
                if archive.is_none() {
                    archive = Some(PathBuf::from(&args[i]));
                } else if out_dir.is_none() {
                    out_dir = Some(PathBuf::from(&args[i]));
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    let password = read_password_from_env_or_file(password, password_file);
    let archive = archive.ok_or("decompress: укажите архив .oz")?;
    let out_dir = out_dir.unwrap_or_else(|| PathBuf::from("out"));
    Ok((archive, out_dir, password))
}

fn parse_export_args(args: &[String]) -> Result<(PathBuf, PathBuf, Option<String>), Box<dyn std::error::Error + Send + Sync>> {
    let mut archive = None;
    let mut zip_path = None;
    let mut password = None;
    let mut password_file = None;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--password" => {
                i += 1;
                if i < args.len() {
                    password = Some(args[i].clone());
                    i += 1;
                }
            }
            "--password-file" => {
                i += 1;
                if i < args.len() {
                    password_file = Some(args[i].as_str());
                    i += 1;
                }
            }
            _ if !args[i].starts_with('-') => {
                if archive.is_none() {
                    archive = Some(PathBuf::from(&args[i]));
                } else if zip_path.is_none() {
                    zip_path = Some(PathBuf::from(&args[i]));
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    let password = read_password_from_env_or_file(password, password_file);
    let archive = archive.ok_or("export-zip: укажите архив .oz")?;
    let zip_path = zip_path.unwrap_or_else(|| PathBuf::from("archive.zip"));
    Ok((archive, zip_path, password))
}
