// A HashMap would have been a better choice,for the OPERATOR_LIST, except
// for the annoying fact that it cannot be compile-time constructed
#[rustfmt::skip]
pub(super) const ELLIPSOID_LIST: [(&str, &str, &str, &str, &str); 47] = [
    ("MERIT",     "6378137",       "6378137",      "298.257",            "MERIT 1983"),
    ("SGS85",     "6378136",       "6378136",      "298.257",            "Soviet Geodetic System 85"),
    ("GRS80",     "6378137",       "6378137",      "298.2572221008827",  "GRS 1980(IUGG, 1980)"),
    ("IAU76",     "6378140",       "6378140",      "298.257",            "IAU 1976"),
    ("airy",      "6377563.396",   "6377563.396",  "299.3249646",        "Airy 1830"),
    ("APL4.9",    "6378137",       "6378137.0",    "298.25",             "Appl. Physics. 1965"),
    ("NWL9D",     "6378145",       "6378145.0",    "298.25",             "Naval Weapons Lab., 1965"),
    ("mod_airy",  "6377340.189",   "6377340.189",  "299.3249373654824",  "Modified Airy"),
    ("andrae",    "6377104.43 ",   "6377104.43",   "300.0",              "Andrae 1876 (Denmark, Iceland)"),
    ("danish",    "6377019.2563",  "6377019.2563", "300.0",              "Andrae 1876 (Denmark, Iceland)"),
    ("aust_SA",   "6378160",       "6378160",      "298.25",             "Australian Natl & S. Amer. 1969"),
    ("GRS67",     "6378160",       "6378160",      "298.2471674270",     "GRS 67(IUGG 1967)"),
    ("GSK2011",   "6378136.5",     "6378136.5",    "298.2564151",        "GSK-2011"),
    ("bessel",    "6377397.155",   "6377397.155",  "299.1528128",        "Bessel 1841"),
    ("bess_nam",  "6377483.865",   "6377483.865",  "299.1528128",        "Bessel 1841 (Namibia)"),
    ("clrk66",    "6378206.4",     "6378206.4",    "294.9786982138982",  "Clarke 1866"),
    ("clrk80",    "6378249.145",   "6378249.145",  "293.4663",           "Clarke 1880 mod."),
    ("clrk80ign", "6378249.2",     "6378249.2",    "293.4660212936269",  "Clarke 1880 (IGN)"),
    ("CPM",       "6375738.7",     "6375738.7",    "334.29",             "Comm. des Poids et Mesures 1799"),
    ("delmbr",    "6376428",       "6376428",      "311.5",              "Delambre 1810 (Belgium)"),
    ("engelis",   "6378136.05",    "6378136.05",   "298.2566",           "Engelis 1985"),
    ("evrst30",   "6377276.345",   "6377276.345",  "300.8017",           "Everest 1830"),
    ("evrst48",   "6377304.063",   "6377304.063",  "300.8017",           "Everest 1948"),
    ("evrst56",   "6377301.243",   "6377301.243",  "300.8017",           "Everest 1956"),
    ("evrst69",   "6377295.664",   "6377295.664",  "300.8017",           "Everest 1969"),
    ("evrstSS",   "6377298.556",   "6377298.556",  "300.8017",           "Everest (Sabah & Sarawak)"),
    ("fschr60",   "6378166",       "6378166",      "298.3",              "Fischer (Mercury Datum) 1960"),
    ("fschr60m",  "6378155",       "6378155",      "298.3",              "Modified Fischer 1960"),
    ("fschr68",   "6378150",       "6378150",      "298.3",              "Fischer 1968"),
    ("helmert",   "6378200",       "6378200",      "298.3",              "Helmert 1906"),
    ("hough",     "6378270",       "6378270",      "297.",               "Hough"),
    ("intl",      "6378388",       "6378388",      "297.",               "International 1909 (Hayford)"),
    ("krass",     "6378245",       "6378245",      "298.3",              "Krassovsky, 1942"),
    ("kaula",     "6378163",       "6378163",      "298.24",             "Kaula 1961"),
    ("lerch",     "6378139",       "6378139",      "298.257",            "Lerch 1979"),
    ("mprts",     "6397300",       "6397300",      "191.",               "Maupertius 1738"),
    ("new_intl",  "6378157.5",     "6378157.5",    "298.2496153900135",  "New International 1967"),
    ("plessis",   "6376523",       "6376523.",     "308.64099709583735", "Plessis 1817 (France)"),
    ("PZ90",      "6378136",       "6378136",      "298.25784",          "PZ-90"),
    ("SEasia",    "6378155",       "6378155",      "298.3000002408657",  "Southeast Asia"),
    ("walbeck",   "6376896",       "6376896",      "302.78000018165636", "Walbeck"),
    ("WGS60",     "6378165",       "6378165",      "298.3",              "WGS 60"),
    ("WGS66",     "6378145",       "6378145",      "298.25",             "WGS 66"),
    ("WGS72",     "6378135",       "6378135",      "298.26",             "WGS 72"),
    ("WGS84",     "6378137",       "6378137",      "298.257223563",      "WGS 84"),
    ("sphere",    "6370997",       "6370997",      "0.",                 "Normal Sphere (r=6370997)"),
    ("unitsphere",      "1",             "1",      "0.",                 "Unit Sphere (r=1)"),
];


// Coefficients to convert 𝜙 to 𝜇, Eq. A5 in [Karney (2022)](crate::bibliography::Kar22)
#[rustfmt::skip]
pub(super) const GEODETIC_TO_RECTIFYING_LATITUDE_COEFFICIENTS: [f64; 12] = [
    -3.0/2., 9.0/16., -3.0/32.,
    15.0/16., -15.0/32., 135.0/2048.,
    -35.0/48., 105.0/256.,
    315.0/512., -189.0/512.,
    -693.0/1280.,
    1001.0/2048.
];

// Coefficients to convert 𝜇 to 𝜙, Eq. A6 in [Karney (2022)](crate::bibliography::Kar22)
// with 0 terms dropped
#[rustfmt::skip]
pub(super) const RECTIFYING_TO_GEODETIC_LATITUDE_COEFFICIENTS: [f64; 12] = [
    3.0/2.,   -27.0/32.,  269.0/512.,
    21.0/16.,  -55.0/32., 6759.0/4096.,
    151.0/96., -417.0/128.,
    1097.0/512., -15543.0/2560.,
    8011.0/2560.,
    293393.0/61440.,
];

/// Coefficients for expansion of the normalized meridian arc unit in terms
/// of *n²*, the square of the third flattening.
/// See [Karney 2010](crate::Bibliography::Kar10) eq. (29)
pub(super) const MERIDIAN_ARC_COEFFICIENTS: [f64; 5] = [1., 1./4.,  1./64.,  1./256.,  25./16384.];
