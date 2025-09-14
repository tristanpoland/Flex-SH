use super::BuiltinCommand;
use crate::core::parser::ParsedCommand;
use anyhow::Result;
use colored::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::process::Child;

pub struct LsCommand;

#[async_trait::async_trait]
impl BuiltinCommand for LsCommand {
    async fn execute(
        &self,
        command: &ParsedCommand,
        current_dir: &mut PathBuf,
        _background_processes: &mut HashMap<u32, Child>,
    ) -> Result<i32> {
        let mut long_format = false;
        let mut show_hidden = false;
        let mut human_readable = false;
        let mut paths = Vec::new();

        for arg in &command.args {
            match arg.as_str() {
                "-l" => long_format = true,
                "-a" => show_hidden = true,
                "-h" => human_readable = true,
                "-la" | "-al" => {
                    long_format = true;
                    show_hidden = true;
                }
                "-lh" | "-hl" => {
                    long_format = true;
                    human_readable = true;
                }
                "-lah" | "-alh" | "-hal" | "-hla" | "-ahl" | "-lha" => {
                    long_format = true;
                    show_hidden = true;
                    human_readable = true;
                }
                path if !path.starts_with('-') => {
                    paths.push(PathBuf::from(path));
                }
                _ => {
                    eprintln!("ls: unknown option: {}", arg);
                    return Ok(1);
                }
            }
        }

        if paths.is_empty() {
            paths.push(current_dir.clone());
        }

        for (i, path) in paths.iter().enumerate() {
            if i > 0 {
                println!();
            }

            let absolute_path = if path.is_absolute() {
                path.clone()
            } else {
                current_dir.join(path)
            };

            if paths.len() > 1 {
                println!("{}:", absolute_path.display());
            }

            if let Err(e) = list_directory(&absolute_path, long_format, show_hidden, human_readable) {
                eprintln!("ls: {}: {}", absolute_path.display(), e);
                return Ok(1);
            }
        }

        Ok(0)
    }

    fn name(&self) -> &'static str {
        "ls"
    }

    fn description(&self) -> &'static str {
        "List directory contents"
    }

    fn usage(&self) -> &'static str {
        "ls [options] [path...]\n  -l  Use long listing format\n  -a  Show hidden files\n  -h  Human readable sizes"
    }
}

fn list_directory(path: &Path, long_format: bool, show_hidden: bool, human_readable: bool) -> Result<()> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();

        if !show_hidden && file_name.starts_with('.') {
            continue;
        }

        entries.push(entry);
    }

    entries.sort_by(|a, b| {
        let a_name = a.file_name();
        let b_name = b.file_name();
        a_name.cmp(&b_name)
    });

    if long_format {
        for entry in entries {
            print_long_format(&entry, human_readable)?;
        }
    } else {
        let names: Vec<String> = entries
            .iter()
            .map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                if entry.path().is_dir() {
                    name.bright_blue().to_string()
                } else {
                    name
                }
            })
            .collect();

        for (i, name) in names.iter().enumerate() {
            if i > 0 {
                print!("  ");
            }
            print!("{}", name);
        }
        if !names.is_empty() {
            println!();
        }
    }

    Ok(())
}

fn print_long_format(entry: &fs::DirEntry, human_readable: bool) -> Result<()> {
    let metadata = entry.metadata()?;
    let file_name = entry.file_name().to_string_lossy().to_string();

    let file_type = if metadata.is_dir() {
        'd'
    } else if metadata.is_file() {
        '-'
    } else {
        'l'
    };

    #[cfg(unix)]
    let permissions = {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        format!(
            "{}{}{}{}{}{}{}{}{}",
            if mode & 0o400 != 0 { 'r' } else { '-' },
            if mode & 0o200 != 0 { 'w' } else { '-' },
            if mode & 0o100 != 0 { 'x' } else { '-' },
            if mode & 0o040 != 0 { 'r' } else { '-' },
            if mode & 0o020 != 0 { 'w' } else { '-' },
            if mode & 0o010 != 0 { 'x' } else { '-' },
            if mode & 0o004 != 0 { 'r' } else { '-' },
            if mode & 0o002 != 0 { 'w' } else { '-' },
            if mode & 0o001 != 0 { 'x' } else { '-' }
        )
    };

    #[cfg(windows)]
    let permissions = {
        let readonly = metadata.permissions().readonly();
        format!(
            "{}{}{}",
            if !readonly { "rw-" } else { "r--" },
            if !readonly { "rw-" } else { "r--" },
            if !readonly { "rw-" } else { "r--" }
        )
    };

    let size = if human_readable {
        format_human_readable(metadata.len())
    } else {
        format!("{:>8}", metadata.len())
    };

    let modified = metadata.modified()?;
    let datetime: chrono::DateTime<chrono::Local> = modified.into();
    let time_str = datetime.format("%b %d %H:%M").to_string();

    let colored_name = if metadata.is_dir() {
        file_name.bright_blue().to_string()
    } else if is_executable(&entry.path()) {
        file_name.bright_green().to_string()
    } else {
        file_name
    };

    println!(
        "{}{} {:>3} {} {} {}",
        file_type,
        permissions,
        1, // link count (simplified)
        size,
        time_str,
        colored_name
    );

    Ok(())
}

fn format_human_readable(size: u64) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
    let mut size_f = size as f64;
    let mut unit_index = 0;

    while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
        size_f /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{:>5}B", size)
    } else {
        format!("{:>4.1}{}", size_f, UNITS[unit_index])
    }
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = path.metadata() {
            return metadata.permissions().mode() & 0o111 != 0;
        }
    }

    #[cfg(windows)]
    {
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            return matches!(ext.as_str(), "exe" | "bat" | "cmd" | "com" | "ps1");
        }
    }

    false
}