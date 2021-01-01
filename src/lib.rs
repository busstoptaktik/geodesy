//! A playground for experimentation with alternative models for geodetic
//! data flow and coordinate representation.
//!
//! Specifically designed to facilitate experiments toward solving
//! identified shortcomings in the [PROJ](https://proj.org) data flow,
//! and the [ISO-19111](https://www.iso.org/standard/74039.html) model
//! for referencing by coordinates.
//!
//! Thomas Knudsen, thokn@sdfe.dk, 2020/2021

pub mod coordinates;
pub mod ellipsoids;
pub mod operators;

pub use ellipsoids::Ellipsoid;
pub use coordinates::CoordinateTuple;
pub use coordinates::DMS;


/// Literature, that has been useful in designing and implementing this library.
pub enum Bibliography {
    /// B.R. Bowring (1976): *Transformation from spatial to geographical coordinates*.
    /// Survey Review 23(181), pp. 323–327
    Bow76,
    /// B.R. Bowring (1985): *The accuracy of geodetic latitude and height equations*.
    /// Survey Review, 28(218), pp.202-206, DOI: 10.1179/sre.1985.28.218.202
    Bow85
}
