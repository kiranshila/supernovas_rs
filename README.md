# supernovas_rs

A safe Rust wrapper around the [SuperNOVAS](https://smithsonian.github.io/SuperNOVAS/) astrometry library.

## Why

- There is no reason to use C for production in this century. One segfault is too many.
- Error handling is elegant, explicit, and always handled
- RAII patterns make working with files and various pieces of memory significantly more ergonomic

## Implementation Details
### "Alternative Methodologies" for Coordinate Transformations

SuperNOVAS implements the new IAU standards (after IAU 2000) for coordinate transformations.
These are higher precision and faster than the classic NOVAS computations.
The old standards are available in NOVAS and SuperNOVAS, but are not wrapped here because there isn't really a reason to use them.

### Time

There are no high-level wrappers around most (Super)NOVAS time routines as these are already implemented in the
fantastic [hifitime](https://docs.rs/hifitime/latest/hifitime/) library, with more speed, accuracy, and safety.
Conversions to NOVAS `Timescale`s is enabled when the `hifitime` feature is enabled (as it is by default).