/// The Gudermannian function (often written as gd), is the work horse for computations involving
/// the isometric latitude (i.e. the vertical coordinate of the Mercator projection)
pub mod gudermannian {
    pub fn fwd(arg: f64) -> f64 {
        arg.sinh().atan()
    }

    pub fn inv(arg: f64) -> f64 {
        arg.tan().asinh()
    }
}

/// ts is the equivalent of Charles Karney's PROJ function `pj_tsfn`.
/// It determines the function ts(phi) as defined in Snyder (1987),
/// Eq. (7-10)
///
/// ts is the exponential of the negated isometric latitude, i.e.
/// exp(-𝜓), but evaluated in a numerically more stable way than
/// the naive ellps.isometric_latitude(...).exp()
///
/// This version is essentially identical to Charles Karney's PROJ
/// version, including the majority of the comments.
///
/// Inputs:
///   (sin 𝜙, cos 𝜙): trigs of geographic latitude
///   e: eccentricity of the ellipsoid
/// Output:
///   ts: exp(-𝜓)  =  1 / (tan 𝜒 + sec 𝜒)
///   where 𝜓 is the isometric latitude (dimensionless)
///   and 𝜒 is the conformal latitude (radians)
///
/// Here the isometric latitude is defined by
///   𝜓 = log(
///           tan(𝜋/4 + 𝜙/2) *
///           ( (1 - e × sin 𝜙) / (1 + e × sin 𝜙) ) ^ (e/2)
///       )
///     = asinh(tan 𝜙) - e × atanh(e × sin 𝜙)
///     = asinh(tan 𝜒)
///
/// where 𝜒 is the conformal latitude
///
pub fn ts(sincos: (f64, f64), e: f64) -> f64 {
    // exp(-asinh(tan 𝜙))
    //    = 1 / (tan 𝜙 + sec 𝜙)
    //    = cos 𝜙 / (1 + sin 𝜙)  good for 𝜙 > 0
    //    = (1 - sin 𝜙) / cos 𝜙  good for 𝜙 < 0
    let factor = if sincos.0 > 0. {
        sincos.1 / (1. + sincos.0)
    } else {
        (1. - sincos.0) / sincos.1
    };
    (e * (e * sincos.0).atanh()).exp() * factor
}

/// Snyder (1982) eq. 12-15, PROJ's pj_msfn()
pub fn pj_msfn(sincos: (f64, f64), es: f64) -> f64 {
    sincos.1 / (1. - sincos.0 * sincos.0 * es).sqrt()
}

/// Equivalent to the PROJ pj_phi2 function
pub fn pj_phi2(ts0: f64, e: f64) -> f64 {
    sinhpsi_to_tanphi((1. / ts0 - ts0) / 2., e).atan()
}

/// Snyder (1982) eq. ??, PROJ's pj_qsfn()
pub fn qs(sinphi: f64, e: f64) -> f64 {
    let es = e * e;
    let one_es = 1.0 - es;

    if e < 1e-7 {
        return 2.0 * sinphi;
    }

    let con = e * sinphi;
    let div1 = 1.0 - con * con;
    let div2 = 1.0 + con;

    one_es * (sinphi / div1 - (0.5 / e) * ((1. - con) / div2).ln())
}

/// Ancillary function for computing the inverse isometric latitude. Follows
/// [Karney, 2011](crate::Bibliography::Kar11), and the PROJ implementation
/// in proj/src/phi2.cpp.
pub fn sinhpsi_to_tanphi(taup: f64, e: f64) -> f64 {
    // min iterations = 1, max iterations = 2; mean = 1.954
    const MAX_ITER: usize = 5;

    // rooteps, tol and tmax are compile time constants, but currently
    // Rust cannot const-evaluate powers and roots, so we must either
    // evaluate these "constants" as lazy_statics, or just swallow the
    // penalty of an extra sqrt and two divisions on each call.
    // If this shows unbearable, we can just also assume IEEE-64 bit
    // arithmetic, and set rooteps = 0.000000014901161193847656
    let rooteps: f64 = f64::EPSILON.sqrt();
    let tol: f64 = rooteps / 10.; // the criterion for Newton's method
    let tmax: f64 = 2. / rooteps; // threshold for large arg limit exact

    let e2m = 1. - e * e;
    let stol = tol * taup.abs().max(1.0);

    // The initial guess.  70 corresponds to chi = 89.18 deg
    let mut tau = if taup.abs() > 70. {
        taup * (e * e.atanh()).exp()
    } else {
        taup / e2m
    };

    // Handle +/-inf, nan, and e = 1
    if (tau.abs() >= tmax) || tau.is_nan() {
        return tau;
    }

    for _ in 0..MAX_ITER {
        let tau1 = (1. + tau * tau).sqrt();
        let sig = (e * (e * tau / tau1).atanh()).sinh();
        let taupa = (1. + sig * sig).sqrt() * tau - sig * tau1;
        let dtau =
            (taup - taupa) * (1. + e2m * (tau * tau)) / (e2m * tau1 * (1. + taupa * taupa).sqrt());
        tau += dtau;

        if (dtau.abs() < stol) || tau.is_nan() {
            return tau;
        }
    }
    f64::NAN
}
