use hifitime::{prelude::*, ut1::Ut1Provider};
use supernovas::{
    positions::{CatalogEntry, Frame, ReferenceSystem, SurfaceObserver},
    time::Timespec,
    Accuracy,
};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Construct an observer on the surface
    let ovro = SurfaceObserver::new(37.2339, -118.282, 1222.0, 10.0, 1010.0);
    // Convert from a hifitime Epoch and UT1 provider to  NOVAS Timespec
    let time = Timespec::from((
        Epoch::from_gregorian_utc(2024, 9, 17, 6, 12, 18, 0),
        Ut1Provider::from_eop_file("supernovas/examples/eop2.short")?,
    ));
    // Setup the observing frame (fast, reduced accuracy) which combines the observing time and place
    let frame = Frame::new(Accuracy::Reduced, &ovro, &time, 0.0, 0.0)?;
    // Make a catalog entry for a sidereal source (in ICRS J2000) we want to point to
    let entry = CatalogEntry::from_simbad("Virgo A", "NGC")?;
    println!("Object: {:#?}", entry);
    // Compute the pointing
    let (az, el) = frame.apparent_local_coordinates(ReferenceSystem::CIRS, &entry)?;

    println!("Az: {az}, El: {el}");

    Ok(())
}
