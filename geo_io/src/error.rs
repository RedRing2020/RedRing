//! エラー型の定義

use std::fmt;

/// ファイルI/O関連のエラー
#[derive(Debug)]
pub enum IoError {
    /// ファイルシステムエラー
    FileSystem(std::io::Error),
    /// フォーマット固有のエラー
    Format(Box<dyn std::error::Error + Send + Sync>),
    /// データ変換エラー
    Conversion(String),
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoError::FileSystem(err) => write!(f, "File system error: {}", err),
            IoError::Format(err) => write!(f, "Format error: {}", err),
            IoError::Conversion(msg) => write!(f, "Conversion error: {}", msg),
        }
    }
}

impl std::error::Error for IoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            IoError::FileSystem(err) => Some(err),
            IoError::Format(err) => Some(err.as_ref()),
            IoError::Conversion(_) => None,
        }
    }
}

impl From<std::io::Error> for IoError {
    fn from(err: std::io::Error) -> Self {
        IoError::FileSystem(err)
    }
}

/// STL固有のエラー
#[derive(Debug)]
pub enum StlError {
    /// ファイルI/Oエラー
    Io(IoError),
    /// 無効なSTLヘッダー
    InvalidHeader(String),
    /// 無効なファセット数
    InvalidFacetCount(String),
    /// 無効な三角形データ
    InvalidTriangle(String),
    /// フォーマット判定エラー
    FormatDetection(String),
    /// 精度変換エラー
    PrecisionConversion(String),
}

impl fmt::Display for StlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StlError::Io(err) => write!(f, "STL I/O error: {}", err),
            StlError::InvalidHeader(msg) => write!(f, "Invalid STL header: {}", msg),
            StlError::InvalidFacetCount(msg) => write!(f, "Invalid facet count: {}", msg),
            StlError::InvalidTriangle(msg) => write!(f, "Invalid triangle: {}", msg),
            StlError::FormatDetection(msg) => write!(f, "Format detection failed: {}", msg),
            StlError::PrecisionConversion(msg) => write!(f, "Precision conversion failed: {}", msg),
        }
    }
}

impl std::error::Error for StlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StlError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<IoError> for StlError {
    fn from(err: IoError) -> Self {
        StlError::Io(err)
    }
}

impl From<std::io::Error> for StlError {
    fn from(err: std::io::Error) -> Self {
        StlError::Io(IoError::FileSystem(err))
    }
}
