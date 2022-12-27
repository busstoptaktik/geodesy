/// Evaluate Σ cᵢ · xⁱ using Horner's scheme
pub fn horner(arg: f64, coefficients: &[f64]) -> f64 {
    if coefficients.len() < 1 {
        return 0.;
    }
    let mut coefficients = coefficients.iter().rev();
    let mut value = *(coefficients.next().unwrap());
    for c in coefficients {
        value = value.mul_add(arg, *c);
    }
    value
}


/// Evaluate Σ cᵢ sin( i · arg ), for i ∈ {order, ... , 1}, using Clenshaw summation
pub fn clenshaw_sin(arg: f64, coefficients: &[f64]) -> f64 {
    let (sin_arg, cos_arg) = arg.sin_cos();
    let x = 2.0 * cos_arg;
    let mut c0 = 0.0;
    let mut c1 = 0.0;

    for c in coefficients.iter().rev() {
        (c1, c0) = (c0, x.mul_add(c0, c - c1));
    }

    sin_arg * c0
}


// Evaluate Σ cᵢ cos( i · arg ), for i ∈ {order, ... , 1}, using Clenshaw summation
pub fn clenshaw_cos(arg: f64, coefficients: &[f64]) -> f64 {
    let cos_arg = arg.cos();
    let x = 2.0 * cos_arg;
    let mut c0 = 0.0;
    let mut c1 = 0.0;

    for c in coefficients.iter().rev() {
        (c1, c0) = (c0, x.mul_add(c0, c - c1));
    }

    cos_arg * c0 - c1
}

/// The Gudermannian function (often written as gd), is the work horse for computations involving
/// the isometric latitude (i.e. the vertical coordinate of the Mercator projection)
pub fn gudermannian(arg: f64) -> f64 {
    arg.sinh().atan()
}

pub fn inverse_gudermannian(arg: f64) -> f64 {
    arg.tan().asinh()
}


// ----- A N C I L L A R Y   F U N C T I O N S -----------------------------------------

// pj_tsfn is the equivalent of Charles Karney's PROJ function of the
// same name, which determines the function ts(phi) as defined in
// Snyder (1987), Eq. (7-10)
//
// ts is the exponential of the negated isometric latitude, i.e.
// exp(-𝜓), but evaluated in a numerically more stable way than
// the naive ellps.isometric_latitude(...).exp()
//
// This version is essentially identical to Charles Karney's PROJ
// version, including the majority of the comments.
//
// Inputs:
//   (sin phi, cos phi) = trigs of geographic latitude
//   e = eccentricity of the ellipsoid
// Output:
//   ts = exp(-psi) where psi is the isometric latitude (dimensionless)
//      = 1 / (tan(chi) + sec(chi))
// Here isometric latitude is defined by
//   psi = log( tan(pi/4 + phi/2) *
//              ( (1 - e*sin(phi)) / (1 + e*sin(phi)) )^(e/2) )
//       = asinh(tan(phi)) - e * atanh(e * sin(phi))
//       = asinh(tan(chi))
//   chi = conformal latitude
pub(crate) fn pj_tsfn(sincos: (f64, f64), e: f64) -> f64 {
    // exp(-asinh(tan(phi)))
    //    = 1 / (tan(phi) + sec(phi))
    //    = cos(phi) / (1 + sin(phi))  good for phi > 0
    //    = (1 - sin(phi)) / cos(phi)  good for phi < 0
    let factor = if sincos.0 > 0. {
        sincos.1 / (1. + sincos.0)
    } else {
        (1. - sincos.0) / sincos.1
    };
    (e * (e * sincos.0).atanh()).exp() * factor
}

// Snyder (1982) eq. 12-15, PROJ's pj_msfn()
pub(crate) fn pj_msfn(sincos: (f64, f64), es: f64) -> f64 {
    sincos.1 / (1. - sincos.0 * sincos.0 * es).sqrt()
}

// Equivalent to the PROJ pj_phi2 function
pub(crate) fn pj_phi2(ts0: f64, e: f64) -> f64 {
    sinhpsi_to_tanphi((1. / ts0 - ts0) / 2., e).atan()
}

// Ancillary function for computing the inverse isometric latitude. Follows
// [Karney, 2011](crate::Bibliography::Kar11), and the PROJ implementation
// in proj/src/phi2.cpp.
// Needs crate-visibility as it is also used in crate::ellipsoid::latitudes
pub(crate) fn sinhpsi_to_tanphi(taup: f64, e: f64) -> f64 {
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


// ----- Tests ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Ellipsoid, Error};
    #[test]
    fn test_horner() -> Result<(), Error> {
        // Coefficients for 3x² + 2x + 1
        let coefficients = [1_f64, 2., 3.];
        assert_eq!(horner(1., &coefficients), 6.);
        assert_eq!(horner(2., &coefficients), 17.);
        assert_eq!(horner(-2., &coefficients), 9.);

        assert_eq!(horner(-2., &[1_f64]), 1.);
        assert_eq!(horner(-2., &[3_f64]), 3.);

        assert_eq!(horner(-2., &[]), 0.);

        // The normalized meridian arc unit
        let e = Ellipsoid::named("GRS80")?;
        let n = e.third_flattening();
        let nn = n*n;
        let d = [1., 1./4.,  1./64.,  1./256.,  25./16384.];
        let result = horner(nn, &d) / (1. + n);
        let expected = 0.9983242984230415;
        assert!((result - expected).abs() < 1e-14);

        Ok(())
    }

    #[test]
    fn test_clenshaw() -> Result<(), Error> {
        // Coefficients for 1sin(x) + 2sin(2x) + 3sin(3x)
        let coefficients = [1_f64, 2., 3.];
        assert_eq!(clenshaw_sin(0., &[]), 0.);
        assert_eq!(clenshaw_sin(1., &[]), 0.);
        assert_eq!(clenshaw_sin(0.5, &[]), 0.);

        let x = 30_f64.to_radians();

        let result = 1.0*x.sin() + 2.0*(2.0*x).sin() + 3.0*(3.0*x).sin();
        assert!((clenshaw_sin(x, &coefficients) - result).abs() < 1e-14);

        let result = 1.0*x.cos() + 2.0*(2.0*x).cos() + 3.0*(3.0*x).cos();
        assert!((clenshaw_cos(x, &coefficients) - result).abs() < 1e-14);
        Ok(())
    }
}
