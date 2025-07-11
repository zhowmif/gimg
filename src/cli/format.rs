use crate::ppm::{is_ppm_by_extension, is_ppm_by_signature};

pub enum FileFormat {
    Ppm,
}

impl FileFormat {
    pub fn is_format_by_signature(&self, file: &[u8]) -> bool {
        match self {
            FileFormat::Ppm => is_ppm_by_signature(file),
        }
    }
    pub fn is_format_by_extension(&self, filename: &str) -> bool {
        match self {
            FileFormat::Ppm => is_ppm_by_extension(filename),
        }
    }
}

pub const SUPPORTED_FORMATS: [FileFormat; 1] = [FileFormat::Ppm];
