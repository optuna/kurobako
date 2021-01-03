//! Helpers for computing hypervolumes.

/// Computes the hypervolume.
pub fn compute(pts: &Vec<Vec<f64>>, ref_pt: &Vec<f64>) -> f64 {
    assert!(!ref_pt.is_empty(), "Reference point must have at least one dimension");
    get_hypervolume_recursive(pts, ref_pt)
}

fn get_hypervolume_recursive(pts: &[Vec<f64>], ref_pt: &Vec<f64>) -> f64 {
    match pts.len() {
        1 => get_hypervolume_two_points(&pts[0], &ref_pt),
        2 => {
            get_hypervolume_two_points(&pts[0], &ref_pt)
                + get_hypervolume_two_points(&pts[1], &ref_pt)
                - get_hypervolume_two_points(&get_max_coordinates(&pts[0], &pts[1]), &ref_pt)
        }
        _ => {
            // get_exclusive_hypervolume depends on the points being sorted by the first dimension.
            let mut pts = pts.to_vec();
            pts.sort_by(|pt0, pt1| pt0[0].partial_cmp(&pt1[0]).unwrap());

            pts.iter()
                .enumerate()
                .map(|(i, pt)| get_exclusive_hypervolume(pt, &pts[i + 1..], ref_pt))
                .sum()
        }
    }
}

fn get_hypervolume_two_points(pt0: &Vec<f64>, pt1: &Vec<f64>) -> f64 {
    assert_eq!(pt0.len(), pt1.len());
    assert!(!pt0.is_empty());

    pt0.iter()
        .zip(pt1.iter())
        .fold(1.0, |prod, (&crd0, &crd1)| prod * (crd0 - crd1).abs())
}

fn get_max_coordinates(pt0: &Vec<f64>, pt1: &Vec<f64>) -> Vec<f64> {
    assert_eq!(pt0.len(), pt1.len());
    assert!(!pt0.is_empty());

    pt0.iter()
        .zip(pt1.iter())
        .map(|(&crd0, &crd1)| crd0.max(crd1))
        .collect()
}

fn get_exclusive_hypervolume(pt: &Vec<f64>, pts: &[Vec<f64>], ref_pt: &Vec<f64>) -> f64 {
    let mut limited_pts: Vec<Vec<f64>> = Vec::new();

    if pts.len() > 0 {
        let intersection_pts: Vec<Vec<f64>> = (0..pts.len())
            .map(|i| get_max_coordinates(&pts[i], &pt))
            .collect();

        limited_pts.push(intersection_pts[0].to_vec());

        // Assert that `pts` is sorted by the first dimension.
        let mut left = 0;
        let mut right = 1;

        while right < pts.len() {
            if intersection_pts[left]
                .iter()
                .zip(intersection_pts[right].iter())
                .any(|(&crd0, &crd1)| crd0 > crd1)
            {
                left = right;
                limited_pts.push(intersection_pts[left].to_vec());
            }
            right += 1;
        }
    }

    get_hypervolume_two_points(pt, ref_pt)
        - match limited_pts.len() {
            0 => 0.0,
            1 => get_hypervolume_two_points(&limited_pts[0], ref_pt),
            _ => get_hypervolume_recursive(&limited_pts, ref_pt),
        }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_float_close(v0: f64, v1: f64) -> bool {
        let rtol: f64 = 1e-5;
        let atol: f64 = 1e-8;
        (v0 - v1).abs() <= atol + rtol * v1.abs()
    }

    #[test]
    fn test_1d_single_point() {
        let pts = vec![vec![0.3]];
        let ref_pt = vec![1.0];
        let hv = 0.7;
        let hv_computed = compute(&pts, &ref_pt);
        assert!(is_float_close(hv_computed, hv));
    }

    #[test]
    fn test_1d_multiple_points() {
        let pts = vec![vec![0.5], vec![0.3], vec![0.2]];
        let ref_pt = vec![1.0];
        let hv = 0.8;
        let hv_computed = compute(&pts, &ref_pt);
        assert!(is_float_close(hv_computed, hv));
    }

    #[test]
    fn test_1d_no_points() {
        let pts = vec![];
        let ref_pt = vec![1.0];
        let hv = 0.0;
        let hv_computed = compute(&pts, &ref_pt);
        assert!(is_float_close(hv_computed, hv));
    }

    #[test]
    fn test_2d_single_point() {
        let pts = vec![vec![0.3, 0.5]];
        let ref_pt = vec![1.0, 1.0];
        let hv = 0.35;
        let hv_computed = compute(&pts, &ref_pt);
        assert!(is_float_close(hv_computed, hv));
    }

    #[test]
    fn test_2d_multiple_points() {
        let mut pts = vec![vec![0.3, 0.5], vec![0.6, 0.2]];
        let ref_pt = vec![1.0, 1.0];
        let hv = 0.47;
        let hv_computed = compute(&pts, &ref_pt);
        assert!(is_float_close(hv_computed, hv));

        // Non-Pareto optimal points do not contribute to the hypervolume.
        pts.push(vec![0.8, 0.7]);
        let hv_computed = compute(&pts, &ref_pt);
        assert!(is_float_close(hv_computed, hv));

        // Points along the Pareto front does similarly not change the hypervolume.
        pts.push(vec![0.3, 0.8]);
        pts.push(vec![0.9, 0.2]);
        let hv_computed = compute(&pts, &ref_pt);
        assert!(is_float_close(hv_computed, hv));
    }

    #[test]
    fn test_2d_no_points() {
        let pts = vec![];
        let ref_pt = vec![1.0, 1.0];
        let hv = 0.0;
        let hv_computed = compute(&pts, &ref_pt);
        assert!(is_float_close(hv_computed, hv));
    }

    #[test]
    fn test_3d_single_point() {
        let pts = vec![vec![0.5, 0.5, 0.5]];
        let ref_pt = vec![1.0, 1.0, 1.0];
        let hv = 0.125;
        let hv_computed = compute(&pts, &ref_pt);
        assert!(is_float_close(hv_computed, hv));
    }

    #[test]
    fn test_3d_no_points() {
        let pts = vec![];
        let ref_pt = vec![1.0, 1.0, 1.0];
        let hv = 0.0;
        let hv_computed = compute(&pts, &ref_pt);
        assert!(is_float_close(hv_computed, hv));
    }

    #[test]
    #[should_panic]
    fn test_invalid_reference_point() {
        let pts = vec![vec![0.5, 0.5]];
        let ref_pt = vec![];
        let _ = compute(&pts, &ref_pt);
    }
}
