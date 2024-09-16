//! Modelled after SuperNOVAS example.c
use hifitime::{prelude::*, ut1::Ut1Provider};
use supernovas::{
    positions::{CatalogEntry, SurfaceObserver},
    time::Timespec,
    Accuracy,
};

pub fn main() {
    println!("SuperNOVAS-Rust Sample Calculations");
    println!("-----------------------------------\n");

    // Construct an observer on the surface
    let location = SurfaceObserver::new(42.0, -70.0, 0.0, 10.0, 1010.0);
    println!("Geodetic location: {:#?}", location);

    // Create a hifitime epoch in UTC
    let epoch = Epoch::from_gregorian_utc(2008, 4, 24, 10, 36, 18, 0);
    // Download the latest JPL UT1 data - this could be provided from EOP data
    let provider = Ut1Provider::download_from_jpl("latest_eop2.long").unwrap();
    // Convert to NOVAS Timespec
    let time = Timespec::from((epoch, provider));
    println!("Timespec: {:#?}", time);

    // Setup the observing frame
    //let frame = Frame::new(Accuracy::Full, &location, &time, 0.0, 0.0).unwrap();

    // Make a catalog entry for a sidereal source
    let entry = CatalogEntry::new(
        "GMB 1830",
        "FK6",
        1307,
        11.88299133,
        37.71867646,
        4003.27,
        -5815.07,
        109.21,
        -98.8,
    )
    .unwrap();
}
