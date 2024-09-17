use supernovas_sys::{novas_debug, novas_debug_mode};

pub mod ephem;
pub mod error;
pub mod positions;
pub mod time;

pub type Result<T> = std::result::Result<T, error::Error>;

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Constants to control the precision of NOVAS nutation calculations.
pub enum Accuracy {
    ///	Use full precision calculations to micro-arcsecond accuracy.
    /// It can be computationally intensive when using the dynamical equator.
    Full = 0,
    /// Calculate with truncated terms. It can be significantly faster if a few milliarcsecond accuracy is sufficient.
    Reduced = 1,
}

/// Enable the debug printing for the underlying C library
pub fn set_debug(enable: bool) {
    unsafe { novas_debug(novas_debug_mode(enable as u32)) }
}
