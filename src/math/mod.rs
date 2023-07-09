use log::warn;

/// The order of the Fourier series used to compute e.g. auxiliary latitudes
pub const POLYNOMIAL_ORDER: usize = 6;

/// Two upper triangular matrices of polynomium coefficients for computing
/// the Fourier coefficients for (a.o.) the auxiliary latitudes
#[derive(Clone, Copy, Debug, Default)]
pub struct PolynomialCoefficients {
    pub fwd: [[f64; POLYNOMIAL_ORDER]; POLYNOMIAL_ORDER],
    pub inv: [[f64; POLYNOMIAL_ORDER]; POLYNOMIAL_ORDER],
}

/// The Fourier coefficients used when computing e.g. auxiliary latitudes
#[derive(Clone, Copy, Debug, Default)]
pub struct FourierCoefficients {
    pub fwd: [f64; POLYNOMIAL_ORDER],
    pub inv: [f64; POLYNOMIAL_ORDER],
    pub etc: [f64; 2],
}

// --- Taylor series polynomium evaluation ----

/// Compute Fourier coefficients by evaluating their corresponding
/// Taylor polynomiums
pub fn fourier_coefficients(
    arg: f64,
    coefficients: &PolynomialCoefficients,
) -> FourierCoefficients {
    let mut result = FourierCoefficients::default();
    for i in 0..POLYNOMIAL_ORDER {
        result.fwd[i] = arg * horner(arg, &coefficients.fwd[i]);
        result.inv[i] = arg * horner(arg, &coefficients.inv[i]);
    }
    result
}

/// Evaluate Σ cᵢ · xⁱ using Horner's scheme
pub fn horner(arg: f64, coefficients: &[f64]) -> f64 {
    if coefficients.is_empty() {
        return 0.;
    }
    let mut coefficients = coefficients.iter().rev();
    let mut value = *(coefficients.next().unwrap());
    for c in coefficients {
        value = value.mul_add(arg, *c);
    }
    value
}

// --- Fourier series summation using Clenshaw's recurrence ---

