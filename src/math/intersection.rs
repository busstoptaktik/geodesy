//! Find the intersection between two line segments. The intersection may either be empty, a
//! single point or a subsegment where the segments overlap.
//!
//! NOTE: We assume that a line segment (x1, y1), (x2, y2) with x1 = x2 and y1 = y2 is
//! a valid line segment. Mathematically speaking, a line segment consists of distinct points,
//! but for completeness, we allow segments to be points in this implementation.
//!
//! Based on Java code by William Fiset, <william.alexandre.fiset@gmail.com>, from his MIT-licensed
//! [Algorithms](https://github.com/williamfiset/Algorithms/blob/master/src/main/java/com/williamfiset/algorithms/geometry/LineSegmentLineSegmentIntersection.java)
//! repository.
//!
//! See also [the related Stack Overflow discussion](https://stackoverflow.com/questions/563198/how-do-you-detect-where-two-line-segments-intersect).

use crate::authoring::*;

const EPS: f64 = 1e-9;

/// Returns two points if the segments are collinear and overlap, zero points if
/// non-overlapping parallel, and one intersection point if intersecting.
pub fn line_segment_intersection(
    line1: (Coor2D, Coor2D),
    line2: (Coor2D, Coor2D),
) -> (Option<Coor2D>, Option<Coor2D>) {
    if !segments_intersect(line1, line2) {
        return (None, None);
    }

    if line1.0.hypot2(&line1.1) < EPS
        && line1.1.hypot2(&line2.0) < EPS
        && line2.0.hypot2(&line2.1) < EPS
    {
        return (Some(line1.0), None);
    }

    let endpoints = get_common_endpoints(line1, line2);
    if endpoints.0.is_some() {
        if endpoints.1.is_none()
            && (line1.0.hypot2(&line1.1) < EPS || line2.0.hypot2(&line2.1) < EPS)
        {
            return (endpoints.0, None);
        }
        return endpoints;
    }

    // No common endpoints
    let collinear_segments = orientation(line1, line2.0) == 0 && orientation(line1, line2.1) == 0;

    if collinear_segments {
        if point_on_line(line1, line2.0) && point_on_line(line1, line2.1) {
            return (Some(line2.0), Some(line2.1));
        }

        if point_on_line(line2, line1.0) && point_on_line(line2, line1.1) {
            return (Some(line1.0), Some(line1.1));
        }

        let mid_point1 = if point_on_line(line1, line2.0) {
            line2.0
        } else {
            line2.1
        };

        let mid_point2 = if point_on_line(line2, line1.0) {
            line1.0
        } else {
            line1.1
        };

        if mid_point1.hypot2(&mid_point2) < EPS {
            return (Some(mid_point1), None);
        }

        return (Some(mid_point1), Some(mid_point2));
    }

    // line1 vertical?
    if (line1.0[0] - line1.1[0]).abs() < EPS {
        let dx = line2.1[0] - line2.0[0];
        let dy = line2.1[1] - line2.0[1];
        let m = dy / dx;
        let b = line2.0[1] - m * line2.0[0];
        return (Some(Coor2D::raw(line1.0[0], m * line1.0[0] + b)), None);
    }

    // line2 vertical?
    if (line2.0[0] - line2.1[0]).abs() < EPS {
        let dx = line1.1[0] - line1.0[0];
        let dy = line1.1[1] - line1.0[1];
        let m = dy / dx;
        let b = line1.0[1] - m * line1.0[0];
        return (Some(Coor2D::raw(line2.0[0], m * line2.0[0] + b)), None);
    }

    // None of them vertical!
    let m1 = (line1.1[1] - line1.0[1]) / (line1.1[0] - line1.0[0]);
    let m2 = (line2.1[1] - line2.0[1]) / (line2.1[0] - line2.0[0]);
    let b1 = line1.0[1] - m1 * line1.0[0];
    let b2 = line2.0[1] - m2 * line2.0[0];
    let x = (b2 - b1) / (m1 - m2);
    let y = (m1 * b2 - m2 * b1) / (m1 - m2);

    (Some(Coor2D::raw(x, y)), None)
}

fn orientation(line: (Coor2D, Coor2D), point: Coor2D) -> i32 {
    let value = (line.1[1] - line.0[1]) * (point[0] - line.1[0])
        - (line.1[0] - line.0[0]) * (point[1] - line.1[1]);
    if value.abs() < EPS {
        return 0;
    }
    if value > EPS { -1 } else { 1 }
}

fn point_on_line(line: (Coor2D, Coor2D), point: Coor2D) -> bool {
    orientation(line, point) == 0
        && line.0[0].min(line.1[0]) <= point[0]
        && point[0] <= line.0[0].max(line.1[0])
        && line.0[1].min(line.1[1]) <= point[1]
        && point[1] <= line.0[1].max(line.1[1])
}

fn segments_intersect(line1: (Coor2D, Coor2D), line2: (Coor2D, Coor2D)) -> bool {
    let o1 = orientation(line1, line2.0);
    let o2 = orientation(line1, line2.1);
    let o3 = orientation(line2, line1.0);
    let o4 = orientation(line2, line1.1);

    if o1 != o2 && o3 != o4 {
        return true;
    }
    if o1 == 0 && point_on_line(line1, line2.0) {
        return true;
    }
    if o2 == 0 && point_on_line(line1, line2.1) {
        return true;
    }
    if o3 == 0 && point_on_line(line2, line1.0) {
        return true;
    }
    if o4 == 0 && point_on_line(line2, line1.1) {
        return true;
    }
    false
}

