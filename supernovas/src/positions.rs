//! Routines for computing positions of local and astronomical objects

use crate::{error::Error, time::Timespec, Accuracy};
use std::{
    ffi::{CStr, CString},
    fmt::Debug,
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::null,
};
use supernovas_sys::{
    cat_entry, make_cat_entry, make_cat_object, make_observer_at_geocenter, make_observer_in_space,
    make_observer_on_surface, novas_accuracy, novas_app_to_hor, novas_frame, novas_make_frame,
    novas_reference_system, novas_sky_pos, novas_transform_type, observer, place_star, sky_pos,
    transform_cat, SIZE_OF_CAT_NAME, SIZE_OF_OBJ_NAME,
};

/// An observer position
pub struct Observer {
    location: ObserverLocation,
    inner: observer,
}

/// The position an observer can be
pub enum ObserverLocation {
    /// A hypothetical observer at the Earth's geocetner
    Geocenter,
    /// An observer on the surface of the earth
    Surface,
    /// An observer in space, neaer earth (like a spacecraft)
    Space,
}

impl Observer {
    /// Construct a new [`Observer`] on the surface of the earth
    ///
    /// - lat: Geodetic (ITRS) latitude in degrees; north positive
    /// - lon: Geodetic (ITRS) longitude in degrees; east positive
    /// - elev: Altidude above sea level in meters
    /// - temp: Temperature in celsius
    /// - pressure: Pressure in mBar
    pub fn new_on_surface(lat: f64, lon: f64, elev: f64, temp: f64, pressure: f64) -> Self {
        let mut obs_loc = MaybeUninit::uninit();
        // Safety: The pointer to the obs_loc will never be null, and that is the only situation where this would error
        let _ = unsafe {
            make_observer_on_surface(lat, lon, elev, temp, pressure, obs_loc.as_mut_ptr())
        };
        // Safety: The above initialization is garunteed to succeed, so this is init
        Self {
            location: ObserverLocation::Surface,
            inner: unsafe { obs_loc.assume_init() },
        }
    }

    /// Construct a new [`Observer`] at the Earth's geocenter
    ///
    /// - pos: (x,y,z) position in km
    /// - vel: (x,y,z) velocity in km/s
    pub fn new_in_space(pos: &[f64; 3], vel: &[f64; 3]) -> Self {
        let mut obs_loc = MaybeUninit::uninit();
        // Safety: The pointer to the obs_loc will never be null, and that is the only situation where this would error
        let _ = unsafe { make_observer_in_space(pos.as_ptr(), vel.as_ptr(), obs_loc.as_mut_ptr()) };
        // Safety: The above initialization is garunteed to succeed, so this is init
        Self {
            location: ObserverLocation::Space,
            inner: unsafe { obs_loc.assume_init() },
        }
    }

    /// Construct a new [`Observer`] at the Earth's geocenter
    pub fn new_at_geocenter() -> Self {
        let mut obs_loc = MaybeUninit::uninit();
        // Safety: The pointer to the obs_loc will never be null, and that is the only situation where this would error
        let _ = unsafe { make_observer_at_geocenter(obs_loc.as_mut_ptr()) };
        // Safety: The above initialization is garunteed to succeed, so this is init
        Self {
            location: ObserverLocation::Geocenter,
            inner: unsafe { obs_loc.assume_init() },
        }
    }
}

// Spoof the debug print for the inner struct
impl Debug for Observer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurfaceObserver")
            .field("longitude", &self.inner.on_surf.longitude)
            .field("latitude", &self.inner.on_surf.latitude)
            .field("elevation", &self.inner.on_surf.height)
            .field("temperature", &self.inner.on_surf.temperature)
            .field("pressure", &self.inner.on_surf.pressure)
            .finish()
    }
}

#[derive(Debug)]
/// Coordinate transformations for [`CatEntry::transform`]
pub enum Transformation {
    /// Update the catalog entry to account for proper motion between two dates in a fixed frame
    ProperMotion { jd_tt_in: f64, jd_tt_out: f64 },
    /// Applies rotation to the reference frame
    Precession { jd_tt_in: f64, jd_tt_out: f64 },
    /// Combined action of proper motion and precession
    ChangeEpoch { jd_tt_in: f64, jd_tt_out: f64 },
    /// Transorm from dynamical system of J2000.0 to ICRS
    J2000ToICRS,
    /// Inverse transformation of J2000 To ICRS
    ICRSToJ2000,
}

impl From<Transformation> for novas_transform_type {
    fn from(value: Transformation) -> Self {
        match value {
            Transformation::ProperMotion { .. } => novas_transform_type::PROPER_MOTION,
            Transformation::Precession { .. } => novas_transform_type::PRECESSION,
            Transformation::ChangeEpoch { .. } => novas_transform_type::CHANGE_EPOCH,
            Transformation::J2000ToICRS => novas_transform_type::CHANGE_J2000_TO_ICRS,
            Transformation::ICRSToJ2000 => novas_transform_type::CHANGE_ICRS_TO_J2000,
        }
    }
}

/// Astronmetric data for any sidereal object located outside the solar system
pub struct CatalogEntry(pub cat_entry);

