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
}
