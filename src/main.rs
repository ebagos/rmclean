// 複数のディレクトリのファイルのハッシュ値を比較し同一の場合は最新版を残して削除する

use serde::{Serialize, Deserialize};
use std::env;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::Hasher;
use std::io::{BufReader, Read};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    dirs: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FileData {
    path: String,
    size: u64,
    date: String,
    hash: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct DirData {
    files: Vec<FileData>,
}

// Configの読み込み
fn readConfig(path: &str) -> Config {
    let file = File::open(path).expect("config.json path");
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader).expect("config.json read error");
    result
}

// 引数を解析する
fn parseArgs() -> Vec<String> {
    env::args().collect()
}

// ファイルのハッシュ値を計算する
fn calcHash(path: &str) -> u64 {
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
fn getDirData(path: &str) -> DirData {
    let file = File::open(path);
    if file.is_err() {
        return DirData { files: Vec::new() };
    }
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader);
    if result.is_err() {
        return DirData { files: Vec::new() };
    }
    result
}

// 与えられたファイルが DirDatas に記録されているか確認する
// 一致するファイル名があった場合、サイズと更新日付を確認し、一致したらそのままリターンする
// ファイル名が一致するがサイズや更新日付が一致しない場合、ファイル情報を更新する
// 一致するファイル名がなかった場合、ファイル情報を獲得し、DirDatas.json に追加する
fn checkFile(dirData: &mut DirData, path: &str) {
    for file in dirData.files {
        if file.path == path {
            if file.size == size && file.date == date.to_string() {
                return;
            } else {
                let size = std::fs::metadata(path).unwrap().len();
                let date = std::fs::metadata(path).unwrap().modified().unwrap();
                let hash = calcHash(path);
                return;
            }
        }
    }
    // DirData.filesに追加する
    let size = std::fs::metadata(path).unwrap().len();
    let date = std::fs::metadata(path).unwrap().modified().unwrap();
    let hash = calcHash(path);
    dirData.files.push(FileData {
        path: path.to_string(),
        size: size,
        date: date.to_string(),
        hash: hash,
    });
}

// DirData.filesから存在しないファイルを削除する
fn removeFromDirData(dirData: &mut DirData, path: &str) {
    for file in dirData.files {
        let path = Path::new(path);
        if !path.is_file() {
            dirData.files.remove(file);
        }
    }
}

// DirDataをresults.jsonに書き込む
fn writeDirData(dirData: &DirData, path: &str) {
    let file = File::create(path).expect("file path");
    serde_json::to_writer_pretty(file, &dirData).expect("json write error");
}

// results.jsonからハッシュ値が同じfileのうち更新日付が最新のものを残して他を削除する
fn removeOldFiles(dirDatas: Vec<DirData>) {
    let mut hash_map: HashMap<u64, FileData> = HashMap::new();
    for dirData in dirDatas {
        for file in dirData.files {
            if hash_map.contains_key(file.hash) {
                if hash_map[file.hash].date < file.date {
                    let old = hash_map[file.hash];
                    remove_file(old.path).expect("file remove error");
                    hash_map[file.hash] = file;
                }
            } else {
                hash_map.insert(file.hash, file);
            }
        }
    }

    dirData.files = hash_map.values().collect();

// 単一ディレクトリの処理
fn processDir(dir: &str) -> DirData {
    let mut dirData = getDirData(dir + "/results.json");
    let paths = std::fs::read_dir(dir).unwrap();
    for path in paths {
        let path = path.unwrap().path();
        if path.is_file() {
            if path.to_str().unwrap() == dir + "/results.json" {
                continue;
            }
            checkFile(&mut dirData, path.to_str().unwrap());
        }
    }
    removeFromDirData(&mut dirData, dir);
    writeDirData(&dirData, "results.json");
    dirData
}

fn main() {
    let mut config_path = "";
    let args = parseArgs();
    if args.len() != 2 {
        config_path = "./config.json";
    } else {
        configpath = args[1];
    }
    let config = readConfig(config_path);
    let all = Vec<DirData>::new();
    for dir in config.dirs {
        all.push(processDir(dir));
    }

}