pub mod clenshaw {
    /// Evaluate Σ cᵢ sin( i · arg ), for i ∈ {order, ... , 1}, using Clenshaw summation
    pub fn sin(arg: f64, coefficients: &[f64]) -> f64 {
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
    pub fn cos(arg: f64, coefficients: &[f64]) -> f64 {
        let cos_arg = arg.cos();
        let x = 2.0 * cos_arg;
        let mut c0 = 0.0;
        let mut c1 = 0.0;

        for c in coefficients.iter().rev() {
            (c1, c0) = (c0, x.mul_add(c0, c - c1));
        }
        cos_arg * c0 - c1
    }

    /// Evaluate Σ cᵢ Sin( i · arg ), for i ∈ {order, ... , 1}, using Clenshaw summation.
    /// i.e. a series of complex sines with real coefficients
    #[allow(unused_assignments)] // For symmetric initialization of hr2, hi2
    pub fn complex_sin(arg: [f64; 2], coefficients: &[f64]) -> [f64; 2] {
        // Prepare the trigonometric factors
        let (sin_r, cos_r) = arg[0].sin_cos();
        let sinh_i = arg[1].sinh();
        let cosh_i = arg[1].cosh();
        let r = 2. * cos_r * cosh_i;
        let i = -2. * sin_r * sinh_i;
        let mut coefficients = coefficients.iter().rev();

        // Handle zero length series by conventionally assigning them the sum of 0
        let Some(c) = coefficients.next() else {
            return [0.; 2];
        };

        // Initialize the recurrence coefficients
        let (mut hr2, mut hr1, mut hr) = (0., 0., *c);
        let (mut hi2, mut hi1, mut hi) = (0., 0., 0.);

        for c in coefficients {
            // Rotate the recurrence coefficients
            (hr2, hi2, hr1, hi1) = (hr1, hi1, hr, hi);

            // Update the recurrent sum
            hr = -hr2 + r * hr1 - i * hi1 + c;
            hi = -hi2 + i * hr1 + r * hi1;
        }

        // Finalize the sum
        let r = sin_r * cosh_i;
        let i = cos_r * sinh_i;
        [r * hr - i * hi, r * hi + i * hr]
    }

    // --- Clenshaw versions optimized for Transverse Mercator ---

    /// Evaluate Σ cᵢ sin( i · arg ), for i ∈ {order, ... , 1}, using Clenshaw summation
    ///
    /// Functionally identical to [clenshaw_sin](crate::math::clenshaw::sin), but
    /// takes advantage trigonometric factors, which are conveniently computed ahead-of-call in
    /// the Transverse Mercator code, tmerc. Since tmerc is so widely used, this optimization
    /// makes good sense, despite the more clumsy call signature. Also, for the same reason
    /// we assert that, despite that compiler heuristics may beg to differ, this function should
    /// always be inlined.
    #[inline(always)]
    pub fn sin_optimized_for_tmerc(trig: [f64; 2], coefficients: &[f64]) -> f64 {
        // Unpack the trigonometric factors for better readability.
        let (sin_arg, cos_arg) = (trig[0], trig[1]);
        let x = 2.0 * cos_arg;
        let mut c0 = 0.0;
        let mut c1 = 0.0;

        for c in coefficients.iter().rev() {
            (c1, c0) = (c0, x.mul_add(c0, c - c1));
        }
        sin_arg * c0
    }

    /// Evaluate Σ cᵢ Sin( i · arg ), for i ∈ {order, ... , 1}, using Clenshaw summation.
    /// i.e. a series of complex sines with real coefficients.
    ///
    /// Functionally identical to [clenshaw_complex_sin](crate::math::clenshaw::complex_sin), but
    /// takes advantage of some trigonometric and hyperbolic factors, which are conveniently
    /// computed ahead-of-call in the Transverse Mercator code, tmerc. Since tmerc is so widely
    /// used, this optimization makes good sense, despite the more clumsy call signature. Also,
    /// we assert that, despite that compiler heuristics may beg to differ, this function should
    /// always be inlined.
    #[allow(unused_assignments)] // For symmetric initialization of hr2, hi2
    #[inline(always)]
    pub fn complex_sin_optimized_for_tmerc(
        trig: [f64; 2],
        hyp: [f64; 2],
        coefficients: &[f64],
    ) -> [f64; 2] {
        // Unpack the trigonometric and hyperbolic factors for better readability.
        let (sin_r, cos_r) = (trig[0], trig[1]);
        let (sinh_i, cosh_i) = (hyp[0], hyp[1]);
        let r = 2. * cos_r * cosh_i;
        let i = -2. * sin_r * sinh_i;

        // Prepare the iterator for summation in reverse order
        let mut coefficients = coefficients.iter().rev();

        // Handle zero length series by conventionally assigning them the sum of 0
        let Some(c) = coefficients.next() else {
            return [0.; 2];
        };

        // Initialize the recurrence coefficients
        let (mut hr2, mut hr1, mut hr) = (0., 0., *c);
        let (mut hi2, mut hi1, mut hi) = (0., 0., 0.);

        for c in coefficients {
            // Rotate the recurrence coefficients
            (hr2, hi2, hr1, hi1) = (hr1, hi1, hr, hi);

            // Update the recurrent sum
            hr = -hr2 + r * hr1 - i * hi1 + c;
            hi = -hi2 + i * hr1 + r * hi1;
        }

        // Finalize the sum
        let r = sin_r * cosh_i;
        let i = cos_r * sinh_i;
        [r * hr - i * hi, r * hi + i * hr]
    }
}

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

pub mod angular {
    /// Simplistic transformation from degrees, minutes and seconds-with-decimals
    /// to degrees-with-decimals. No sanity check: Sign taken from degree-component,
    /// minutes forced to unsigned by i16 type, but passing a negative value for
    /// seconds leads to undefined behaviour.
    pub fn dms_to_dd(d: i32, m: u16, s: f64) -> f64 {
        d.signum() as f64 * (d.abs() as f64 + (m as f64 + s / 60.) / 60.)
    }

    /// Simplistic transformation from degrees and minutes-with-decimals
    /// to degrees-with-decimals. No sanity check: Sign taken from
    /// degree-component, but passing a negative value for minutes leads
    /// to undefined behaviour.
    pub fn dm_to_dd(d: i32, m: f64) -> f64 {
        d.signum() as f64 * (d.abs() as f64 + (m / 60.))
    }

    /// Simplistic transformation from the ISO-6709 DDDMM.mmm format to
    /// to degrees-with-decimals. No sanity check: Invalid input,
    /// such as 5575.75 (where the number of minutes exceed 60) leads
    /// to undefined behaviour.
    pub fn iso_dm_to_dd(iso_dm: f64) -> f64 {
        let sign = iso_dm.signum();
        let dm = iso_dm.abs() as u32;
        let fraction = iso_dm.abs() - dm as f64;
        let d = dm / 100;
        let m = (dm - d * 100) as f64 + fraction;
        sign * (d as f64 + (m / 60.))
    }

    /// Transformation from degrees-with-decimals to the ISO-6709 DDDMM.mmm format.
    pub fn dd_to_iso_dm(dd: f64) -> f64 {
        let sign = dd.signum();
        let dd = dd.abs();
        let d = dd.floor();
        let m = (dd - d) * 60.;
        sign * (d * 100. + m)
    }

