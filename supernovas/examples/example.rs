use hifitime::{prelude::*, ut1::Ut1Provider};
use supernovas::{
    ephem::provide_ephem,
    positions::{CatalogEntry, Frame, Observer, ReferenceSystem},
    time::Timespec,
    Accuracy,
};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the ephemeris
    provide_ephem("supernovas/examples/de440.bsp")?;
    // Construct an observer on the surface
    let ovro = Observer::new_on_surface(37.2339, -118.282, 1222.0, 10.0, 1010.0);
    // Convert from a hifitime Epoch and UT1 provider to  NOVAS Timespec
    let time = Timespec::from((
        Epoch::from_gregorian_utc(2024, 9, 17, 6, 12, 18, 0),
        Ut1Provider::from_eop_file("supernovas/examples/eop2.short")?,
    ));
    // Setup the observing frame (fast, reduced accuracy) which combines the observing time and place
    let frame = Frame::new(Accuracy::Full, &ovro, &time, 0.0, 0.0)?;
    // Make a catalog entry for a sidereal source (in ICRS J2000) we want to point to
    let entry = CatalogEntry::from_simbad("Vega", "HIP")?;
    println!("SIMBAD Result: {:#?}", entry);
    // Compute the pointing
    let now = std::time::SystemTime::now();
    let (az, el) = frame.apparent_local_coordinates(ReferenceSystem::CIRS, &entry)?;

    println!("Az: {az}, El: {el}");

    Ok(())
}

// Az: 294.32856148157197, El: 37.64677260032829  - DE440
// Az: 294.32856125619685, El: 37.646772610004746 - Reduced
