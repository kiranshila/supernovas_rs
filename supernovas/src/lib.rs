use supernovas_sys::{novas_accuracy, novas_debug, novas_debug_mode};

#[cfg(feature = "calceph")]
pub mod ephem;
pub mod error;
pub mod positions;
pub mod simbad;
pub mod time;

pub type Result<T> = std::result::Result<T, error::Error>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Constants to control the precision of NOVAS nutation calculations.
pub enum Accuracy {
    ///	Use full precision calculations to micro-arcsecond accuracy.
    /// It can be computationally intensive when using the dynamical equator.
    Full,
    /// Calculate with truncated terms. It can be significantly faster if a few milliarcsecond accuracy is sufficient.
    Reduced,
}

impl From<Accuracy> for novas_accuracy {
    fn from(value: Accuracy) -> Self {
        match value {
            Accuracy::Full => novas_accuracy::NOVAS_FULL_ACCURACY,
            Accuracy::Reduced => novas_accuracy::NOVAS_REDUCED_ACCURACY,
        }
    }
}

/// Enable the debug printing for the underlying C library
pub fn set_debug(enable: bool) {
    unsafe { novas_debug(novas_debug_mode(enable as u32)) }
}
