//! # 圧縮モジュール
//!
//! バックアップファイルの圧縮・展開機能を提供します。
//! zstd と gzip アルゴリズムをサポートします。

pub mod engines;

// 主要な型と関数を再エクスポート
pub use engines::{CompressionEngine, CompressionType, CompressionConfig, CompressedData};