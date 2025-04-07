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

    let re = Regex::new(r"\d+").unwrap();
    let mut max_id = Arc::new(Mutex::new(last_id));
    let (tx, rx) = mpsc::channel::<PathBuf>();
    let rx = Arc::new(Mutex::new(rx));

    let mut handles = Vec::new();
    for _ in 0..THREAD_COUNT {
        let rx = Arc::clone(&rx);
        let dst_dir = dst_dir.to_path_buf();
        let re = re.clone();
        let max_id = Arc::clone(&max_id);

        let handle = thread::spawn(move || {
            loop {
                let path = {
                    // 获取任务
                    let lock = rx.lock().unwrap();
                    lock.recv()
                };

                let path = match path {
                    Ok(p) => p,
                    Err(_) => break, // 通道关闭时退出线程
                };

                if let Some(ext) = path.extension() {
                    if ext.to_ascii_uppercase() != "NEF" {
                        continue;
                    }
                } else {
                    continue;
                }

                let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if let Some(caps) = re.captures(file_stem) {
                    if let Some(matched) = caps.get(0) {
                        if let Ok(id) = matched.as_str().parse::<u32>() {
                            let current_max = {
                                let lock = max_id.lock().unwrap();
                                *lock
                            };

                            if id <= current_max {
                                continue;
                            }

                            let file_name = path.file_name().unwrap();
                            let dst_file = dst_dir.join(file_name);

                            match fs::copy(&path, &dst_file) {
                                Ok(_) => {
                                    println!("Copied {:?} to {:?}", path, dst_file);
                                    let mut lock = max_id.lock().unwrap();
                                    if id > *lock {
                                        *lock = id;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to copy {:?}: {}", path, e);
                                }
                            }
                        }
                    }
                }
            }
        });

        handles.push(handle);
    }

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        tx.send(entry.path()).unwrap();
    }

    drop(tx);

    for handle in handles {
        handle.join().unwrap();
    }

    let final_max_id = *max_id.lock().unwrap();
    if final_max_id > last_id {
        update_log(log_path, final_max_id)?;
        println!("update last id: {}", final_max_id);
    } else {
        println!("without update.");
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