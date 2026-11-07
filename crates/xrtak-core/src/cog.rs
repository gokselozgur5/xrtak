use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoTiffMetadata {
    pub width: usize,
    pub height: usize,
    pub bits_per_sample: u16,
    /// 1 = uint, 2 = int, 3 = IEEE float
    pub sample_format: u16,
    pub data_offsets: Vec<usize>,
    /// Optional geographic top-left origin (`lon, lat`) from `ModelTiepointTag` (33922)
    pub origin_lon_lat: Option<(f64, f64)>,
    /// Optional pixel scale (`scale_x, scale_y`) from `ModelPixelScaleTag` (33550)
    pub pixel_scale_xy: Option<(f64, f64)>,
}

#[derive(Clone, Copy)]
enum Endian {
    Little,
    Big,
}

impl Endian {
    fn u16(self, b: &[u8], off: usize) -> Option<u16> {
        let s = b.get(off..off + 2)?;
        Some(match self {
            Self::Little => u16::from_le_bytes([s[0], s[1]]),
            Self::Big => u16::from_be_bytes([s[0], s[1]]),
        })
    }

    fn u32(self, b: &[u8], off: usize) -> Option<u32> {
        let s = b.get(off..off + 4)?;
        Some(match self {
            Self::Little => u32::from_le_bytes([s[0], s[1], s[2], s[3]]),
            Self::Big => u32::from_be_bytes([s[0], s[1], s[2], s[3]]),
        })
    }

    fn i16(self, b: &[u8], off: usize) -> Option<i16> {
        let s = b.get(off..off + 2)?;
        Some(match self {
            Self::Little => i16::from_le_bytes([s[0], s[1]]),
            Self::Big => i16::from_be_bytes([s[0], s[1]]),
        })
    }

    fn f32(self, b: &[u8], off: usize) -> Option<f32> {
        let s = b.get(off..off + 4)?;
        Some(match self {
            Self::Little => f32::from_le_bytes([s[0], s[1], s[2], s[3]]),
            Self::Big => f32::from_be_bytes([s[0], s[1], s[2], s[3]]),
        })
    }

    fn f64(self, b: &[u8], off: usize) -> Option<f64> {
        let s = b.get(off..off + 8)?;
        Some(match self {
            Self::Little => f64::from_le_bytes([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]]),
            Self::Big => f64::from_be_bytes([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]]),
        })
    }
}

/// Parses the primary IFD of a GeoTIFF / Cloud-Optimized GeoTIFF (`REQ-COG-01`).
pub fn parse_geotiff_metadata(bytes: &[u8]) -> Result<GeoTiffMetadata, String> {
    if bytes.len() < 8 {
        return Err("GeoTIFF buffer shorter than 8-byte TIFF header".to_string());
    }
    let endian = match (&bytes[0..2], bytes[2], bytes[3]) {
        (b"II", 42, 0) => Endian::Little,
        (b"MM", 0, 42) => Endian::Big,
        _ => return Err("Invalid TIFF byte-order or magic number (expected II/MM + 42)".to_string()),
    };

    let ifd_offset = endian
        .u32(bytes, 4)
        .ok_or_else(|| "Truncated IFD offset".to_string())? as usize;
    let entry_count = endian
        .u16(bytes, ifd_offset)
        .ok_or_else(|| "Truncated IFD entry count".to_string())? as usize;

    let mut width = 0usize;
    let mut height = 0usize;
    let mut bits_per_sample = 16u16;
    let mut sample_format = 1u16;
    let mut data_offsets = Vec::new();
    let mut origin_lon_lat = None;
    let mut pixel_scale_xy = None;

    for i in 0..entry_count {
        let base = ifd_offset + 2 + i * 12;
        let tag = endian.u16(bytes, base).ok_or("Truncated IFD tag")?;
        let field_type = endian.u16(bytes, base + 2).ok_or("Truncated IFD type")?;
        let count = endian.u32(bytes, base + 4).ok_or("Truncated IFD count")? as usize;
        let val_off = base + 8;

        let read_scalar_u32 = || -> Option<u32> {
            match field_type {
                3 => endian.u16(bytes, val_off).map(u32::from),
                4 => endian.u32(bytes, val_off),
                _ => endian.u32(bytes, val_off),
            }
        };

        match tag {
            256 => width = read_scalar_u32().unwrap_or(0) as usize,
            257 => height = read_scalar_u32().unwrap_or(0) as usize,
            258 => bits_per_sample = read_scalar_u32().unwrap_or(16) as u16,
            339 => sample_format = read_scalar_u32().unwrap_or(1) as u16,
            273 | 324 => {
                if count == 1 {
                    if let Some(off) = read_scalar_u32() {
                        data_offsets.push(off as usize);
                    }
                } else if let Some(arr_off) = endian.u32(bytes, val_off) {
                    let arr_base = arr_off as usize;
                    for k in 0..count {
                        let item = if field_type == 3 {
                            endian.u16(bytes, arr_base + k * 2).map(|v| v as usize)
                        } else {
                            endian.u32(bytes, arr_base + k * 4).map(|v| v as usize)
                        };
                        if let Some(off) = item {
                            data_offsets.push(off);
                        }
                    }
                }
            }
            33550 if count >= 2 => {
                if let Some(arr_off) = endian.u32(bytes, val_off) {
                    let b_off = arr_off as usize;
                    if let (Some(sx), Some(sy)) = (endian.f64(bytes, b_off), endian.f64(bytes, b_off + 8)) {
                        pixel_scale_xy = Some((sx, sy));
                    }
                }
            }
            33922 if count >= 6 => {
                if let Some(arr_off) = endian.u32(bytes, val_off) {
                    let b_off = arr_off as usize;
                    if let (Some(lon), Some(lat)) =
                        (endian.f64(bytes, b_off + 24), endian.f64(bytes, b_off + 32))
                    {
                        origin_lon_lat = Some((lon, lat));
                    }
                }
            }
            _ => {}
        }
    }

    if width == 0 || height == 0 || data_offsets.is_empty() {
        return Err("GeoTIFF missing required ImageWidth, ImageLength, or Strip/TileOffsets".to_string());
    }

    Ok(GeoTiffMetadata {
        width,
        height,
        bits_per_sample,
        sample_format,
        data_offsets,
        origin_lon_lat,
        pixel_scale_xy,
    })
}
