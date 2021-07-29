
/// Some literature, that has been useful in designing and implementing this library.
pub enum Bibliography {
    /// B.R. Bowring (1976): *Transformation from spatial to geographical coordinates*.
    /// Survey Review 23(181), pp. 323–327.
    Bow76,

    /// B. R. Bowring (1983): *New equations for meridional distance*.
    /// Bull. Geodesique 57, 374–381.
    /// [DOI](https://doi.org/10.1007/BF02520940).
    Bow83,

    /// B.R. Bowring (1985): *The accuracy of geodetic latitude and height equations*.
    /// Survey Review, 28(218), pp.202-206,
    /// [DOI](https://doi.org/10.1179/sre.1985.28.218.202)
    Bow85,

    /// B.R. Bowring (1989): *Transverse mercator equations obtained from a spherical basis*.
    /// Survey Review 30(233), pp.125-133,
    /// [DOI](https://doi.org/10.1179/sre.1989.30.233.125)
    /// (See also [Transverse Mercator: Bowring series](https://en.wikipedia.org/wiki/Transverse_Mercator:_Bowring_series)).
    Bow89,

    /// S.J. Claessens (2019): *Efficient transformation from Cartesian to geodetic coordinates*.
    /// Computers and Geosciences, Vol. 133, article 104307
    /// [DOI](https://doi.org/10.1016/j.cageo.2019.104307)
    Cla19,

    /// Toshio Fukushima (1999): *Fast transform from geocentric to geodetic coordinates*.
    /// Journal of Geodesy, 73(11), pp.603–610
    /// [DOI](https://doi.org/10.1007/s001900050271)
    Fuk99,

    /// Toshio Fukushima (2006): *Transformation from Cartesian to Geodetic Coordinates Accelerated by Halley’s Method*.
    /// Journal of Geodesy, 79(12), pp.689-693
    /// [DOI](https://doi.org/10.1007/s00190-006-0023-2)
    Fuk06,

    /// Charles F.F. Karney (2010): *Transverse Mercator with an accuracy of a few nanometers*.
    /// [pdf](https://arxiv.org/pdf/1002.1417.pdf)
    Kar10,

    /// Charles F.F. Karney (2011): *Transverse Mercator with an accuracy of a few nanometers*.
    /// J. Geodesy. 85(8): 475–485.
    /// [DOI](https://doi.org/10.1007/s00190-011-0445-3).
    Kar11,

    /// Charles F.F. Karney (2012) *Algorithms for geodesics*.
    /// [pdf](https://arxiv.org/pdf/1109.4448.pdf)
    Kar12,

    /// Charles F.F. Karney (2013) *Algorithms for geodesics*. Journal of Geodesy 87, 43–55.
    /// [DOI](https://doi.org/10.1007/s00190-012-0578-z)
    Kar13,

    /// R.E. Deakin, M.N. Hunter and C.F.F. Karney (2012):
    /// *A fresh look at the UTM projection:
    /// Karney-Krueger equations*.
    /// Surveying and Spatial Sciences Institute (SSSI)
    /// Land Surveying Commission National Conference,
    /// Melbourne, 18-21 April, 2012.
    Dea12,

    /// L. Krüger (1912). *Konforme Abbildung des Erdellipsoids in der Ebene*.
    /// Royal Prussian Geodetic Institute, New Series 52.
    /// [DOI](https://dx.doi.org/10.2312/GFZ.b103-krueger28).
    Kru12,

    /// T. Vincenty (1975) *Direct and Inverse Solutions of Geodesics on the Ellipsoid
    /// with application of nested equations*.
    /// Survey Review, 23(176): 88-93.
    /// [pdf](https://www.ngs.noaa.gov/PUBS_LIB/inverse.pdf)
    /// (See also Wikipedia: [Vincenty's formulae](https://en.wikipedia.org/wiki/Vincenty's_formulae)).
    Vin75,

    /// T. Vincenty (1976). *Correspondence*. Survey Review. 23(180): 294.
    Vin76,
}