    /// Simplistic transformation from the ISO-6709 DDDMMSS.sss
    /// format to degrees-with-decimals. No sanity check: Invalid input,
    /// such as 557575.75 (where the number of minutes and seconds both
    /// exceed 60) leads to undefined behaviour.
    pub fn iso_dms_to_dd(iso_dms: f64) -> f64 {
        let sign = iso_dms.signum();
        let dms = iso_dms.abs() as u32;
        let fraction = iso_dms.abs() - dms as f64;
        let d = dms / 10000;
        let ms = dms - d * 10000;
        let m = ms / 100;
        let s = (ms - m * 100) as f64 + fraction;
        sign * (d as f64 + ((s / 60.) + m as f64) / 60.)
    }

    /// Transformation from degrees-with-decimals to the extended
    /// ISO-6709 DDDMMSS.sss format.
    pub fn dd_to_iso_dms(dd: f64) -> f64 {
        let sign = dd.signum();
        let dd = dd.abs();
        let d = dd.floor();
        let mm = (dd - d) * 60.;
        let m = mm.floor();
        let s = (mm - m) * 60.;
        sign * (d * 10000. + m * 100. + s)
    }

    /// normalize arbitrary angles to [-π, π):
    pub fn normalize_symmetric(angle: f64) -> f64 {
        use std::f64::consts::PI;
        let angle = (angle + PI) % (2.0 * PI);
        angle - PI * angle.signum()
    }

