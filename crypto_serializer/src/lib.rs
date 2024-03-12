extern crate lz4_compress as lz4;
// use std::fs::File;

use lzzzz::lz4f::{WriteCompressor, ReadDecompressor, PreferencesBuilder};
use std::{fs::File, io::prelude::*};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;

// TODO important 
// cbor legacy lz network loading time 4.580944619s # legacy lz4
// network.cbor.lz4 loading time 3.708280823s # lzzz
// network.bincode.lz4 loading time 497.77615ms => 86.57% FASTER THAN ciborium

// seems that the limitations of bincode have been fixed regarding 128 bit integers. 
// BUT I AM NOT SURE IF bincode IS CROSS SYSTEM COMPATIBLE. OS cross compatible or ISA compatible. This is not clear

// FILE SIZES:
// -rw-rw-r--   1 stanko stanko   13M maj 24 22:37  network.bincode.lz4 # lzzz+bincode gives BEST compression rates
// -rw-rw-r--   1 stanko stanko   21M maj 24 22:41  network.cbor.lz4 # lzzz
// -rw-rw-r-- 1 stanko stanko 40M apr 19 00:19 lexicon_network_paths.net # legacy lz4



#[derive(Error, Debug)]
pub enum CryptoSerializerError {
    #[error("ciborium reader error '{0}'")]
    CiboriumReaderError(String),
    #[error("ciborium writer error '{0}'")]
    CiboriumWriterError(String),
    #[error("lz4 decompress error '{0}'")]
    Lz4DecompressError(String),
    #[error("lzzzz compress error '{0}'")]
    LzzzzCompressionError(String),
    #[error("lzzzz decompress error '{0}'")]
    LzzzzDecompressionError(String),
    #[error("bincode serialize error '{0}'")]
    BincodeSerializeError(String),
    #[error("bincode deserialize error '{0}'")]
    BincodeDeserializeError(String),
}

// TODO implement mutliple run compression since it can make the files smaller or just use a 2 round
pub fn compress_lz4(input: &[u8]) -> Vec<u8> {
    lz4::compress(input)
}

pub fn decompress_lz4<'a>(input: &'a [u8]) -> Result<Vec<u8>, CryptoSerializerError> {
    lz4::decompress(input).map_err(|err| CryptoSerializerError::Lz4DecompressError(format!("{}", err)))
}

pub fn cbor_to_vec<T>(value: &T) -> Result<Vec<u8>, CryptoSerializerError>
where
    T: Serialize,
{
    let mut buf: Vec<u8> = vec![];
    let r = ciborium::ser::into_writer(&value, &mut buf);
    r.map_err(|err| CryptoSerializerError::CiboriumWriterError(format!("{}", err)))?;
    Ok(buf)
}

// serde_cbor::from_slice(&contents)
pub fn cbor_from_slice<'a, T>(slice: &'a [u8]) -> Result<T, CryptoSerializerError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let r = ciborium::de::from_reader(slice);
    r.map_err(|err| CryptoSerializerError::CiboriumReaderError(format!("{}", err)))
}

pub fn cbor_to_vec_lz4<T>(value: &T) -> Result<Vec<u8>, CryptoSerializerError>
where
    T: Serialize,
{
    let ok = cbor_to_vec(value)?;
    Ok(compress_lz4(&ok))
}

pub fn cbor_from_slice_lz4<'a, T>(slice: &[u8]) -> Result<T, CryptoSerializerError>
where
    T: DeserializeOwned,
{
    let dec = decompress_lz4(&slice)?;
    cbor_from_slice(&dec)
}

fn lzzzz_cbor_from_file_err(err: String) -> CryptoSerializerError {
    CryptoSerializerError::LzzzzDecompressionError(err)
}

