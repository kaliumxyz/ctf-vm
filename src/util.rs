

pub fn to_u16 (higher: u8, lower: u8) -> u16 {
    return (higher as u16) << 8 | lower as u16;
}
