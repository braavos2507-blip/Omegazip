//! Динамический выбор кодека по контексту и тесту в памяти.

use crate::analyzer::DataContext;
use crate::codec_backend;
use crate::huffman;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    Dense,
    Balanced,
    Fast,
    MaxRatio,
    Store,
}

impl Codec {
    pub fn from_context(ctx: DataContext) -> Self {
        match ctx {
            DataContext::Text => Codec::Dense,
            DataContext::Binary => Codec::Balanced,
            DataContext::Realtime => Codec::Fast,
            DataContext::Media => Codec::Store,
            DataContext::Archive => Codec::MaxRatio,
        }
    }
}

pub fn compress(codec: Codec, data: &[u8]) -> std::io::Result<Vec<u8>> {
    match codec {
        Codec::Dense => huffman::encode(data),
        Codec::Balanced => codec_backend::balanced_encode(data),
        Codec::Fast => Ok(codec_backend::fast_encode(data)),
        Codec::MaxRatio => codec_backend::max_ratio_encode(data),
        Codec::Store => Ok(data.to_vec()),
    }
}

pub fn decompress(codec: Codec, data: &[u8]) -> std::io::Result<Vec<u8>> {
    match codec {
        Codec::Dense => huffman::decode(data),
        Codec::Balanced => codec_backend::balanced_decode(data),
        Codec::Fast => codec_backend::fast_decode(data),
        Codec::MaxRatio => codec_backend::max_ratio_decode(data),
        Codec::Store => Ok(data.to_vec()),
    }
}

pub fn best_compress(data: &[u8], context: DataContext) -> std::io::Result<(Codec, Vec<u8>)> {
    if context == DataContext::Media {
        return Ok((Codec::Store, data.to_vec()));
    }
    let preferred = Codec::from_context(context);
    let candidates = [preferred, Codec::Balanced, Codec::Fast];

    let mut best: Option<(Codec, Vec<u8>)> = None;
    for c in candidates {
        if let Ok(compressed) = compress(c, data) {
            if compressed.len() < data.len() {
                let is_better = best
                    .as_ref()
                    .is_none_or(|(_, b)| compressed.len() < b.len());
                if is_better {
                    best = Some((c, compressed));
                }
            }
        }
    }

    if let Some((c, out)) = best {
        Ok((c, out))
    } else {
        Ok((Codec::Store, data.to_vec()))
    }
}

pub fn codec_id(c: &Codec) -> u8 {
    match c {
        Codec::Dense => 0,
        Codec::Balanced => 1,
        Codec::Fast => 2,
        Codec::MaxRatio => 3,
        Codec::Store => 4,
    }
}

pub fn codec_from_id(id: u8) -> Codec {
    match id {
        0 => Codec::Dense,
        1 => Codec::Balanced,
        2 => Codec::Fast,
        3 => Codec::MaxRatio,
        _ => Codec::Store,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::DataContext;

    #[test]
    fn test_best_compress_text() {
        let text = "Hello world! This is a test. ".repeat(500);
        let data = text.as_bytes();
        let (codec, compressed) = best_compress(data, DataContext::Text).unwrap();
        println!("Text {} bytes -> {:?} {} bytes ({}%)", data.len(), codec, compressed.len(), compressed.len() * 100 / data.len());
        assert!(compressed.len() < data.len(), "Text should compress! {} >= {}", compressed.len(), data.len());
        assert_ne!(codec, Codec::Store, "Should not fall back to Store for text");
    }

    #[test]
    fn test_huffman_roundtrip() {
        let text = "The quick brown fox jumps over the lazy dog. ".repeat(100);
        let data = text.as_bytes();
        let encoded = crate::huffman::encode(data).unwrap();
        println!("Huffman: {} -> {} bytes", data.len(), encoded.len());
        let decoded = crate::huffman::decode(&encoded).unwrap();
        assert_eq!(data.len(), decoded.len(), "Huffman round-trip length mismatch: {} != {}", data.len(), decoded.len());
        assert_eq!(data, decoded.as_slice(), "Huffman round-trip data mismatch");
    }

    #[test]
    fn test_codec_roundtrips() {
        let data = b"Hello World! Testing compression roundtrip. ".repeat(200);
        for codec in [Codec::Dense, Codec::Balanced, Codec::Fast, Codec::MaxRatio] {
            let compressed = compress(codec, &data).unwrap();
            let decompressed = decompress(codec, &compressed).unwrap();
            assert_eq!(data.as_slice(), decompressed.as_slice(), "{:?} round-trip failed", codec);
            println!("{:?}: {} -> {} bytes", codec, data.len(), compressed.len());
        }
    }

    #[test]
    fn test_huffman_single_symbol() {
        let data = vec![0xAA; 1000];
        let encoded = crate::huffman::encode(&data).unwrap();
        println!("Single-symbol: {} -> {} bytes", data.len(), encoded.len());
        assert!(encoded.len() < data.len(), "Single-symbol should compress");
        let decoded = crate::huffman::decode(&encoded).unwrap();
        assert_eq!(data, decoded, "Single-symbol round-trip failed");
    }

    #[test]
    fn test_pipeline_roundtrip() {
        let dir = std::env::temp_dir().canonicalize().unwrap().join("omegazip_test_rt");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let input = dir.join("input.txt");
        let archive = dir.join("output.oz");
        let out_dir = dir.join("extracted");
        let text = "Pipeline round-trip test data. ".repeat(500);
        std::fs::write(&input, &text).unwrap();
        let n = crate::pipeline::compress_to_path(&input, &archive).unwrap();
        assert_eq!(n, 1);
        let archive_size = std::fs::metadata(&archive).unwrap().len();
        let input_size = text.len() as u64;
        println!("Pipeline: {} -> {} bytes ({}%)", input_size, archive_size, archive_size * 100 / input_size);
        assert!(archive_size < input_size, "Archive should be smaller: {} >= {}", archive_size, input_size);
        let m = crate::pipeline::decompress_to_path(&archive, &out_dir).unwrap();
        assert_eq!(m, 1);
        // Find the extracted file
        let extracted: Vec<_> = walkdir::WalkDir::new(&out_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect();
        assert_eq!(extracted.len(), 1);
        let content = std::fs::read_to_string(extracted[0].path()).unwrap();
        assert_eq!(content, text, "Decompressed content mismatch");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
