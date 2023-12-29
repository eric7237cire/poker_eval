use std::{path::PathBuf, env};

use dotenv::dotenv;

pub fn get_perfect_hash_path() -> PathBuf {
    dotenv().ok();

    let data_dir = env::var("DATA_DIR").unwrap();

    let path = PathBuf::from(data_dir).join("hash.dat");
    
    path
}

pub fn get_lookup_path() -> PathBuf {
    dotenv().ok();

    let data_dir = env::var("LOOKUP_CODE_PATH").unwrap();

    let path = PathBuf::from(data_dir);
    
    path
}