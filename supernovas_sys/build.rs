use std::env;
use std::path::PathBuf;

fn main() {
    // Build supernovas C library
    cc::Build::new()
        .include("vendor/include")
        .file("vendor/src/novas.c")
        .file("vendor/src/nutation.c")
        .file("vendor/src/refract.c")
        .file("vendor/src/frames.c")
        .file("vendor/src/timescale.c")
        .compile("supernovas");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Only build bindings for NOVAS, not for it's dependents
        .allowlist_file(".*novas.h")
        // Use "newtype enums" for the C enums (to avoid UB)
        .newtype_enum("novas_accuracy")
        .newtype_enum("novas_timescale")
        .newtype_enum("novas_cio_location_type")
        .newtype_enum("novas_debug_mode")
        .newtype_enum("novas_dynamical_type")
        .newtype_enum("novas_earth_rotation_measure")
        .newtype_enum("novas_equator_type")
        .newtype_enum("novas_equatorial_class")
        .newtype_enum("novas_equinox_type")
        .newtype_enum("novas_frametie_direction")
        .newtype_enum("novas_nutation_direction")
        .newtype_enum("novas_object_type")
        .newtype_enum("novas_observer_place")
        .newtype_enum("novas_origin")
        .newtype_enum("novas_planet")
        .newtype_enum("novas_pole_offset_type")
        .newtype_enum("novas_reference_system")
        .newtype_enum("novas_refraction_model")
        .newtype_enum("novas_refraction_type")
        .newtype_enum("novas_transform_type")
        .newtype_enum("novas_wobble_direction")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