    /// normalize arbitrary angles to [0, 2π):
    pub fn normalize_positive(angle: f64) -> f64 {
        use std::f64::consts::PI;
        let angle = angle % (2.0 * PI);
        if angle < 0. {
            return angle + 2.0 * PI;
        }
        angle
    }
}

// ----- A N C I L L A R Y   F U N C T I O N S -----------------------------------------

// ts is the equivalent of Charles Karney's PROJ function `pj_tsfn`.
// It determines the function ts(phi) as defined in Snyder (1987),
// Eq. (7-10)
//
// ts is the exponential of the negated isometric latitude, i.e.
// exp(-𝜓), but evaluated in a numerically more stable way than
// the naive ellps.isometric_latitude(...).exp()
//
// This version is essentially identical to Charles Karney's PROJ
// version, including the majority of the comments.
//
// Inputs:
//   (sin 𝜙, cos 𝜙): trigs of geographic latitude
//   e: eccentricity of the ellipsoid
// Output:
//   ts: exp(-𝜓)  =  1 / (tan 𝜒 + sec 𝜒)
//   where 𝜓 is the isometric latitude (dimensionless)
//   and 𝜒 is the conformal latitude (radians)
//
// Here the isometric latitude is defined by
//   𝜓 = log(
//           tan(𝜋/4 + 𝜙/2) *
//           ( (1 - e × sin 𝜙) / (1 + e × sin 𝜙) ) ^ (e/2)
//       )
//     = asinh(tan 𝜙) - e × atanh(e × sin 𝜙)
//     = asinh(tan 𝜒)
//
// where 𝜒 is the conformal latitude
//
pub(crate) fn ts(sincos: (f64, f64), e: f64) -> f64 {
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

// Snyder (1982) eq. 12-15, PROJ's pj_msfn()
pub(crate) fn pj_msfn(sincos: (f64, f64), es: f64) -> f64 {
    sincos.1 / (1. - sincos.0 * sincos.0 * es).sqrt()
}

// Equivalent to the PROJ pj_phi2 function
pub(crate) fn pj_phi2(ts0: f64, e: f64) -> f64 {
    sinhpsi_to_tanphi((1. / ts0 - ts0) / 2., e).atan()
}

// Snyder (1982) eq. ??, PROJ's pj_qsfn()
pub(crate) fn qs(sinphi: f64, e: f64) -> f64 {
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

/// Parse sexagesimal degrees, i.e. degrees, minutes and seconds in the
/// format 45:30:36, 45:30:36N,-45:30:36 etc.
pub fn parse_sexagesimal(angle: &str) -> f64 {
    // Degrees, minutes, and seconds
    let mut dms = [0.0, 0.0, 0.0];
    let mut angle = angle.trim();

    // Empty?
    let n = angle.len();
    if n == 0 || angle == "NaN" {
        return f64::NAN;
    }

    // Handle NSEW indicators
    let mut postfix_sign = 1.0;
    if "wWsSeEnN".contains(&angle[n - 1..]) {
        if "wWsS".contains(&angle[n - 1..]) {
            postfix_sign = -1.0;
        }
        angle = &angle[..n - 1];
    }

    // Split into as many elements as given: D, D:M, D:M:S
    for (i, element) in angle.split(':').enumerate() {
        if i < 3 {
            if let Ok(v) = element.parse::<f64>() {
                dms[i] = v;
                continue;
            }
        }
        // More than 3 elements?
        warn!("Cannot parse {angle} as a real number or sexagesimal angle");
        return f64::NAN;
    }

    // Sexagesimal conversion if we have more than one element. Otherwise
    // decay gracefully to plain real/f64 conversion
    let sign = dms[0].signum() * postfix_sign;
    sign * (dms[0].abs() + (dms[1] + dms[2] / 60.0) / 60.0)
}

// ----- Tests ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Ellipsoid, Error};

    #[test]
    fn test_parse_sexagesimal() -> Result<(), Error> {
        assert_eq!(1.51, parse_sexagesimal("1:30:36"));
        assert_eq!(-1.51, parse_sexagesimal("-1:30:36"));
        assert_eq!(1.51, parse_sexagesimal("1:30:36N"));
        assert_eq!(-1.51, parse_sexagesimal("1:30:36S"));
        assert_eq!(1.51, parse_sexagesimal("1:30:36e"));
        assert_eq!(-1.51, parse_sexagesimal("1:30:36w"));
        assert!(parse_sexagesimal("q1:30:36w").is_nan());

        Ok(())
    }

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
        let nn = n * n;
        let d = [1., 1. / 4., 1. / 64., 1. / 256., 25. / 16384.];
        let result = horner(nn, &d) / (1. + n);
        let expected = 0.9983242984230415;
        assert!((result - expected).abs() < 1e-14);

        Ok(())
    }

    #[test]
    fn test_clenshaw() -> Result<(), Error> {
        // Coefficients for 1sin(x) + 2sin(2x) + 3sin(3x)
        let coefficients = [1., 2., 3.];
        assert_eq!(clenshaw::sin(0., &[]), 0.);
        assert_eq!(clenshaw::sin(1., &[]), 0.);
        assert_eq!(clenshaw::sin(0.5, &[]), 0.);

        let x = 30_f64.to_radians();

        // Clenshaw sine-series summation
        let result = 1.0 * x.sin() + 2.0 * (2.0 * x).sin() + 3.0 * (3.0 * x).sin();
        assert!((clenshaw::sin(x, &coefficients) - result).abs() < 1e-14);

        // Clenshaw cosine-series summation
        let result = 1.0 * x.cos() + 2.0 * (2.0 * x).cos() + 3.0 * (3.0 * x).cos();
        assert!((clenshaw::cos(x, &coefficients) - result).abs() < 1e-14);

        // Clenshaw complex sine summation
        let coefficients = [6., 5., 4., 3., 2., 1.];
        let arg = [30f64.to_radians(), 60f64.to_radians()];
        // Canonical result from Poder/Engsager implementation
        let r = 248.658846388817693;
        let i = -463.436347907636559;
        // Let's see if we can reproduce that...
        let sum = clenshaw::complex_sin(arg, &coefficients);
        assert!((sum[0] - r).abs() < 1e-14);
        assert!((sum[1] - i).abs() < 1e-14);

        // Canonical result for complex cosine clenshaw, from Poder/Engsager implementation
        // let r = -461.338884918028953;
        // let i = -246.855278649982154;

        Ok(())
    }

    #[test]
    fn test_angular() {
        // dms
        assert_eq!(angular::dms_to_dd(55, 30, 36.), 55.51);
        assert_eq!(angular::dm_to_dd(55, 30.60), 55.51);

        // iso_dm + iso_dms
        assert!((angular::iso_dm_to_dd(5530.60) - 55.51).abs() < 1e-10);
        assert!((angular::iso_dm_to_dd(15530.60) - 155.51).abs() < 1e-10);
        assert!((angular::iso_dm_to_dd(-15530.60) + 155.51).abs() < 1e-10);
        assert!((angular::iso_dms_to_dd(553036.0) - 55.51).abs() < 1e-10);
        assert_eq!(angular::dd_to_iso_dm(55.5025), 5530.15);
        assert_eq!(angular::dd_to_iso_dm(-55.5025), -5530.15);
        assert_eq!(angular::dd_to_iso_dms(55.5025), 553009.);
        assert_eq!(angular::dd_to_iso_dms(-55.51), -553036.);

        assert_eq!(angular::iso_dm_to_dd(5500.), 55.);
        assert_eq!(angular::iso_dm_to_dd(-5500.), -55.);
        assert_eq!(angular::iso_dm_to_dd(5530.60), -angular::iso_dm_to_dd(-5530.60));
        assert_eq!(
            angular::iso_dms_to_dd(553036.),
            -angular::iso_dms_to_dd(-553036.00)
        );
    }
}