impl CatalogEntry {
    /// Construct a new catalog entry
    ///
    /// - name: The object name
    /// - catalog: The catalog identifier
    /// - cat_num: The object number in the catalog
    /// - ra: Right ascensioni in hours
    /// - dec: Declination of the object in degrees
    /// - pm_ra: Proper motion in right ascension in mas/yr
    /// - pm_dec: Proper motion in declination in mas/yr
    /// - parallax: Parallax in mas
    /// - rad_vel: Radial velocity of the object in km/s
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

    /// Construct a new CatalogEntry from ra and dec in HMS, DMS instead of fracional hour and degree
    pub fn new_hms(
        name: &str,
        catalog: &str,
        num: i64,
        ra: (u8, u8, f64),
        dec: (u16, u16, f64),
        pm_ra: f64,
        pm_dec: f64,
        parallax: f64,
        rad_vel: f64,
    ) -> super::Result<Self> {
        let ra = (ra.0 as f64) + (ra.1 as f64) / 60.0 + (ra.2 as f64) / 3600.0;
        let dec = (dec.0 as f64) + (dec.1 as f64) / 60.0 + (dec.2 as f64) / 3600.0;
        Self::new(
            name, catalog, num, ra, dec, pm_ra, pm_dec, parallax, rad_vel,
        )
    }

    /// Transform this catalog entry into another coordinate system with an optional new catalog name
    ///
    /// See docs on constraints [here](https://smithsonian.github.io/SuperNOVAS/apidoc/html/novas_8h.html#a59caeca70d1fdd02e41ed62f20675e6c)
    pub fn transform(
        &mut self,
        transformation: Transformation,
        new_cat: Option<String>,
    ) -> super::Result<()> {
        // Create the jd_tt_in and out values
        let (jd_tt_in, jd_tt_out) = match transformation {
            Transformation::ProperMotion {
                jd_tt_in,
                jd_tt_out,
            } => (jd_tt_in, jd_tt_out),
            Transformation::Precession {
                jd_tt_in,
                jd_tt_out,
            } => (jd_tt_in, jd_tt_out),
            Transformation::ChangeEpoch {
                jd_tt_in,
                jd_tt_out,
            } => (jd_tt_in, jd_tt_out),
            Transformation::J2000ToICRS => (0.0, 0.0),
            Transformation::ICRSToJ2000 => (0.0, 0.0),
        };
        // Deal with catalog name (if it exists)
        let out_id;
        if let Some(catalog) = new_cat {
            if catalog.len() as u32 > SIZE_OF_CAT_NAME {
                return Err(Error::InvalidString);
            } else {
                let new_cat_c = CString::new(catalog).map_err(|_| Error::InvalidString)?;
                out_id = new_cat_c.as_ptr();
            }
        } else {
            out_id = null();
        }
        // Safety: We've checked the length of the string already and the arguments will not be null
        unsafe {
            let _ = transform_cat(
                transformation.into(),
                jd_tt_in,
                &self.0 as *const _,
                jd_tt_out,
                out_id,
                &mut self.0 as *mut _,
            );
        }
        Ok(())
    }
}

impl Debug for CatalogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Safety: We created these strings and checked for validity then, so they *should* still be valid here
        let name = unsafe { CStr::from_ptr(self.0.starname.as_ptr()) };
        let catalog = unsafe { CStr::from_ptr(self.0.catalog.as_ptr()) };
        f.debug_struct("CatalogEntry")
            .field("name", &name)
            .field("catalog", &catalog)
            .field("number", &self.0.starnumber)
            .field("ra", &self.0.ra)
            .field("dec", &self.0.dec)
            .field("pm_ra", &self.0.promora)
            .field("pm_dec", &self.0.promodec)
            .field("parallax", &self.0.parallax)
            .field("rad_vel", &self.0.radialvelocity)
            .finish()
    }
}

/// A set of parameters that uniquely define the place and time of observation
pub struct Frame<'a> {
    inner: novas_frame,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Frame<'a> {
    pub fn new(
        acc: Accuracy,
        obs: &'a Observer,
        time: &'a Timespec,
        dx: f64,
        dy: f64,
    ) -> super::Result<Self> {
        // NOTE: This structure holds on to references to the observer and time, so it must capture their lifetimes
        let mut frame = MaybeUninit::uninit();
        let frame = unsafe {
            let ret = novas_make_frame(
                acc.into(),
                &(obs.inner) as *const _,
                &(time.0) as *const _,
                dx,
                dy,
                frame.as_mut_ptr(),
            );
            // check ret
            if ret != 0 {
                return Err(Error::LowerLevel(ret));
            }
            frame.assume_init()
        };
        Ok(Frame {
            inner: frame,
            _marker: PhantomData,
        })
    }

    /// Computes the local coordinates (az,el in degrees) of a catalog (sidereal) source in the given ReferenceSystem
    pub fn apparent_local_coordinates(
        &self,
        ref_sys: ReferenceSystem,
        entry: &CatalogEntry,
    ) -> super::Result<(f64, f64)> {
        // Ignore refraction for now

        // Compute the apparent position
        let sky_pos = SkyPosition::try_from_frame_entry(entry, self, ref_sys)?;

        let mut az = MaybeUninit::uninit();
        let mut el = MaybeUninit::uninit();

        let (az, el) = unsafe {
            let _ = novas_app_to_hor(
                &self.inner as *const _,
                ref_sys.into(),
                sky_pos.ra(),
                sky_pos.dec(),
                None,
                az.as_mut_ptr(),
                el.as_mut_ptr(),
            );
            (az.assume_init(), el.assume_init())
        };

        Ok((az, el))
    }
}

