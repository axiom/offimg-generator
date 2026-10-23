//! Shared image conversion and genuine 1-bit BMP export.

pub mod bmp;
pub mod dither;
pub mod export;
#[cfg(feature = "gui")]
pub mod gui;
pub mod mono;
pub mod pipeline;

pub const WIDTH: u32 = 400;
pub const HEIGHT: u32 = 240;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
