//! Modelled after SuperNOVAS example.c
use hifitime::{prelude::*, ut1::Ut1Provider};
use supernovas::{
    positions::{CatalogEntry, Frame, SurfaceObserver},
    time::Timespec,
    Accuracy,
};

pub fn main() {
    println!("SuperNOVAS-Rust Sample Calculations");
    println!("-----------------------------------\n");

    // Construct an observer on the surface
    let ovro = SurfaceObserver::new(37.2339, -118.282, 1222.0, 10.0, 1010.0);
    println!("OVRO location: {:#?}", ovro);

    // Create a hifitime epoch in UTC
    let epoch = Epoch::from_gregorian_utc(2024, 9, 17, 6, 12, 18, 0);
    // Download the latest JPL UT1 data - this could be provided from EOP data
    let provider = Ut1Provider::from_eop_file("eop2.short").unwrap();
    // Convert to NOVAS Timespec
    let time = Timespec::from((epoch, provider));

    // Setup the observing frame
    let frame = Frame::new(Accuracy::Full, &ovro, &time, 0.0, 0.0).unwrap();

    // Make a catalog entry for a sidereal source (in J2000)
    let entry = CatalogEntry::new(
        "Vega",
        "HIP",
        91262,
        11.88299133,
        37.71867646,
        4003.27,
        -5815.07,
        109.21,
        -98.8,
    )
    .unwrap();
}
