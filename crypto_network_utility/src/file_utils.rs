use std::fs::File;
use std::io::prelude::*;
use std::path::{Path};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileUtilsError {
    #[error("File name or path '{0}' doesn't exist. Please make sure you are passing an existing file path.")]
    MissingFileThatMustExist(String),
    #[error("File name or path '{0}' already exist. Operation aborted to prevent data loss. Backup or remove this file manually.")]
    PresentFileThatMustNotExist(String),
}

pub fn file_must_exist(path: &Path) -> Result<(), FileUtilsError> {
    if path.exists() {
        Ok(())
    } else {
        Err(FileUtilsError::MissingFileThatMustExist(path.display().to_string()))
    }
}

pub fn file_must_not_exist(path: &Path) -> Result<(), FileUtilsError> {
    if !path.exists() {
        Ok(())
    } else {
        Err(FileUtilsError::PresentFileThatMustNotExist(path.display().to_string()))
    }
}

pub fn write_to_file<P: AsRef<Path>>(file_path: &P, buf: &[u8]) -> std::io::Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(&buf)?;
    Ok(())
}

pub fn read_from_file<P: AsRef<Path>>(file_path: &P) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut contents: Vec<u8> = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

pub fn save_cbor_lz4_file<P: AsRef<Path>, T: serde::Serialize>(file_path: &P, value: &T) -> anyhow::Result<()> {
    let cbor_buf_lz4 = crypto_serializer::cbor_to_vec_lz4(&value)?;
    write_to_file(&file_path, &cbor_buf_lz4)?;
    Ok(())
}

pub fn read_cbor_lz4_file<P: AsRef<Path>, T: serde::de::DeserializeOwned>(file_path: &P) -> anyhow::Result<T> {
    let cbor_buf_lz4 = read_from_file(&file_path)?;
    let ret: T = crypto_serializer::cbor_from_slice_lz4(&cbor_buf_lz4)?;
    Ok(ret)
}

pub fn save_json_file<P: AsRef<Path>, T: serde::Serialize>(file_path: &P, value: &T) -> anyhow::Result<()> {
    let json_bytes = serde_json::to_vec(&value)?;
    write_to_file(&file_path, &json_bytes)?;
    Ok(())
}

pub fn read_json_file<P: AsRef<Path>, T: serde::de::DeserializeOwned>(file_path: &P) -> anyhow::Result<T> {
    let json_str = read_from_file(&file_path)?;
    let ret: T = serde_json::from_slice(&json_str)?;
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
}