/// Positional coordinaate reference systems
///
/// These determine only how the celestial pole is to be located, but not how velocities are to be referenced.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ReferenceSystem {
    /// Geocentric Celestial Reference system.
    ///
    /// Essentially the same as ICRS but includes aberration and gravitational deflection for an observer around Earth.
    GCRS,
    /// True equinox Of Date: dynamical system of the 'true' equator, with its origin at the 'true' equinox (pre IAU 2006 system).
    ///
    /// cIt is inherently less precise than the new standard CIRS because mainly because it is based on separate, and less-precise, precession and nutation models (Lieske et. al. 1977).
    TOD,
    /// Celestial Intermediate Reference System: dynamical system of the true equator, with its origin at the CIO (preferred since IAU 2006)
    CIRS,
    /// International Celestial Reference system. The equatorial system fixed to the frame of distant quasars.
    ICRS,
    /// The J2000 dynamical reference system
    J2000,
    /// Mean equinox of date: dynamical system of the 'mean' equator, with its origin at the 'mean' equinox (pre IAU 2006 system).
    ///
    /// It includes precession (Lieske et. al. 1977), but no nutation.
    MOD,
}

impl From<ReferenceSystem> for novas_reference_system {
    fn from(value: ReferenceSystem) -> Self {
        match value {
            ReferenceSystem::GCRS => novas_reference_system::NOVAS_GCRS,
            ReferenceSystem::TOD => novas_reference_system::NOVAS_TOD,
            ReferenceSystem::CIRS => novas_reference_system::NOVAS_CIRS,
            ReferenceSystem::ICRS => novas_reference_system::NOVAS_ICRS,
            ReferenceSystem::J2000 => novas_reference_system::NOVAS_J2000,
            ReferenceSystem::MOD => novas_reference_system::NOVAS_MOD,
        }
    }
}

/// A celestial object's place on the sky
pub struct SkyPosition(sky_pos);

impl SkyPosition {
    /// Apparent, topocentric, or astrometric declination in degrees
    pub fn dec(&self) -> f64 {
        self.0.dec
    }

    /// Apparent, topocentric, or astrometric right ascension in hours
    pub fn ra(&self) -> f64 {
        self.0.ra
    }

    /// Radial velocity in km/s
    pub fn rad_vel(&self) -> f64 {
        self.0.rv
    }

    /// True (geometric, Euclidian) distance to solar system body (if it is a solar system body)
    pub fn distance(&self) -> Option<f64> {
        if self.0.dis != 0.0 {
            Some(self.0.dis)
        } else {
            None
        }
    }

    /// Unit vector towards object (dimensionless)
    pub fn r_hat(&self) -> &[f64; 3] {
        &self.0.r_hat
    }

    /// Calculates an apparent location on the sky for a CatalogEntry
    ///
    /// This takes into account proper motion
    pub fn try_from_frame_entry(
        entry: &CatalogEntry,
        frame: &Frame,
        ref_sys: ReferenceSystem,
    ) -> super::Result<Self> {
        // First we need to make the `object` structure from the catalog entry
        let mut obj = MaybeUninit::uninit();
        // Safety: Nothing here is null and names and numbers are valid
        // This is copying data into the object, so lifetimes here are ok
        let obj = unsafe {
            let _ = make_cat_object(&entry.0 as *const _, obj.as_mut_ptr());
            obj.assume_init()
        };
        // The compute the sky position
        let mut sky_pos = MaybeUninit::uninit();
        let sky_pos = unsafe {
            let ret = novas_sky_pos(
                &obj as *const _,
                &frame.inner as *const _,
                ref_sys.into(),
                sky_pos.as_mut_ptr(),
            );
            assert_eq!(ret, 0);
            sky_pos.assume_init()
        };

        Ok(Self(sky_pos))
    }

    pub fn place(
        jd_tt: f64,
        entry: &CatalogEntry,
        obs: &Observer,
        ut1_to_tt: f64,
        ref_sys: ReferenceSystem,
        acc: Accuracy,
    ) -> super::Result<Self> {
        let mut sky_pos = MaybeUninit::uninit();
        let sky_pos = unsafe {
            let ret = place_star(
                jd_tt,
                &entry.0 as *const _,
                &obs.inner as *const _,
                ut1_to_tt,
                ref_sys.into(),
                acc.into(),
                sky_pos.as_mut_ptr(),
            );
            assert_eq!(ret, 0);
            sky_pos.assume_init()
        };

        Ok(Self(sky_pos))
    }
}

impl Debug for SkyPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SkyPosition").field(&self.0).finish()
    }
}
