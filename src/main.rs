use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::sync::{mpsc, Arc, Mutex};

use regex::Regex;

const THREAD_COUNT: usize = 4;

fn main() -> std::io::Result<()> {
    let src_dir = Path::new("/Users/nino/Documents/测试/");
    let dst_dir = Path::new("/tmp/test_pic_copy/target_dir/");
    let log_path = Path::new("/tmp/test_pic_copy/log.txt");

    if !dst_dir.exists() {
        fs::create_dir(dst_dir)?;
    }

    let last_id = read_last_id(log_path).unwrap_or(0);

    println!("last id: {}", last_id);

    let mut max_id = last_id;

    let re = Regex::new(r"\d+").unwrap();

    for entry in fs::read_dir(src_dir)? {
        let dir_entry = entry?;
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
                    if id <= last_id {
                        continue;   //已经拷贝过了
                    }

                    let file_name = path.file_name().unwrap();
                    let dst_file = dst_dir.join(file_name);

                    println!("Copying {:?} to {:?}", path, dst_file);
                    fs::copy(&path, &dst_file)?;

                    if id > max_id {
                        max_id = id;
                    }
                }
            }
        }

        if max_id > last_id {
            update_log(log_path, max_id)?;
            println!("update id : {}", max_id);
        } else {
            println!("no change");
        }
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