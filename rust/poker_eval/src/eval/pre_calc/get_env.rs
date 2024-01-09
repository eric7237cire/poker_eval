use std::{env, path::PathBuf};

use dotenv::dotenv;

pub fn get_perfect_hash_path() -> PathBuf {
    dotenv().ok();

    let data_dir = env::var("DATA_DIR").unwrap();

    let path = PathBuf::from(data_dir).join("hash.dat");

    path
}

pub fn get_lookup_path() -> PathBuf {
    dotenv().ok();

    let lookup_path_str = env::var("LOOKUP_CODE_PATH").unwrap();

    let path = PathBuf::from(lookup_path_str);

    path
}

pub fn get_boom_path() -> PathBuf {
    dotenv().ok();

    let boom_str = env::var("BOOM_EVAL_CODE_PATH").unwrap();

    let path = PathBuf::from(boom_str);

    path
}

pub fn get_data_file_path(file_name: &str) -> PathBuf {
    dotenv().ok();

    let data_dir = env::var("DATA_DIR").unwrap();

    let path = PathBuf::from(data_dir).join(file_name);

    path
}
