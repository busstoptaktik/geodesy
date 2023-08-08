/// The `OpParameter` enumeration is used to represent which defining parameters
/// are valid for a given `Op`eration.
///
/// The individual `Op`eration implementations use these to define the types of
/// the parameters accepted, and whether they are *required* (in which case the
/// provided default value is set to `None`), or *optional* (in which
/// case, a default value of the proper type is provided). The odd man out here
/// is the `Flag` type: Since a flag is a boolean which is true if present and
/// false if not, it does not make much sense to provide a default in this case.
///
/// Any other parameters given should be ignored, but warned about.
///
/// For a given operation, the union of the sets of its required and optional
/// parameters is called the *gamut* of the operation.
#[derive(Debug)]
pub enum OpParameter {
    /// A flag is a boolean that is true if present, false if not
    Flag { key: &'static str },
    /// The natural numbers + zero (𝐍₀ or 𝐖 in math terms)
    Natural {
        key: &'static str,
        default: Option<usize>,
    },
    /// Integers (𝐙 in math terms)
    Integer {
        key: &'static str,
        default: Option<i64>,
    },
    /// Reals (𝐑 in math terms)
    Real {
        key: &'static str,
        default: Option<f64>,
    },
    /// A series of reals (𝐑ⁿ in math terms)
    Series {
        key: &'static str,
        default: Option<&'static str>,
    },
    /// Any kind of text
    Text {
        key: &'static str,
        default: Option<&'static str>,
    },
    /// Any set of comma-separated texts
    Texts {
        key: &'static str,
        default: Option<&'static str>,
    },
}
