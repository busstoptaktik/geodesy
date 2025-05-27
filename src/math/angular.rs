use log::warn;

/// Simplistic transformation from degrees, minutes and seconds-with-decimals
/// to degrees-with-decimals. No sanity check: Sign taken from degree-component,
/// minutes forced to unsigned by i16 type, but passing a negative value for
/// seconds leads to undefined behaviour.
pub fn dms_to_dd(d: i32, m: u16, s: f64) -> f64 {
    (d.abs() as f64 + (m as f64 + s / 60.) / 60.).copysign(d as f64)
}

/// Simplistic transformation from degrees and minutes-with-decimals
/// to degrees-with-decimals. No sanity check: Sign taken from
/// degree-component, but passing a negative value for minutes leads
/// to undefined behaviour.
pub fn dm_to_dd(d: i32, m: f64) -> f64 {
    (d.abs() as f64 + (m / 60.)).copysign(d as f64)
}

/// Simplistic transformation from the ISO-6709 DDDMM.mmm format to
/// to degrees-with-decimals. No sanity check: Invalid input,
/// such as 5575.75 (where the number of minutes exceed 60) leads
/// to undefined behaviour.
pub fn iso_dm_to_dd(iso_dm: f64) -> f64 {
    let magn = iso_dm.abs();
    let dm = magn.trunc() as u64;
    let d = (dm / 100) as f64;
    let m = (dm % 100) as f64 + magn.fract();
    (d + (m / 60.)).copysign(iso_dm)
}

/// Transformation from degrees-with-decimals to the ISO-6709 DDDMM.mmm format.
pub fn dd_to_iso_dm(dd: f64) -> f64 {
    let dm = dd.abs();
    let d = dm.trunc();
    let m = dm.fract() * 60.;
    (d * 100. + m).copysign(dd)
}

/// Simplistic transformation from the ISO-6709 DDDMMSS.sss
/// format to degrees-with-decimals. No sanity check: Invalid input,
/// such as 557575.75 (where the number of minutes and seconds both
/// exceed 60) leads to undefined behaviour.
pub fn iso_dms_to_dd(iso_dms: f64) -> f64 {
    let magn = iso_dms.abs();
    let dms = magn as u32;
    let d = dms / 10000;
    let ms = dms % 10000;
    let m = ms / 100;
    let s = (ms % 100) as f64 + magn.fract();
    (d as f64 + ((s / 60.) + m as f64) / 60.).copysign(iso_dms)
}

/// Transformation from degrees-with-decimals to the extended
/// ISO-6709 DDDMMSS.sss format.
pub fn dd_to_iso_dms(dd: f64) -> f64 {
    let magn = dd.abs();
    let d = magn.trunc();
    let mm = magn.fract() * 60.;
    let m = mm.trunc();
    let s = mm.fract() * 60.;
    (d * 10000. + m * 100. + s).copysign(dd)
}

/// normalize arbitrary angles to [-π, π)
pub fn normalize_symmetric(angle: f64) -> f64 {
    use std::f64::consts::PI;
    let angle = (angle + PI) % (2.0 * PI);
    angle - PI * angle.signum()
}

/// normalize arbitrary angles to [0, 2π)
pub fn normalize_positive(angle: f64) -> f64 {
    use std::f64::consts::PI;
    let angle = angle % (2.0 * PI);
    if angle < 0. {
        return angle + 2.0 * PI;
    }
    angle
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

    #[test]
    fn test_angular() {
        // dms
        assert_eq!(dms_to_dd(55, 30, 36.), 55.51);
        assert_eq!(dm_to_dd(55, 30.60), 55.51);

        // iso_dm + iso_dms
        assert!((iso_dm_to_dd(5530.60) - 55.51).abs() < 1e-10);
        assert!((iso_dm_to_dd(15530.60) - 155.51).abs() < 1e-10);
        assert!((iso_dm_to_dd(-15530.60) + 155.51).abs() < 1e-10);
        assert!((iso_dms_to_dd(553036.0) - 55.51).abs() < 1e-10);
        assert_eq!(dd_to_iso_dm(55.5025), 5530.15);
        assert_eq!(dd_to_iso_dm(-55.5025), -5530.15);
        assert_eq!(dd_to_iso_dms(55.5025), 553009.);
        assert_eq!(dd_to_iso_dms(-55.51), -553036.);

        assert_eq!(iso_dm_to_dd(5500.), 55.);
        assert_eq!(iso_dm_to_dd(-5500.), -55.);
        assert_eq!(iso_dm_to_dd(5530.60), -iso_dm_to_dd(-5530.60));
        assert_eq!(iso_dms_to_dd(553036.), -iso_dms_to_dd(-553036.00));
    }

    #[test]
    fn test_parse_sexagesimal() {
        assert_eq!(1.51, parse_sexagesimal("1:30:36"));
        assert_eq!(-1.51, parse_sexagesimal("-1:30:36"));
        assert_eq!(1.51, parse_sexagesimal("1:30:36N"));
        assert_eq!(-1.51, parse_sexagesimal("1:30:36S"));
        assert_eq!(1.51, parse_sexagesimal("1:30:36e"));
        assert_eq!(-1.51, parse_sexagesimal("1:30:36w"));
        assert!(parse_sexagesimal("q1:30:36w").is_nan());
    }
}
