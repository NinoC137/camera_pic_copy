use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use ctrlc;
use serde::Deserialize;
use regex::Regex;

#[derive(Debug, Deserialize)]
struct Settings {
    src_dir: String,
    dst_dir: String,
    log_path: String,
}

fn read_settings() -> std::io::Result<Settings> {
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent()
        .ok_or(std::io::Error::from(std::io::ErrorKind::NotFound))?;
    let settings_path = exe_dir.join("setting.txt");

    let file = File::open(settings_path)?;
    let reader = BufReader::new(file);
    let settings = serde_json::from_reader(reader)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(settings)
}

fn main() -> std::io::Result<()> {
    let settings = read_settings()?;

    let src_dir = Path::new(&settings.src_dir);
    let dst_dir = Path::new(&settings.dst_dir);
    let log_path = Path::new(&settings.log_path);

    if !dst_dir.exists() {
        fs::create_dir(dst_dir)?;
    }

    let last_id = read_last_id(log_path).unwrap_or(0);

    println!("last id: {}", last_id);

    let mut max_id = last_id;

    let re = Regex::new(r"\d+").unwrap();

    let mut files_with_id: Vec<(u32, PathBuf)> = Vec::new();

    let interrupted = Arc::new(AtomicBool::new(false));
    let int_flag = interrupted.clone();

    ctrlc::set_handler(move || {
        int_flag.store(true, Ordering::SeqCst);
        println!("\n获取到中止指令，正在存储已转存的ID");
    }).expect("Error setting Ctrl-C handler");

    for entry in fs::read_dir(src_dir)? {
        let dir_entry = entry?;
        //获取到的路径是乱序的，并非文件管理器中看到的顺序
        let path = dir_entry.path();

        if let Some(extension) = path.extension() {
            if extension.to_ascii_uppercase() != "NEF" {
                continue;
            }
        } else {
            continue;
        }

        if let Some(file_stem) = path.file_stem() {

            let caps = re.captures(file_stem.to_str()
                .unwrap())
                .unwrap();

            if let Some(matched) = caps.get(0) {
                if let Ok(id) = matched.as_str().parse::<u32>() {
                    files_with_id.push((id, path));
                }
            }
        }
    }

    files_with_id.sort_by_key(|(id, _)| *id);

    for (id, path) in files_with_id {
        if interrupted.load(Ordering::SeqCst) {
            println!("Saving last copied ID :{}", max_id);
            break;
        }

        if id <= last_id {
            continue;
        }

        let file_name = path.file_name().unwrap();
        let dst_file = dst_dir.join(file_name);

        println!("Copying {:?} to {:?}", path, dst_file);
        fs::copy(&path, &dst_file)?;

        if id > max_id {
            max_id = id;
        }
    }

    if max_id > last_id {
        update_log(log_path, max_id)?;
        println!("update id : {}", max_id);
    } else {
        println!("no change");
    }

    Ok(())
}

fn read_last_id(log_path: &Path) -> Option<u32> {
    if !log_path.exists() {
        return None;
    }

    let file = File::open(log_path).ok()?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;

    line.trim().parse::<u32>().ok()
}

fn update_log(log_path: &Path, id: u32) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)?;

    writeln!(&mut file, "{}", id)?;
    Ok(())
}