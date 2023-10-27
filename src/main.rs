// 複数のディレクトリのファイルのハッシュ値を比較し同一の場合は最新版を残して削除する

use serde::{Serialize, Deserialize};
use std::env;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hasher;
use std::io::{BufReader, Read};
use std::time::UNIX_EPOCH;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    dirs: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FileData {
    path: String,
    size: u64,
    date: u64,
    hash: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct DirData {
    files: Vec<FileData>,
}

// Configの読み込み
fn read_config(path: &str) -> Config {
    let file = File::open(path).expect("config.json path");
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader).expect("config.json read error");
    result
}

// 引数を解析する
fn parse_args() -> Vec<String> {
    env::args().collect()
}

// ファイルのハッシュ値を計算する
fn calc_hash(path: &str) -> u64 {
    let file = File::open(path).expect("file path");
    let mut reader = BufReader::new(file);
    let mut hasher = DefaultHasher::new();
    let mut buffer = [0; 1024];

    while let Ok(n) = reader.read(&mut buffer) {
        hasher.write(&buffer);

        if n == 0 {
            break;
        }
    }

    hasher.finish()
}

// result.jsonを取得する
fn get_dirdata(path: &str) -> DirData {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return DirData { files: Vec::new() },
    };
    let reader = BufReader::new(file);
    let result = match serde_json::from_reader(reader) {
        Ok(result) => result,
        Err(_) => return DirData { files: Vec::new() },
    };
    result
}

// 与えられたファイルが dirdatas に記録されているか確認する
// 一致するファイル名があった場合、サイズと更新日付を確認し、一致したらそのままリターンする
// ファイル名が一致するがサイズや更新日付が一致しない場合、ファイル情報を更新する
// 一致するファイル名がなかった場合、ファイル情報を獲得し、dirdatas.json に追加する
fn check_file(dirdata: &mut DirData, path: &str) {
    if let Some(existing_file) = dirdata.files.iter_mut().find(|f| f.path == path) {
        let meta = std::fs::metadata(path).unwrap();
        let size = meta.len();
        let date = meta.modified().unwrap();
        let date = date.duration_since(UNIX_EPOCH).unwrap().as_secs();
    // Compare size and date, and update if different
        if existing_file.size != size || existing_file.date != date {
            existing_file.size = size;
            existing_file.date = date;
            existing_file.hash = calc_hash(path);
        } else {
            return;
        }
    } else {
        // If the file doesn't exist in the Vec, add it
        let size = std::fs::metadata(path).unwrap().len();
        let date = std::fs::metadata(path).unwrap().modified().unwrap();
        let date = date.duration_since(UNIX_EPOCH).unwrap().as_secs();
        let hash = calc_hash(path);
        dirdata.files.push(FileData {
            path: path.to_string(),
            size: size,
            date: date,
            hash: hash,
        });
    }
}

// DirData.filesから存在しないファイルを削除する
fn remove_from_dirdata(dirdata: &mut DirData) {
    dirdata.files.retain(|file| {
        if let Ok(_) = std::fs::metadata(&file.path) {
            true // Keep the file if it exists
        } else {
            false // Remove the file if it doesn't exist
        }
    });
}

// dirdataをresults.jsonに書き込む
fn write_dirdata(dirdata: &DirData, path: &str) {
    let file = File::create(path).expect("file path");
    serde_json::to_writer_pretty(file, &dirdata).expect("json write error");
}

// results.jsonからハッシュ値が同じfileのうち更新日付が最新のものを残して他を削除する
fn remove_old_files(dirs: Vec<DirData>) {
    let mut hash_map: HashMap<u64, FileData> = HashMap::new();

    for dir in dirs {
        for file in dir.files {
            if let Some(existing_file) = hash_map.get_mut(&file.hash) {
                if existing_file.date < file.date {
                    if let Err(err) = std::fs::remove_file(&existing_file.path) {
                        eprintln!("File remove error: {:?}", err);
                    }
                    *existing_file = file;
                } else {
                    if let Err(err) = std::fs::remove_file(&file.path) {
                        eprintln!("File remove error: {:?}", err);
                    }
                }
            } else {
                // 登録されていないので追加する
                hash_map.insert(file.hash, file);
            }
        }
    }
}

// 単一ディレクトリの処理
fn process_dir(dir: &str) -> DirData {
    let mut dirdata = get_dirdata(&format!("{}/results.json", dir));
    let paths: Vec<_> = std::fs::read_dir(dir).unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_file() {
                Some(entry)
            } else {
                None
            }
        })
        .collect();
            
    let file_count = paths.len();
    for (counter, entry) in paths.iter().enumerate() {
        let path = entry.path();
        let path_name = path.file_name().unwrap().to_str().unwrap();

        println!("Processing {} of {} in {}", counter + 1, file_count, dir);

        if path_name != "results.json" {
            check_file(&mut dirdata, path.to_str().unwrap());
        }
    }

    remove_from_dirdata(&mut dirdata);
    write_dirdata(&dirdata, &format!("{}/results.json", dir));
    dirdata
}
/*
fn process_dir(dir: &str) -> DirData {
    let mut dirdata = get_dirdata(&format!("{}/results.json", dir));
    let paths: Vec<_> = std::fs::read_dir(dir).unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .collect();
        
    let file_count = paths.len();
    for (counter, entry) in paths.iter().enumerate() {
        let path = entry.path();
        let path_name = path.file_name().unwrap().to_str().unwrap();

        println!("Processing {} of {} in {}", counter + 1, file_count, dir);

        if path_name != "results.json" {
            check_file(&mut dirdata, path.to_str().unwrap());
        }
    }

    remove_from_dirdata(&mut dirdata);
    write_dirdata(&dirdata, &format!("{}/results.json", dir));
    dirdata
}
*/
fn main() {
    let config_path: &str;
    let args = parse_args();
    if args.len() != 2 {
        config_path = "./config.json";
    } else {
        config_path = &args[1];
    }
    println!("start pre-process");
    let config = read_config(config_path);
    let mut all = Vec::<DirData>::new();
    for dir in config.dirs {
        let dd = process_dir(dir.as_str());
        all.push(dd);
    }
    remove_old_files(all);

    println!("start post-process");
    let config = read_config(config_path);
    for dir in config.dirs {
        process_dir(dir.as_str());
    }
}
