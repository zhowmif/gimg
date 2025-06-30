use crate::colors::RGBA;

use super::{color_type::InvalidBitDepthError, ColorType, InterlaceMethod};

#[derive(Clone, Copy, Debug)]
pub enum CompressionLevel {
    None,
    Best,
    Fast,
}
impl CompressionLevel {
    pub fn to_zlib_u8(&self) -> u8 {
        match self {
            CompressionLevel::None => 0,
            CompressionLevel::Best => 3,
            CompressionLevel::Fast => 1,
        }
    }
}

impl Default for CompressionLevel {
    fn default() -> Self {
        Self::Best
    }
}

pub struct PngConfig {
    pub compression_level: CompressionLevel,
    pub color_type: ColorType,
    pub bit_depth: u8,
    pub interlace_method: InterlaceMethod,
}

pub struct PartialPngConfig {
    compression_level: Option<CompressionLevel>,
    color_type: Option<ColorType>,
    bit_depth: Option<u8>,
    interlace_method: Option<InterlaceMethod>,
}

impl PartialPngConfig {
    pub fn new() -> Self {
        Self {
            compression_level: None,
            color_type: None,
            bit_depth: None,
            interlace_method: None,
        }
    }

    pub fn interlace_method(mut self, interlace_method: InterlaceMethod) -> Self {
        self.interlace_method = Some(interlace_method);
        self
    }

    pub fn compression_level(mut self, compression_level: CompressionLevel) -> Self {
        self.compression_level = Some(compression_level);
        self
    }

    pub fn bit_depth(mut self, bit_depth: u8) -> Self {
        self.bit_depth = Some(bit_depth);
        self
    }

    pub fn color_type(mut self, color_type: ColorType) -> Self {
        self.color_type = Some(color_type);
        self
    }
}

impl PngConfig {
    pub fn new(
        compression_level: CompressionLevel,
        color_type: ColorType,
        bit_depth: u8,
        interlace_method: InterlaceMethod,
    ) -> Self {
        Self {
            compression_level,
            color_type,
            bit_depth,
            interlace_method,
        }
    }

    pub fn create_from_partial(partial_config: PartialPngConfig, unique_colors: &[RGBA]) -> Self {
        let number_of_unique_colors = unique_colors.len();
        let should_colors_be_indexed = number_of_unique_colors <= u8::MAX.into();
        let has_alpha = unique_colors.iter().any(|color| !color.is_opaque());
        let is_grayscale = unique_colors.iter().all(|color| color.is_greyscale());

        let color_type: ColorType = partial_config.color_type.unwrap_or({
            if is_grayscale {
                if has_alpha {
                    ColorType::GreyscaleAlpha
                } else {
                    ColorType::Greyscale
                }
            } else if should_colors_be_indexed {
                ColorType::IndexedColor
            } else if has_alpha {
                ColorType::TrueColorAlpha
            } else {
                ColorType::Truecolor
            }
        });
        let bit_depth = partial_config.bit_depth.unwrap_or(match color_type {
            ColorType::IndexedColor | ColorType::Greyscale => {
                if number_of_unique_colors <= 2 {
                    1
                } else if number_of_unique_colors <= 4 {
                    2
                } else {
                    8
                }
            }
            _ => 8,
        });

        if let Err(InvalidBitDepthError(color_type, bit_depth)) =
            color_type.check_bit_depth_validty(bit_depth)
        {
            panic!(
                "Invalid bit depth {} for color type {}",
                bit_depth,
                Into::<u8>::into(&color_type),
            );
        }

        Self {
            compression_level: partial_config.compression_level.unwrap_or_default(),
            interlace_method: partial_config.interlace_method.unwrap_or_default(),
            color_type,
            bit_depth,
        }
    }
}
