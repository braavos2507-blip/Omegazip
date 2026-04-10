//! Property-based регрессия кодеков и лёгкий smoke анализатора (без паник на случайных байтах).

use omegazip::{analyze_bytes, compress, decompress, Codec};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn prop_store_roundtrip(data in prop::collection::vec(any::<u8>(), 0..16_384)) {
        let compressed = compress(Codec::Store, &data).unwrap();
        let out = decompress(Codec::Store, &compressed).unwrap();
        prop_assert_eq!(data, out);
    }

    #[test]
    fn prop_balanced_roundtrip(data in prop::collection::vec(any::<u8>(), 0..8192)) {
        let compressed = compress(Codec::Balanced, &data).unwrap();
        let out = decompress(Codec::Balanced, &compressed).unwrap();
        prop_assert_eq!(data, out);
    }

    #[test]
    fn prop_fast_roundtrip(data in prop::collection::vec(any::<u8>(), 0..8192)) {
        let compressed = compress(Codec::Fast, &data).unwrap();
        let out = decompress(Codec::Fast, &compressed).unwrap();
        prop_assert_eq!(data, out);
    }

    #[test]
    fn prop_max_ratio_roundtrip(data in prop::collection::vec(any::<u8>(), 0..4096)) {
        let compressed = compress(Codec::MaxRatio, &data).unwrap();
        let out = decompress(Codec::MaxRatio, &compressed).unwrap();
        prop_assert_eq!(data, out);
    }

    #[test]
    fn prop_analyze_bytes_no_panic(data in prop::collection::vec(any::<u8>(), 0..8192)) {
        let _ = analyze_bytes(&data, None);
    }
}
