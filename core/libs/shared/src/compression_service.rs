use std::io::{Read, Write};

use crate::{SharedError, SharedResult};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;

pub fn compress(content: &[u8]) -> SharedResult<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(content)
        .map_err(|_| SharedError::Unexpected("unexpected compression error"))?;
    Ok(encoder
        .finish()
        .map_err(|_| SharedError::Unexpected("unexpected compression error"))?)
}

pub fn decompress(content: &[u8]) -> SharedResult<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(content);
    let mut result = Vec::<u8>::new();
    decoder
        .read_to_end(&mut result)
        .map_err(|_| SharedError::Unexpected("unexpected decompression error"))?;
    Ok(result)
}

#[test]
fn compress_decompress() {
    assert_eq!(decompress(&compress(b"hello").unwrap(),).unwrap(), b"hello");
}
