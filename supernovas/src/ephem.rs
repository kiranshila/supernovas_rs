//! Module wrapping working with ephemeris

use calceph::{CalcephBin, PositionUnit, TimeUnit};
use std::{
    ffi::{c_char, c_double, c_int, c_long, c_short},
    path::Path,
    slice,
    sync::{LazyLock, Mutex},
};
use supernovas_sys::{
    novas_origin, novas_planet, set_ephem_provider, set_planet_provider, set_planet_provider_hp,
};

/// 2012 definition of the astronomical unit from the IAU in km
const AU: f64 = 149_597_870.700;

static EPHEM_PROVIDER: LazyLock<Mutex<Option<CalcephBin>>> = LazyLock::new(|| Mutex::new(None));

pub fn naif_ephem_lookup(
    id: i32,
    jd_tdb_high: f64,
    jd_tdb_low: f64,
    origin: novas_origin,
) -> super::Result<[f64; 6]> {
    // Grab the global provider
    let mut ceph = EPHEM_PROVIDER.lock().unwrap();
    let center = match origin {
        novas_origin::NOVAS_BARYCENTER => 0,   // NAIFID_SSB
        novas_origin::NOVAS_HELIOCENTER => 10, // NAIFID_SUN
        _ => unreachable!(),
    };
    let mut pv = match &mut *ceph {
        None => return Err(crate::error::Error::EphemNotLoaded),
        Some(c) => c.compute_position_units_naif(
            jd_tdb_high,
            jd_tdb_low,
            id,
            center,
            PositionUnit::Kilometer,
            TimeUnit::Day,
        )?,
    };
    // Convert result to AU and AU/s
    pv.iter_mut().for_each(|i| *i /= AU);
    Ok(pv)
}

unsafe extern "C" fn ceph_ephem_provider(
    _name: *const c_char,
    id: c_long,
    jd_tdb_high: c_double,
    jd_tdb_low: c_double,
    origin: *mut novas_origin,
    pos: *mut c_double,
    vel: *mut c_double,
) -> c_int {
    // Just always use SSB here
    if origin.is_null() {
        return -1;
    }
    *origin = novas_origin::NOVAS_BARYCENTER;
    match naif_ephem_lookup(
        id.try_into().expect("Invalid NAIFID"),
        jd_tdb_high,
        jd_tdb_low,
        *origin,
    ) {
        Ok(pv) => {
            if !pos.is_null() {
                let slice = slice::from_raw_parts_mut(pos, 3);
                slice.clone_from_slice(&pv[..3]);
            }
            if !vel.is_null() {
                let slice = slice::from_raw_parts_mut(vel, 3);
                slice.clone_from_slice(&pv[3..]);
            }
            0
        }
        Err(_) => -1,
    }
}

fn novas_planet_naif(planet: novas_planet) -> i32 {
    match planet {
        novas_planet::NOVAS_SSB => 0,
        novas_planet::NOVAS_MERCURY => 199,
        novas_planet::NOVAS_VENUS => 299,
        novas_planet::NOVAS_EARTH => 399,
        novas_planet::NOVAS_MARS => 499,
        novas_planet::NOVAS_JUPITER => 599,
        novas_planet::NOVAS_SATURN => 699,
        novas_planet::NOVAS_URANUS => 799,
        novas_planet::NOVAS_NEPTUNE => 899,
        novas_planet::NOVAS_PLUTO => 999,
        novas_planet::NOVAS_SUN => 10,
        novas_planet::NOVAS_MOON => 301,
        _ => unreachable!(),
    }
}

unsafe extern "C" fn ceph_planet_provider_hp(
    jd_tdb: *const c_double,
    body: novas_planet,
    origin: novas_origin,
    pos: *mut c_double,
    vel: *mut c_double,
) -> c_short {
    if jd_tdb.is_null() || pos.is_null() || vel.is_null() {
        return 3;
    }
    let jd_tdb = slice::from_raw_parts(jd_tdb, 2);
    // Perfom the computation
    match naif_ephem_lookup(novas_planet_naif(body), jd_tdb[0], jd_tdb[1], origin) {
        Err(_) => 3,
        Ok(pv) => {
            let pos_slice = slice::from_raw_parts_mut(pos, 3);
            pos_slice.clone_from_slice(&pv[..3]);
            let vel_slice = slice::from_raw_parts_mut(vel, 3);
            vel_slice.clone_from_slice(&pv[3..]);
            0
        }
    }
}

unsafe extern "C" fn ceph_planet_provider(
    jd_tdb: c_double,
    body: novas_planet,
    origin: novas_origin,
    pos: *mut c_double,
    vel: *mut c_double,
) -> c_short {
    if pos.is_null() || vel.is_null() {
        return 3;
    }
    // Perfom the computation
    match naif_ephem_lookup(novas_planet_naif(body), jd_tdb, 0.0, origin) {
        Err(_) => 3,
        Ok(pv) => {
            let pos_slice = slice::from_raw_parts_mut(pos, 3);
            pos_slice.clone_from_slice(&pv[..3]);
            let vel_slice = slice::from_raw_parts_mut(vel, 3);
            vel_slice.clone_from_slice(&pv[3..]);
            0
        }
    }
}

/// Provide high-precision ephemeris for the major planets, overriding the default behavior
pub fn provide_ephem<P: AsRef<Path>>(file: P) -> super::Result<()> {
    // Try to load the file
    let ceph = CalcephBin::new(file)?;
    // Update the gloabl provider
    let mut provider = EPHEM_PROVIDER.lock().unwrap();
    *provider = Some(ceph);
    // Attach the provider to SuperNOVAS
    unsafe {
        set_ephem_provider(Some(ceph_ephem_provider));
        set_planet_provider(Some(ceph_planet_provider));
        set_planet_provider_hp(Some(ceph_planet_provider_hp));
    }
    Ok(())
}
