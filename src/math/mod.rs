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
