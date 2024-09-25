//! Routines involving construction and conversion between instances in time in different time scales

use std::{fmt::Debug, mem::MaybeUninit};
use supernovas_sys::{novas_set_split_time, novas_timescale, novas_timespec};

#[cfg(feature = "hifitime")]
use hifitime::{ut1::Ut1Provider, Duration, Epoch};

#[repr(u32)]
#[allow(unused)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Timescale {
    /// Barycentric Coordinate Time (TCB)
    TCB = 0,
    /// Barycentric Dynamical Time (TDB)
    TDB = 1,
    /// Geocentric Coordinate Time (TCG)
    TCG = 2,
    /// Terrestrial Time (TT)
    TT = 3,
    /// Innternational Atomic Time (TAI)
    TAI = 4,
    /// GPS Time
    GPS = 5,
    /// Universal Coordinated Time (UTC)
    UTC = 6,
    /// UT1 earth rotation time, based on the measured Earth orientation parameters published in IERS Bulletin A.
    UT1 = 7,
}

/// The instant object needed for time calculations
#[repr(transparent)]
pub struct Timespec(pub(crate) novas_timespec);

impl Timespec {
    /// Sets an astronomical time to the split Julian Date value, defined in the specified timescale.
    ///
    /// The split into the integer and fractional parts can be done in any convenient way.
    /// The highest precision is reached if the fractional part is ≤ 1 day.
    /// In that case, the time may be specified to picosecond accuracy, if needed.
    ///
    /// - timescale: Timescale of the provided time
    /// - ijd: Integer part in Julian days in the specified timescale
    /// - fjd: Fractional part in Julian days in the specified timescale
    /// - leap: Leap seconds, e.g. as published by IERS Bulletin C
    /// - dut1: UT1-UTC time difference, e.g. as published in IERS Bulletin A in seconds
    pub fn from_split_time(timescale: Timescale, ijd: i64, fjd: f64, leap: i32, dut1: f64) -> Self {
        let inner_ts = novas_timescale(timescale as u32);
        let mut ts = MaybeUninit::uninit();
        let ts = unsafe {
            let _ = novas_set_split_time(inner_ts, ijd, fjd, leap, dut1, ts.as_mut_ptr());
            ts.assume_init()
        };
        Timespec(ts)
    }
}

// Spoof the debug print for the inner struct
impl Debug for Timespec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Timespec")
            .field("dut1", &self.0.dut1)
            .field("fjd_tt", &self.0.fjd_tt)
            .field("ijd_tt", &self.0.ijd_tt)
            .field("tt2tdb", &self.0.tt2tdb)
            .field("ut1_to_tt", &self.0.ut1_to_tt)
            .finish()
    }
}

#[cfg(feature = "hifitime")]
impl From<(Epoch, Ut1Provider)> for Timespec {
    fn from(item: (Epoch, Ut1Provider)) -> Self {
        // hifitime is "TAI-native" while NOVAS likes to work in "TT"
        // This is a little weird because TT is derived from TAI, so it would be "more correct" to work in TT
        // Here, we use hifitime to perform the conversions between whatever timescale the Epoch is in to TT as we don't trust the implementation in the C
        // Case and point, there is a bug in SuperNOVAS where the conversion between TAI and TT is 3us off

        // Extract the TT time (TT days since the Julian epoch)
        let tt = item.0.to_jde_tt_duration();
        let (_, d, h, m, s, ms, us, ns) = tt.decompose();
        let ijd_tt = d as i64;
        // Recompose the days remainder as a single float
        let tt_remainder = Duration::compose(1, 0, h, m, s, ms, us, ns);
        let fjd_tt = tt_remainder.to_seconds();

        // Get the total accumulated leap seconds
        let leap = item.0.leap_seconds_iers();

        // Compute UT1-UTC difference
        let utc = item.0.to_utc_duration();
        let ut1 = item.0.to_ut1_duration(item.1);
        let dut1 = (ut1 - utc).to_seconds();

        Timespec::from_split_time(Timescale::TT, ijd_tt, fjd_tt, leap, dut1)
    }
}
