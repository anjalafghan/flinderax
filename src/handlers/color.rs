pub fn pack(color: (u8, u8, u8)) -> i32 {
    ((color.0 as i32) << 16) | ((color.1 as i32) << 8) | (color.2 as i32)
}

pub fn unpack(color: i32) -> (u8, u8, u8) {
    (
        ((color >> 16) & 0xFF) as u8,
        ((color >> 8) & 0xFF) as u8,
        (color & 0xFF) as u8,
    )
}
