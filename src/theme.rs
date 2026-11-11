//! Colors and sizes used by the whole interface.

pub const BACKGROUND: u32 = 0x11131a;
pub const SURFACE: u32 = 0x1a1d27;
pub const SURFACE_HIGH: u32 = 0x252a3a;
pub const BORDER: u32 = 0x2d3245;
pub const TEXT: u32 = 0xeceff7;
pub const TEXT_DIM: u32 = 0x8b93a8;
pub const ACCENT: u32 = 0x5b8cff;
pub const GOOD: u32 = 0x34d399;
pub const WARNING: u32 = 0xfbbf24;
pub const DANGER: u32 = 0xf87171;
pub const WARNING_BACKGROUND: u32 = 0x3a2f12;

/// Window size at start-up and the smallest size the user can shrink it to.
pub const START_SIZE: (f32, f32) = (1100.0, 800.0);
pub const MIN_SIZE: (f32, f32) = (480.0, 420.0);
