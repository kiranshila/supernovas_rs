//! Routines for computing positions of local and astronomical objects

use crate::{time::Timespec, Accuracy};

use super::Error;
use std::{ffi::CString, fmt::Debug, marker::PhantomData, mem::MaybeUninit};
use supernovas_sys::{
    cat_entry, make_cat_entry, make_observer_on_surface, novas_accuracy, novas_frame,
    novas_make_frame, observer, SIZE_OF_CAT_NAME, SIZE_OF_OBJ_NAME,
};

/// An observer on the surface
#[repr(transparent)]
pub struct SurfaceObserver(observer);

impl SurfaceObserver {
    /// Construct a new SurfaceObserver
    ///
    /// - lat: Geodetic (ITRS) latitude in degrees; north positive
    /// - lon: Geodetic (ITRS) longitude in degrees; east positive
    /// - elev: Altidude above sea level in meters
    /// - temp: Temperature in celsius
    /// - pressure: Pressure in mBar
    pub fn new(lat: f64, lon: f64, elev: f64, temp: f64, pressure: f64) -> Self {
        let mut obs_loc = MaybeUninit::uninit();
        // Safety: The pointer to the obs_loc will never be null, and that is the only situation where this would error
        let _ = unsafe {
            make_observer_on_surface(lat, lon, elev, temp, pressure, obs_loc.as_mut_ptr())
        };
        // Safety: The above initialization is garunteed to succeed, so this is init
        Self(unsafe { obs_loc.assume_init() })
    }

    /// Get the configured observation location longitude (Geodetic in degrees; north positive)
    pub fn longitude(&self) -> f64 {
        self.0.on_surf.longitude
    }

    /// Get the configured observation location latitude (Geodetic in degrees; east positive)
    pub fn latitude(&self) -> f64 {
        self.0.on_surf.latitude
    }

    /// Get the configured observation location elevation (Above sea level, meters)
    pub fn elevation(&self) -> f64 {
        self.0.on_surf.height
    }

    /// Get the configured observation location temperature (Celcius)
    pub fn temperature(&self) -> f64 {
        self.0.on_surf.temperature
    }

    /// Get the configured observation location pressure (mBar)
    pub fn pressure(&self) -> f64 {
        self.0.on_surf.pressure
    }
}

// Spoof the debug print for the inner struct
impl Debug for SurfaceObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurfaceObserver")
            .field("longitude", &self.longitude())
            .field("latitude", &self.latitude())
            .field("elevation", &self.elevation())
            .field("temperature", &self.temperature())
            .field("pressure", &self.pressure())
            .finish()
    }
}

/// Astronmetric data for any sidereal object located outside the solar system
pub struct CatalogEntry(cat_entry);

impl CatalogEntry {
    pub fn new(
        name: &str,
        catalog: &str,
        num: i64,
        ra: f64,
        dec: f64,
        pm_ra: f64,
        pm_dec: f64,
        parallax: f64,
        rad_vel: f64,
    ) -> super::Result<Self> {
        // Check string sizes
        if name.len() as u32 > SIZE_OF_OBJ_NAME {
            return Err(Error::InvalidString);
        }
        if catalog.len() as u32 > SIZE_OF_CAT_NAME {
            return Err(Error::InvalidString);
        }
        let mut entry = MaybeUninit::uninit();
        // We need to do allocations here because C needs the extra byte for the \0
        let catalog = CString::new(catalog).map_err(|_| Error::InvalidString)?;
        let name = CString::new(name).map_err(|_| Error::InvalidString)?;
        let entry = unsafe {
            // Safety: We're going to check the string lengths before we call, and the struct will not be NULL
            // Internally, this does a strcpy, so its ok that C doesn't own this memory
            let _ = make_cat_entry(
                name.as_ptr(),
                catalog.as_ptr(),
                num,
                ra,
                dec,
                pm_ra,
                pm_dec,
                parallax,
                rad_vel,
                entry.as_mut_ptr(),
            );
            entry.assume_init()
        };
        Ok(Self(entry))
    }
}

// /// A set of parameters that uniquely define the place and time of observation
// pub struct Frame<'a> {
//     inner: novas_frame,
//     obs_marker: PhantomData<&'a SurfaceObserver>,
//     time_marker: PhantomData<&'a Timespec>,
// }

// impl<'a> Frame<'a> {
//     pub fn new(
//         acc: Accuracy,
//         obs: &SurfaceObserver,
//         time: &Timespec,
//         dx: f64,
//         dy: f64,
//     ) -> super::Result<Self> {
//         let inner_acc = novas_accuracy(acc as u32);
//         // NOTE: This structure holds on to references to the observer and time, so it must capture their lifetimes
//         let mut frame = MaybeUninit::uninit();
//         let frame = unsafe {
//             let ret = novas_make_frame(
//                 inner_acc,
//                 &(obs.0) as *const _,
//                 &(time.0) as *const _,
//                 dx,
//                 dy,
//                 frame.as_mut_ptr(),
//             );
//             // check ret
//             assert_eq!(ret, 0);
//             frame.assume_init()
//         };
//         Ok(Frame {
//             inner: frame,
//             obs_marker: PhantomData,
//             time_marker: PhantomData,
//         })
//     }
// }