fn get_common_endpoints(
    line1: (Coor2D, Coor2D),
    line2: (Coor2D, Coor2D),
) -> (Option<Coor2D>, Option<Coor2D>) {
    let p1 = line1.0;
    let p2 = line1.1;
    let p3 = line2.0;
    let p4 = line2.1;

    if p1.hypot2(&p3) < EPS {
        if p2.hypot2(&p4) < EPS {
            return (Some(p1), Some(p2));
        }
        return (Some(p1), None);
    }

    if p1.hypot2(&p4) < EPS {
        if p2.hypot2(&p3) < EPS {
            return (Some(p1), Some(p2));
        }
        return (Some(p1), None);
    }

    if p2.hypot2(&p3) < EPS {
        if p1.hypot2(&p4) < EPS {
            return (Some(p2), Some(p1));
        }
        return (Some(p2), None);
    }

    if p2.hypot2(&p4) < EPS {
        if p1.hypot2(&p3) < EPS {
            return (Some(p2), Some(p1));
        }
        return (Some(p2), None);
    }

    (None, None)
}

#[cfg(test)]
mod tests {
    use crate::authoring::*;

    #[test]
    fn intersection() -> Result<(), Error> {
        // Two intersecting segments. A test case from William Fiset's Java code
        let p1 = Coor2D::raw(-2., 4.);
        let p2 = Coor2D::raw(3., 3.);
        let p3 = Coor2D::raw(0., 0.);
        let p4 = Coor2D::raw(2., 4.);
        let points = super::line_segment_intersection((p1, p2), (p3, p4));
        assert!(points.1.is_none());
        let point = points.0.unwrap();
        let expected = Coor2D::raw(1.636, 3.273);
        assert!(point.hypot2(&expected) < 1e-3);

        // Non-intersecting, but also non-parallel segments
        let p1 = Coor2D::raw(0., 0.);
        let p2 = Coor2D::raw(10., 10.);
        let p3 = Coor2D::raw(10., 0.);
        let p4 = Coor2D::raw(5., 4.);
        let points = super::line_segment_intersection((p1, p2), (p3, p4));
        assert!(points.0.is_none());
        assert!(points.1.is_none());

        // The intersecting diagonals of the unit square
        let p1 = Coor2D::raw(0., 0.);
        let p2 = Coor2D::raw(1., 1.);
        let p3 = Coor2D::raw(1., 0.);
        let p4 = Coor2D::raw(0., 1.);
        let points = super::line_segment_intersection((p1, p2), (p3, p4));
        assert!(points.0.is_some() && points.1.is_none());
        assert_eq!(points.0.unwrap(), Coor2D::raw(0.5, 0.5));

        // Collinear, overlapping segments. Returning the endpoints of their common part.
        let p1 = Coor2D::raw(-10., 0.);
        let p2 = Coor2D::raw(10., 0.);
        let p3 = Coor2D::raw(-5., 0.);
        let p4 = Coor2D::raw(5., 0.);
        let points = super::line_segment_intersection((p1, p2), (p3, p4));
        assert!(points.0.is_some() && points.1.is_some());
        assert_eq!(points.0.unwrap()[0], -5.0);
        assert_eq!(points.1.unwrap()[0], 5.0);

        // Collinear, touching segments.
        let p1 = Coor2D::raw(0., 0.);
        let p2 = Coor2D::raw(10., 10.);
        let p3 = Coor2D::raw(10., 10.);
        let p4 = Coor2D::raw(30., 30.);
        let points = super::line_segment_intersection((p1, p2), (p3, p4));
        assert!(points.0.is_some() && points.1.is_none());
        assert_eq!(points.0.unwrap()[0], 10.0);

        // Collinear, non-overlapping segments.
        let p1 = Coor2D::raw(0., 0.);
        let p2 = Coor2D::raw(10., 10.);
        let p3 = Coor2D::raw(20., 20.);
        let p4 = Coor2D::raw(30., 30.);
        let points = super::line_segment_intersection((p1, p2), (p3, p4));
        assert!(points.0.is_none() && points.1.is_none());

        // Parallel segments.
        let p1 = Coor2D::raw(0., 0.);
        let p2 = Coor2D::raw(10., 10.);
        let p3 = Coor2D::raw(20., 0.);
        let p4 = Coor2D::raw(30., 10.);
        let points = super::line_segment_intersection((p1, p2), (p3, p4));
        assert!(points.0.is_none() && points.1.is_none());

        // Parallel, horizontal segments.
        let p1 = Coor2D::raw(-10., 0.);
        let p2 = Coor2D::raw(10., 0.);
        let p3 = Coor2D::raw(-5., 10.);
        let p4 = Coor2D::raw(5., 10.);
        let points = super::line_segment_intersection((p1, p2), (p3, p4));
        assert!(points.0.is_none() && points.1.is_none());

        // Four identical points
        let p1 = Coor2D::raw(-10., 0.);
        let points = super::line_segment_intersection((p1, p1), (p1, p1));
        assert!(points.1.is_none());
        assert_eq!(points.0.unwrap(), p1);

        Ok(())
    }
}
