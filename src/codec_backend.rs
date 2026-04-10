// Единственный модуль, где подключаются внешние реализации сжатия.
// Имена не упоминаются в интерфейсе.

pub fn balanced_encode(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    zstd::encode_all(data, 3).map_err(std::io::Error::other)
}

pub fn balanced_decode(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    zstd::decode_all(data).map_err(std::io::Error::other)
}

pub fn fast_encode(data: &[u8]) -> Vec<u8> {
    lz4_flex::block::compress_prepend_size(data)
}

pub fn fast_decode(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    lz4_flex::block::decompress_size_prepended(data)
        .map_err(std::io::Error::other)
}

pub fn max_ratio_encode(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut enc = xz2::write::XzEncoder::new(&mut out, 6);
    std::io::Write::write_all(&mut enc, data)?;
    enc.finish()?;
    Ok(out)
}

pub fn max_ratio_decode(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut dec = xz2::read::XzDecoder::new(data);
    let mut out = Vec::new();
    std::io::copy(&mut dec, &mut out)?;
    Ok(out)
}

/// Максимальное сжатие (XZ level 9, как 7-Zip ultra).
pub fn max_ratio_ultra_encode(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut enc = xz2::write::XzEncoder::new(&mut out, 9);
    std::io::Write::write_all(&mut enc, data)?;
    enc.finish()?;
    Ok(out)
}
