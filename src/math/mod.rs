//! Miscellaneous math functions for general use

/// Free functions used in more than one module of the crate.
pub mod ancillary;
pub use ancillary::gudermannian;

/// Free functions for handling and converting between
/// different representations of angles.
pub mod angular;

/// Computations involving the Jacobian matrix for investigation
///  of the geometrical properties of map projections.
pub mod jacobian;

/// Fourier- and Taylor series
pub mod series;
pub use series::fourier;
pub use series::taylor;

pub use series::taylor::fourier_coefficients;
pub use series::FourierCoefficients;
pub use series::PolynomialCoefficients;
