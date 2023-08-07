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

/// The Jacobian matrix for investigation of the geometrical properties of
/// map projections. Can be converted to the more easily digestible
/// [`Factors`] struct, using the `.factors()` method.
#[derive(Debug, Default)]
#[rustfmt::skip]
pub struct Jacobian {
    // The geographical coordinates (in degrees) of the evaluation point
    pub latitude: f64,
    pub longitude: f64,

    // The derivatives at the evaluation point of easting (x) and northing (y)
    // with respect to longitude (lam) and latitude (phi)
    pub dx_dlam: f64,
    pub dy_dlam: f64,
    pub dx_dphi: f64,
    pub dy_dphi: f64,

    // The ellipsoid on which the jacobian is evaluated
    pub ellps: Ellipsoid
}

/// Geometrical properties of map projections. The `Factors` struct is
/// derived from the more fundamental `Jacobian`, using its `.factors()`
/// method
#[derive(Debug, Default)]
#[rustfmt::skip]
pub struct Factors {
    // Scalar factors                // Common textbook designation
    pub meridional_scale: f64,           // h
    pub parallel_scale: f64,             // k
    pub areal_scale: f64,                // s

    // Angular factors
    pub angular_distortion: f64,         // ω
    pub meridian_parallel_angle: f64,    // θ'
    pub meridian_convergence: f64,       // α

    // Tissot indicatrix
    pub tissot_semimajor: f64,           // a
    pub tissot_semiminor: f64,           // b
}

impl Jacobian {
    /// Compute the Jacobian matrix for the map projection represented by `op`.
    /// The `scale` parameters define the scaling from input units to degrees,
    /// and from output units to metres. Hence, if input is in radians and
    /// output in feet, `scale`should be set to `[1f64.to_degrees(), 0.3048]`.
    /// The `swap` parameters indicate whether input and output is swapped
    /// with respect to the Rust Geodesy internal GIS convention of "longitude
    /// before latitude, and easting before northing". Hence, if input is in
    /// the geographical convention of latitude/longitude, and output is in
    /// easting/northing, `swap` should be set to `[true, false]`. The `ellps`
    /// parameter should be set to the relevant ellipsoid for the projection.
    /// While it could be derived directly from the other input data in the
    /// case of a single operation, the potential case of a pipeline operation
    /// makes it necessary to actively select which one to use. In all but the
    /// most demanding situations it is, however, probably fine to just select
    /// `Ellipsoid::default()`, i.e. GRS80.
    ///
    /// Mostly based on the PROJ function [pj_deriv](https://github.com/OSGeo/PROJ/blob/master/src/deriv.cpp),
    #[allow(dead_code)]
    #[rustfmt::skip]
    fn new(ctx: &impl Context, op: OpHandle, scale: [f64; 2], swap: [bool; 2], ellps: Ellipsoid, at: Coor2D) -> Result<Jacobian, Error> {

        // If we have input in degrees, we must multiply the output by a factor of 180/pi
        // For user convenience, scale[0] is a "to degrees"-factor, i.e. scale[0]==1
        // indicates degrees, whereas scale[0]==180/pi indicates that input angles are
        // in radians.
        // To convert input coordinates to radians, divide by `angular_scale`
        let angular_scale = 1f64.to_degrees() / scale[0];

        // If we have output in feet, we must multiply the output by a factor of 0.3048
        // For user convenience, scale[1] is a "to metres"-factor, i.e. scale[1]==1
        // indicates metres, whereas scale[1]==0.3048 indicates that output lengths
        // are in feet, and scale[1]=201.168 indicates that output is in furlongs
        let linear_scale = scale[1];

        let h = 1e-5 * angular_scale;
        let d = (4.0 * h * ellps.semimajor_axis()).recip() * linear_scale * angular_scale;

        let mut coo = [Coor2D::origin(); 4];

        let (e, n) = if swap[0] {(at[1], at[0])} else {(at[0], at[1])};

        // Latitude and longitude in degrees for the return value
        let latitude = n * scale[0];
        let longitude = e * scale[0];

        // North-east of POI
        coo[0] = Coor2D::raw(e + h, n + h);
        // South-east of POI
        coo[1] = Coor2D::raw(e + h, n - h);
        // South-west of POI
        coo[2] = Coor2D::raw(e - h, n - h);
        // North-west of POI
        coo[3] = Coor2D::raw(e - h, n + h);
        if swap[0] {
            coo[0] = Coor2D::raw(coo[0][1], coo[0][0]);
            coo[1] = Coor2D::raw(coo[1][1], coo[1][0]);
            coo[2] = Coor2D::raw(coo[2][1], coo[2][0]);
            coo[3] = Coor2D::raw(coo[3][1], coo[3][0]);
        }
        ctx.apply(op, Fwd, &mut coo)?;

        // Handle output swapping
        let (e, n) = if swap[1] {(1, 0)} else {(0, 1)};

        //        NE          SE         SW          NW
        let dx_dlam =  (coo[0][e] + coo[1][e] - coo[2][e] - coo[3][e]) * d;
        let dy_dlam =  (coo[0][n] + coo[1][n] - coo[2][n] - coo[3][n]) * d;
        let dx_dphi =  (coo[0][e] - coo[1][e] - coo[2][e] + coo[3][e]) * d;
        let dy_dphi =  (coo[0][n] - coo[1][n] - coo[2][n] + coo[3][n]) * d;

        Ok(Jacobian{latitude, longitude, dx_dlam, dy_dlam, dx_dphi, dy_dphi, ellps})
    }

    /// This closely follows the PROJ function pj_factors() and its friendly wrapper
    /// proj_factors(), i.e. closely following Snyder's magnum opus
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
