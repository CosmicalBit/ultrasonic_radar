#[repr(u8)]
pub enum SendHeader {
    Point = 0,
}

impl SendHeader {
    pub const fn to_bytes(self) -> [u8; 1] {
        let num = self as u8;
        num.to_be_bytes()
    }

    pub fn from_bytes(bytes: [u8; 1]) -> Self {
        match bytes[0] {
            0 => Self::Point,
            _ => todo!(),
        }
    }
}
