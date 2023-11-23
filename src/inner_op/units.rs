use std::collections::HashMap;

/// Units are taken from PROJ https://github.com/OSGeo/PROJ/blob/master/src/units.c,

pub struct Unit(&'static str, &'static str, &'static str, f64);
impl Unit {
    pub fn name(&self) -> &'static str {
        self.0
    }
    pub fn _factor(&self) -> &'static str {
        self.1
    }
    pub fn _description(&self) -> &'static str {
        self.2
    }
    pub fn multiplier(&self) -> f64 {
        self.3
    }
}

/// Represents a set of linear units and their conversion to meters.
#[rustfmt::skip]
pub const LINEAR_UNITS: [Unit; 21] = [
    Unit("km",      "1000",              "Kilometer",                    1000.0),
    Unit("m",       "1",                 "Meter",                        1.0),
    Unit("dm",      "1/10",              "Decimeter",                    0.1),
    Unit("cm",      "1/100",             "Centimeter",                   0.01),
    Unit("mm",      "1/1000",            "Millimeter",                   0.001),
    Unit("kmi",     "1852",              "International Nautical Mile",  1852.0),
    Unit("in",      "0.0254",            "International Inch",           0.0254),
    Unit("ft",      "0.3048",            "International Foot",           0.3048),
    Unit("yd",      "0.9144",            "International Yard",           0.9144),
    Unit("kmi",     "1609.344",          "International Statute Mile",   1609.344),
    Unit("fath",    "1.8288",            "International Fathom",         1.8288),
    Unit("ch",      "20.1168",           "International Chain",          20.1168),
    Unit("link",    "0.201168",          "International Link",           0.201168),
    Unit("us-in",   "1/39.37",           "U.S. Surveyor's Inch",         100.0 / 3937.0),
    Unit("us-ft",   "0.304800609601219", "U.S. Surveyor's Foot",         1200.0 / 3937.0, ),
    Unit("us-yd",   "0.914401828803658", "U.S. Surveyor's Yard",         3600.0 / 3937.0, ),
    Unit("us-ch",   "20.11684023368047", "U.S. Surveyor's Chain",        79200.0 / 3937.0,   ),
    Unit("us-mi",   "1609.347218694437", "U.S. Surveyor's Statute Mile", 6336000.0 / 3937.0, ),
    Unit("ind-yd",  "0.91439523",        "Indian Yard",                  0.91439523),
    Unit("ind-ft",  "0.30479841",        "Indian Foot",                  0.30479841),
    Unit("ind-ch",  "20.11669506",       "Indian Chain",                 20.11669506),
];

const GRAD_TO_RAD: f64 = 0.015707963267948967;
const DEG_TO_RAD: f64 = 0.017453292519943296;

// Angular units and there conversion to radians
#[rustfmt::skip]
pub const ANGULAR_UNITS: [Unit; 3] = [
    Unit("rad",     "1.0",                  "Radian",   1.0),
    Unit("deg",     "0.017453292519943296", "Degree",   DEG_TO_RAD),
    Unit("grad",    "0.015707963267948967", "Grad",     GRAD_TO_RAD),
];

/// Returns a map of linear units and their conversion to meters.
pub fn linear_units_map() -> HashMap<&'static str, &'static Unit> {
    LINEAR_UNITS
        .iter()
        .map(|&ref unit| (unit.name(), unit))
        .collect()
}

/// Returns a map of angular units and their conversion to radians.
pub fn angular_units_map() -> HashMap<&'static str, &'static Unit> {
    ANGULAR_UNITS
        .iter()
        .map(|&ref unit| (unit.name(), unit))
        .collect()
}
