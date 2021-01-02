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
    Bow85,

    /// Charles F.F. Karney (2011): *Transverse Mercator with an accuracy of a few nanometers*.
    /// J. Geodesy. 85(8): 475–485.
    /// arXiv:1002.1417. Bibcode:2011JGeod..85..475K.
    /// doi:10.1007/s00190-011-0445-3. S2CID 118619524.
    Kar11,

    /// R.E. Deakin, M.N. Hunter and C.F.F. Karney (2012):
    /// A fresh look at the UTM projection:
    /// Karney-Krueger equations.
    /// Surveying and Spatial Sciences Institute (SSSI)
    /// Land Surveying Commission National Conference,
    /// Melbourne, 18-21 April, 2012
    Dea12,

    /// Krüger, L. (1912). Konforme Abbildung des Erdellipsoids in der Ebene.
    /// Royal Prussian Geodetic Institute, New Series 52.
    /// https://dx.doi.org/10.2312/GFZ.b103-krueger28
    Kru12,
}
