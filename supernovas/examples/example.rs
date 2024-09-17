//! Modelled after SuperNOVAS example.c
use hifitime::{prelude::*, ut1::Ut1Provider};
use supernovas::{
    positions::{CatalogEntry, Frame, SurfaceObserver},
    set_debug,
    time::Timespec,
    Accuracy,
};

pub fn main() {
    println!("SuperNOVAS-Rust Sample Calculations");
    println!("-----------------------------------\n");

    set_debug(true);

    // Construct an observer on the surface
    let ovro = SurfaceObserver::new(37.2339, -118.282, 1222.0, 10.0, 1010.0);
    println!("OVRO location: {:#?}", ovro);

    // Create a hifitime epoch in UTC
    let epoch = Epoch::from_gregorian_utc(2024, 9, 17, 6, 12, 18, 0);
    // Download the latest JPL UT1 data - this could be provided from EOP data
    let provider = Ut1Provider::from_eop_file("supernovas/examples/eop2.short").unwrap();
    // Convert to NOVAS Timespec
    let time = Timespec::from((epoch, provider));

    // Setup the observing frame (fast, reduced accuracy)
    let frame = Frame::new(Accuracy::Reduced, &ovro, &time, 0.0, 0.0).unwrap();

    // Make a catalog entry for a sidereal source (in ICRS J2000)
    let entry = CatalogEntry::new_hms(
        "Vega",
        "HIP",
        91262,
        (18, 36, 56.33635),
        (38, 47, 1.2802),
        200.94,
        19.23725227,
        130.23,
        -13.5,
    )
    .unwrap();

    dbg!(entry);
}
