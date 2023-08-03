use crate::context_authoring::*;
pub mod minimal;
pub use minimal::Minimal;

#[cfg(feature = "with_plain")]
pub mod plain;
#[cfg(feature = "with_plain")]
pub use plain::Plain;

// ----- T H E   C O N T E X T   T R A I T ---------------------------------------------

/// The `Context` trait defines the mode of communication between *Rust Geodesy* internals
/// and the external context (i.e. typically resources like grids, transformation definitions,
/// or ellipsoid parameters).
pub trait Context {
    /// In general, implementations should make sure that `new` differs from `default`
    /// only by adding access to the builtin adaptors (`geo:in`, `gis:out` etc.)
    fn new() -> Self
    where
        Self: Sized;

    /// Instantiate the operation given by `definition`
    fn op(&mut self, definition: &str) -> Result<OpHandle, Error>;

    /// Apply operation `op` to `operands`
    fn apply(
        &self,
        op: OpHandle,
        direction: Direction,
        operands: &mut dyn CoordinateSet,
    ) -> Result<usize, Error>;

    /// Globally defined default values (typically just `ellps=GRS80`)
    fn globals(&self) -> BTreeMap<String, String>;

    /// Definitions of steps
    fn steps(&self, op: OpHandle) -> Result<&Vec<String>, Error>;

    /// Parsed parameters of a specific step
    fn params(&self, op: OpHandle, index: usize) -> Result<ParsedParameters, Error>;

    /// Register a new user-defined operator
    fn register_op(&mut self, name: &str, constructor: OpConstructor);
    /// Register a new user-defined resource (macro, ellipsoid parameter set...)
    fn register_resource(&mut self, name: &str, definition: &str);

    /// Helper for the `Op` instantiation logic in `Op::op(...)`
    fn get_op(&self, name: &str) -> Result<OpConstructor, Error>;
    /// Helper for the `Op` instantiation logic in `Op::op(...)`
    fn get_resource(&self, name: &str) -> Result<String, Error>;

    /// Access `blob`-like resources by identifier
    fn get_blob(&self, name: &str) -> Result<Vec<u8>, Error>;

    /// Access grid resources by identifier
    fn get_grid(&self, name: &str) -> Result<Grid, Error>;
}






#[derive(Debug, Default)]
#[rustfmt::skip]
pub struct Jacobian {
    // The latitude (in degrees) of the evaluation point
    latitude: f64,

    // The derivatives at the evaluation point of easting (x) and northing (y)
    // with respect to longitude (lam) and latitude (phi)
    dx_dlam: f64,
    dy_dlam: f64,
    dx_dphi: f64,
    dy_dphi: f64,

    // The ellipsoid on which the jacobian is evaluated
    ellps: Ellipsoid
}

#[derive(Debug, Default)]
#[rustfmt::skip]
pub struct Factors {
    // Scalar factors                // Common textbook designation
    meridional_scale: f64,           // h
    parallel_scale: f64,             // k
    areal_scale: f64,                // s

    // Angular factors
    angular_distortion: f64,         // ω
    meridian_parallel_angle: f64,    // θ'
    meridian_convergence: f64,       // α

    // Tissot indicatrix
    tissot_semimajor: f64,           // a
    tissot_semiminor: f64,           // b
}

impl Jacobian {
    // This closely follows the PROJ function pj_factors() and its friendly wrapper
    // proj_factors(), i.e. closely following Snyder's magnum opus
    pub fn factors(&self) -> Factors {
        let mut f = Factors::default();
        let x_l = self.dx_dlam;
        let y_l = self.dy_dlam;
        let x_p = self.dx_dphi;
        let y_p = self.dy_dphi;

        let (s, c) = self.latitude.to_radians().sin_cos();
        let es = self.ellps.eccentricity_squared();

        // Linear scaling factors
        let h = x_p.hypot(y_p);
        let k = x_l.hypot(y_l) / c;

        // Correction of linear scaling factors for ellipsoidal geometry
        let t = 1. - es * s * s;
        let n = t.sqrt();
        let h = h * (t * n / (1. - es));
        let k = k * n;
        let r = t * t / (1. - es);

        f.meridional_scale = h;
        f.parallel_scale = k;
        f.areal_scale = (y_p * x_l - x_p * y_l) * r / c;

        // Tissot axes
        let t = h * h + k * k;
        let a = (t + 2. * f.areal_scale).sqrt();
        let t = (t - 2. * f.areal_scale).clamp(0., f64::MAX).sqrt();
        f.tissot_semiminor = 0.5 * (a - t);
        f.tissot_semimajor = 0.5 * (a + t);

        // Angular elements
        f.meridian_parallel_angle = (f.areal_scale / (h * k)).clamp(-1., 1.).asin().to_degrees();
        f.meridian_convergence = -x_p.atan2(y_p).to_degrees();
        let a = f.tissot_semimajor;
        let b = f.tissot_semiminor;
        f.angular_distortion = 2. * ((a - b) / (a + b)).asin();
        f
    }
}



// Help context providers provide canonically named, built in coordinate adaptors
#[rustfmt::skip]
pub const BUILTIN_ADAPTORS: [(&str, &str); 8] = [
    ("geo:in",  "adapt from=neuf_deg"),
    ("geo:out", "adapt to=neuf_deg"  ),
    ("gis:in",  "adapt from=enuf_deg"),
    ("gis:out", "adapt to=enuf_deg"  ),
    ("neu:in",  "adapt from=neuf"    ),
    ("neu:out", "adapt to=neuf"      ),
    ("enu:in",  "adapt from=enuf"    ),
    ("enu:out", "adapt to=enuf"      ),
];
