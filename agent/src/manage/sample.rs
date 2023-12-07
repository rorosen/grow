use std::num::ParseIntError;

pub mod air;
pub mod light;
pub mod water_level;

fn parse_hex_u8(src: &str) -> Result<u8, ParseIntError> {
    u8::from_str_radix(src.trim_start_matches("0x"), 16)
}
