pub mod magic;
pub mod analyzer;
pub mod preprocess;
pub mod huffman;
pub mod codec_backend;
pub mod codec;
pub mod dedup;
pub mod chunked;
pub mod crypto;
pub mod recovery;
pub mod repo;
pub mod pipeline;
pub mod compat;
pub mod smart_preset;

pub use compat::{
    seven_zip_status, seven_zip_install_howto, SevenZipStatus, RAR_FORMAT_NOTE, SEVENZIP_PATH_HINT,
    looks_like_supported_archive_path,
};

pub use pipeline::{
    compress_to_path, compress_to_path_with_options,
    compress_dispatch, compress_advanced_dispatch,
    decompress_to_path, decompress_to_path_with_password, decompress_to_path_with_options,
    decompress_any_to_path, decompress_any_to_path_with_password, decompress_any_to_path_with_options,
    export_to_zip, archive_info, list_archive, list_any_archive, list_any_archive_with_password,
    CompressOptions, Progress, ProgressPhase, Preset, ArchiveInfo,
};
pub use analyzer::{analyze_file, analyze_bytes, AnalysisResult, DataContext};
pub use preprocess::{preprocess, read_preprocess_result, PreprocessResult};
pub use codec::{Codec, compress, decompress, best_compress};
pub use dedup::{BlockStore, BlockRef};
pub use chunked::{chunks, DEFAULT_CHUNK_SIZE};
pub use repo::{repo_init, backup as repo_backup, restore as repo_restore, list_snapshots, repo_push};
pub use smart_preset::{
    effective_compress_preset_hint, effective_oz_preset_from_service_context,
    suggest_compress_preset_hint, suggested_preset_for_path, CompressPresetHint, DIRECTORY_MAX_DEPTH,
    DIRECTORY_SAMPLE_CAP,
};
