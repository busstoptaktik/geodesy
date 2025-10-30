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

pub mod taylor {
    use super::FourierCoefficients;
    use super::POLYNOMIAL_ORDER;
    use super::PolynomialCoefficients;

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
}

// --- Fourier series summation using Clenshaw's recurrence ---

pub mod fourier {

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
    /// Functionally identical to [clenshaw_sin](crate::math::series::fourier::sin), but
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
    /// Functionally identical to [clenshaw_complex_sin](crate::math::series::fourier::complex_sin), but
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

// ----- Tests ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::taylor::*;
    use crate::authoring::*;

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
        use super::*;

        // Coefficients for 1sin(x) + 2sin(2x) + 3sin(3x)
        let coefficients = [1., 2., 3.];
        assert_eq!(fourier::sin(0., &[]), 0.);
        assert_eq!(fourier::sin(1., &[]), 0.);
        assert_eq!(fourier::sin(0.5, &[]), 0.);

        let x = 30_f64.to_radians();

        // Clenshaw sine-series summation
        let result = 1.0 * x.sin() + 2.0 * (2.0 * x).sin() + 3.0 * (3.0 * x).sin();
        assert!((fourier::sin(x, &coefficients) - result).abs() < 1e-14);

        // Clenshaw cosine-series summation
        let result = 1.0 * x.cos() + 2.0 * (2.0 * x).cos() + 3.0 * (3.0 * x).cos();
        assert!((fourier::cos(x, &coefficients) - result).abs() < 1e-14);

        // Clenshaw complex sine summation
        let coefficients = [6., 5., 4., 3., 2., 1.];
        let arg = [30f64.to_radians(), 60f64.to_radians()];
        // Canonical result from Poder/Engsager implementation
        let r = 248.658_846_388_817_7;
        let i = -463.436_347_907_636_56;
        // Let's see if we can reproduce that...
        let sum = fourier::complex_sin(arg, &coefficients);
        assert!((sum[0] - r).abs() < 1e-14);
        assert!((sum[1] - i).abs() < 1e-14);

        // Canonical result for complex cosine clenshaw, from Poder/Engsager implementation
        // let r = -461.338884918028953;
        // let i = -246.855278649982154;

        Ok(())
    }
}