pub fn lzzzz_cbor_from_file<T>(path: &str) -> Result<T, CryptoSerializerError> where
T: DeserializeOwned, {
    let mut f = File::open(path)
        .map_err(|err| lzzzz_cbor_from_file_err(format!("File::open {err}")))?;
    let mut r: ReadDecompressor<&mut File> = ReadDecompressor::new(&mut f)
        .map_err(|err| lzzzz_cbor_from_file_err(format!("ReadDecompressor::new {err}")))?;
    let mut decomp = Vec::new();
    r.read_to_end(&mut decomp)
        .map_err(|err| lzzzz_cbor_from_file_err(format!("r.read_to_end {err}")))?;
    cbor_from_slice(&decomp)
}

fn lzzzz_cbor_to_file_err(err_str: String) -> CryptoSerializerError {
    CryptoSerializerError::LzzzzCompressionError(err_str)
}

pub fn lzzzz_cbor_to_file<T>(comp_level: i32, value: &T, path: &str) -> Result<(), CryptoSerializerError> where
T: Serialize, {
    let cbor_serialized = cbor_to_vec(value)?;
    let mut f = File::create(path)
        .map_err(|err| lzzzz_cbor_to_file_err(format!("File::create {err}")))?;
    let prefs = PreferencesBuilder::new().compression_level(comp_level).build();
    let mut w = WriteCompressor::new(&mut f, prefs)
        .map_err(|err| lzzzz_cbor_to_file_err(format!("WriteCompressor::new {err}")))?;
    w.write_all(&cbor_serialized)
        .map_err(|err| lzzzz_cbor_to_file_err(format!("w.write_all {err}")))?;
    
    Ok(())
}

fn lzzzz_bincode_from_file_err(err: String) -> CryptoSerializerError {
    CryptoSerializerError::LzzzzDecompressionError(err)
}

pub fn lzzzz_bincode_from_file<T>(path: &str) -> Result<T, CryptoSerializerError> where
T: DeserializeOwned, {
    let mut f = File::open(path)
        .map_err(|err| lzzzz_bincode_from_file_err(format!("File::open {err}")))?;
    let mut r: ReadDecompressor<&mut File> = ReadDecompressor::new(&mut f)
        .map_err(|err| lzzzz_bincode_from_file_err(format!("ReadDecompressor::new {err}")))?;
    let mut decomp = Vec::new();
    r.read_to_end(&mut decomp)
        .map_err(|err| lzzzz_bincode_from_file_err(format!("r.read_to_end {err}")))?;
    bincode::deserialize(&decomp).map_err(|err| CryptoSerializerError::BincodeDeserializeError(err.to_string()))
}

fn lzzzz_bincode_to_file_err(err_str: String) -> CryptoSerializerError {
    CryptoSerializerError::LzzzzCompressionError(err_str)
}

pub fn lzzzz_bincode_to_file<T>(comp_level: i32, value: &T, path: &str) -> Result<(), CryptoSerializerError> where
T: Serialize, {
    let bincode_serialized = bincode::serialize(&value)
        .map_err(|err| CryptoSerializerError::BincodeSerializeError(err.to_string()))?;
    let mut f = File::create(path)
        .map_err(|err| lzzzz_bincode_to_file_err(format!("File::create {err}")))?;
    let prefs = PreferencesBuilder::new().compression_level(comp_level).build();
    let mut w = WriteCompressor::new(&mut f, prefs)
        .map_err(|err| lzzzz_bincode_to_file_err(format!("WriteCompressor::new {err}")))?;
    w.write_all(&bincode_serialized)
        .map_err(|err| lzzzz_bincode_to_file_err(format!("w.write_all {err}")))?;
    
    Ok(())
}


// // TODO maybe it could be possible to have a writer and reader functions/methods
// pub fn cbor_to_vec_lz4_writer<T>(value: &T) -> Result<Vec<u8>, CryptoSerializerError>
// where
//     T: Serialize,
// {
//     let ok = cbor_to_vec(value)?;
//     Ok(compress_lz4(&ok))
// }

// pub fn cbor_from_slice_lz4_reader<'a, T>(slice: &[u8]) -> Result<T, CryptoSerializerError>
// where
//     T: DeserializeOwned,
// {
//     let dec = decompress_lz4(&slice)?;
//     cbor_from_slice(&dec)
// }
