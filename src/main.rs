use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use ctrlc;
use regex::Regex;
use serde::Deserialize;

use crossbeam_channel::{bounded, Receiver};

const THREAD_COUNT: usize = 4;

#[derive(Debug, Deserialize)]
struct Settings {
    src_dir: String,
    dst_dir: String,
    log_path: String,
}

fn read_settings() -> std::io::Result<Settings> {
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().ok_or(std::io::Error::from(std::io::ErrorKind::NotFound))?;
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

    // 读取文件并提取编号
    let mut files_with_id: Vec<(u32, PathBuf)> = fs::read_dir(src_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let ext = path.extension()?.to_ascii_uppercase();
            if ext != "NEF" {
                return None;
            }

            let stem = path.file_stem()?.to_str()?;
            let caps = re.captures(stem)?;
            let matched = caps.get(0)?;
            let id = matched.as_str().parse::<u32>().ok()?;

            Some((id, path))
        })
        .collect();

    // 排序
    files_with_id.sort_by_key(|(id, _)| *id);

    // 中断处理
    let interrupted = Arc::new(AtomicBool::new(false));
    let int_flag = interrupted.clone();
    ctrlc::set_handler(move || {
        int_flag.store(true, Ordering::SeqCst);
        println!("\n中断信号收到，准备安全退出");
    }).expect("Error setting Ctrl-C handler");

    // 通道用于多线程任务调度
    let (tx, rx) = bounded::<(u32, PathBuf)>(32);

    // 拷贝线程数
    let num_workers = THREAD_COUNT;
    let mut handles = Vec::new();

    // 用于同步更新最大 ID
    let max_id_arc = Arc::new(parking_lot::Mutex::new(max_id));

    for _ in 0..num_workers {
        let dst_dir = dst_dir.to_path_buf();
        let rx: Receiver<(u32, PathBuf)> = rx.clone();
        let interrupted = interrupted.clone();
        let max_id_arc = Arc::clone(&max_id_arc);

        let handle = thread::spawn(move || {
            while let Ok((id, path)) = rx.recv() {
                if interrupted.load(Ordering::SeqCst) {
                    break;
                }

                let file_name = path.file_name().unwrap();
                let dst_file = dst_dir.join(file_name);
                println!("Copying {:?} to {:?}", path, dst_file);

                if let Ok(_) = fs::copy(&path, &dst_file) {
                    let mut max = max_id_arc.lock();
                    if id > *max {
                        *max = id;
                    }
                }
            }
        });

        handles.push(handle);
    }

    // 主线程分发任务
    for (id, path) in files_with_id {
        if interrupted.load(Ordering::SeqCst) {
            break;
        }

        if id <= last_id {
            continue;
        }

        tx.send((id, path)).unwrap();
    }

    drop(tx); // 关闭发送端

    for handle in handles {
        handle.join().unwrap();
    }

    // 写入最大 ID
    let final_max_id = *max_id_arc.lock();
    if final_max_id > last_id {
        update_log(log_path, final_max_id)?;
        println!("已更新 ID: {}", final_max_id);
    } else {
        println!("没有新文件被拷贝");
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
