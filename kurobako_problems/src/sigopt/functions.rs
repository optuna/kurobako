use super::bessel::bessel0;
use kurobako_core::{ErrorKind, Result};
use std::f64::consts::PI;
use std::f64::EPSILON;
use std::fmt;
use std::iter;

pub trait TestFunction: 'static + fmt::Debug + Send + Sync {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>>;
    fn evaluate(&self, xs: &[f64]) -> f64;
}

#[derive(Debug)]
pub struct Ackley;
impl TestFunction for Ackley {
    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        Ok(iter::repeat((-10.0, 30.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let dim = xs.len() as f64;
        let a = 20.0;
        let b = 0.2;
        let c = 2.0 * PI;
        let d = (xs.iter().map(|&x| x * x).sum::<f64>() / dim).sqrt();
        let e = -a * (-b * d).exp();
        let f = xs.iter().map(|&x| (c * x).cos()).sum::<f64>();
        let g = (f / dim).exp();
        e - g + a + 1f64.exp()
    }
}

#[derive(Debug)]
pub struct Adjiman;
impl TestFunction for Adjiman {
    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 2, ErrorKind::InvalidInput);
        Ok(vec![(-1.0, 2.0), (-1.0, 1.0)])
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        return xs[0].cos() * xs[1].sin() - xs[0] / (xs[1] * xs[1] + 1.0);
    }
}

#[derive(Debug)]
pub struct Alpine02;
impl TestFunction for Alpine02 {
    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 2, ErrorKind::InvalidInput);
        Ok(vec![(0.0, 10.0), (0.0, 10.0)])
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        xs.iter().map(|&x| x.sqrt() * x.sin()).product()
    }
}

#[derive(Debug)]
pub struct Branin02;
impl TestFunction for Branin02 {
    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 2, ErrorKind::InvalidInput);
        Ok(vec![(-5.0, 15.0), (-5.0, 15.0)])
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let x1 = xs[0];
        let x2 = xs[1];

        (x2 - (5.1 / (4.0 * PI * PI)) * x1 * x1 + 5.0 * x1 / PI - 6.0).powi(2)
            + 10.0 * (1.0 - 1.0 / (8.0 * PI)) * x1.cos() * x2.cos()
            + (x1 * x1 + x2 * x2 + 1.0).ln()
            + 10.0
    }
}

#[derive(Debug)]
pub struct Bukin06;
impl TestFunction for Bukin06 {
    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 2, ErrorKind::InvalidInput);
        Ok(vec![(-15.0, -5.0), (-3.0, 3.0)])
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let x1 = xs[0];
        let x2 = xs[1];
        100.0 * (x2 - 0.01 * x1 * x1).abs().sqrt() + 0.01 * (x1 + 10.0).abs()
    }
}

#[derive(Debug)]
pub struct CarromTable;
impl TestFunction for CarromTable {
    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 2, ErrorKind::InvalidInput);
        Ok(vec![(-10.0, 10.0), (-10.0, 10.0)])
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let x1 = xs[0];
        let x2 = xs[1];
        -((x1.cos() * x2.cos() * (1.0 - (x1 * x1 + x2 * x2).sqrt() / PI).abs().exp()).powi(2))
            / 30.0
    }
}

#[derive(Debug)]
pub struct Csendes;
impl TestFunction for Csendes {
    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        Ok(iter::repeat((-0.5, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        xs.iter()
            .map(|&x| x.powi(6) * (2.0 + (1.0 / (x + EPSILON)).sin()))
            .sum()
    }
}

#[derive(Debug)]
pub struct Deb02;
impl TestFunction for Deb02 {
    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let dim = xs.len() as f64;
        let a = -(1.0 / dim);
        let b = xs
            .iter()
            .map(|&x| (5.0 * PI * (x.powf(0.75) - 0.05)).sin().powi(6))
            .sum::<f64>();
        a * b
    }
}

#[derive(Debug)]
pub struct DeflectedCorrugatedSpring;
impl TestFunction for DeflectedCorrugatedSpring {
    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        let alpha = 5.0;
        Ok(iter::repeat((0.0, 1.5 * alpha)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let alpha = 5.0;
        let k = 5.0;
        let a = xs.iter().map(|&x| (x - alpha).powi(2)).sum::<f64>();
        -(k * a.sqrt()).cos() + 0.1 * a
    }
}

#[derive(Debug)]
pub struct Easom;
impl TestFunction for Easom {
    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        Ok(iter::repeat((-100.0, 20.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let n = xs.len() as f64;

        let a = 20.0;
        let b = 0.2;
        let c = 2.0 * PI;
        let d = (xs.iter().map(|&x| x * x).sum::<f64>() / n).sqrt();
        let e = (xs.iter().map(|&x| (c * x).cos()).sum::<f64>() / n).exp();
        -a * (-b * d).exp() - e + a + 1f64.exp()
    }
}

#[derive(Debug)]
pub struct Exponential;
impl TestFunction for Exponential {
    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        Ok(iter::repeat((-0.7, 0.2)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let a = xs.iter().map(|&x| x * x).sum::<f64>();
        -(-0.5 * a).exp()
    }
}

#[derive(Debug)]
pub struct Hartmann3;
impl TestFunction for Hartmann3 {
    fn default_dimension(&self) -> usize {
        3
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 3, ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let a = [
            [3.0, 0.1, 3.0, 0.1],
            [10.0, 10.0, 10.0, 10.0],
            [30.0, 35.0, 30.0, 35.0],
        ];
        let p = [
            [0.36890, 0.46990, 0.10910, 0.03815],
            [0.11700, 0.43870, 0.87320, 0.57430],
            [0.26730, 0.74700, 0.55470, 0.88280],
        ];
        let c = [1.0, 1.2, 3.0, 3.2];
        let e = (0..4)
            .map(|i| {
                let mut d = 0.0;
                for j in 0..3 {
                    d += a[j][i] * (xs[j] - p[j][i]).powi(2);
                }
                c[i] * (-d).exp()
            })
            .sum::<f64>();
        -e
    }
}

#[derive(Debug)]
pub struct Hartmann6;
impl TestFunction for Hartmann6 {
    fn default_dimension(&self) -> usize {
        6
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 6, ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let a = [
            [10.0, 0.05, 3.0, 17.0],
            [3.0, 10.0, 3.5, 8.0],
            [17.0, 17.0, 1.7, 0.05],
            [3.5, 0.1, 10.0, 10.0],
            [1.7, 8.0, 17.0, 0.1],
            [8.0, 14.0, 8.0, 14.0],
        ];
        let p = [
            [0.1312, 0.2329, 0.2348, 0.4047],
            [0.1696, 0.4135, 0.1451, 0.8828],
            [0.5569, 0.8307, 0.3522, 0.8732],
            [0.0124, 0.3736, 0.2883, 0.5743],
            [0.8283, 0.1004, 0.3047, 0.1091],
            [0.5886, 0.9991, 0.6650, 0.0381],
        ];
        let c = [1.0, 1.2, 3.0, 3.2];
        let e = (0..4)
            .map(|i| {
                let mut d = 0.0;
                for j in 0..6 {
                    d += a[j][i] * (xs[j] - p[j][i]).powi(2);
                }
                c[i] * (-d).exp()
            })
            .sum::<f64>();
        -e
    }
}

#[derive(Debug)]
pub struct HelicalValley;
impl TestFunction for HelicalValley {
    fn default_dimension(&self) -> usize {
        3
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 3, ErrorKind::InvalidInput);
        Ok(iter::repeat((-1.0, 2.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let x1 = xs[0];
        let x2 = xs[1];
        let x3 = xs[2];
        100.0
            * ((x3 - 10.0 * x2.atan2(x1) / 2.0 / PI).powi(2)
                + ((x1 * x1 + x2 * x2).sqrt() - 1.0).powi(2))
            + x3 * x3
    }
}

#[derive(Debug)]
pub struct HimmelBlau;
impl TestFunction for HimmelBlau {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 2, ErrorKind::InvalidInput);
        Ok(iter::repeat((-2.0, 6.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let x1 = xs[0];
        let x2 = xs[1];
        (x1.powi(2) + x2 - 11.0).powi(2) + (x1 + x2.powi(2) - 7.0).powi(2)
    }
}

#[derive(Debug)]
pub struct LennardJones6;
impl TestFunction for LennardJones6 {
    fn default_dimension(&self) -> usize {
        6
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 6, ErrorKind::InvalidInput);
        Ok(iter::repeat((-3.0, 3.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let k = xs.len() / 3;
        let mut s = 0.0;
        for i in 0..k - 1 {
            for j in i + 1..k {
                let a = 3 * i;
                let b = 3 * j;
                let xd = xs[a] - xs[b];
                let yd = xs[a + 1] - xs[b + 1];
                let zd = xs[a + 2] - xs[b + 2];
                let ed = xd * xd + yd * yd + zd * zd;
                let ud = ed * ed * ed + 1e-8;
                if ed > 0.0 {
                    s += (1.0 / ud - 2.0) / ud;
                }
            }
        }
        s.min(0.0)
    }
}

struct McCourtBase;
impl McCourtBase {
    fn evaluate<'a, F, I>(xs: &'a [f64], kernel: F, coefs: &'static [f64]) -> f64
    where
        F: FnOnce(&'a [f64]) -> I,
        I: 'a + Iterator<Item = f64>,
    {
        coefs.iter().zip(kernel(xs)).map(|(&c, k)| c * k).sum()
    }

    fn dist_sq_1<'a>(
        xs: &'a [f64],
        centers: &'static [&'static [f64]],
        e_mat: &'static [&'static [f64]],
    ) -> impl 'a + Iterator<Item = f64> {
        e_mat.iter().zip(centers.iter()).map(move |(evec, center)| {
            xs.iter()
                .zip(center.iter())
                .zip(evec.iter().map(|&x| x.sqrt()))
                .map(|((&x, &c), e)| ((x - c) * e).abs())
                .sum::<f64>()
        })
    }

    fn dist_sq_2<'a>(
        xs: &'a [f64],
        centers: &'static [&'static [f64]],
        e_mat: &'static [&'static [f64]],
    ) -> impl 'a + Iterator<Item = f64> {
        e_mat.iter().zip(centers.iter()).map(move |(evec, center)| {
            let a = xs
                .iter()
                .zip(center.iter())
                .zip(evec.iter())
                .map(|((&x, &c), &e)| (x - c) * e);
            let b = xs.iter().zip(center.iter()).map(|(&x, &c)| x - c);
            a.zip(b).map(|(a, b)| a * b).sum::<f64>()
        })
    }

    fn dist_sq_inf<'a>(
        xs: &'a [f64],
        centers: &'static [&'static [f64]],
        e_mat: &'static [&'static [f64]],
    ) -> impl 'a + Iterator<Item = f64> {
        e_mat.iter().zip(centers.iter()).map(move |(evec, center)| {
            let mut max = std::f64::NEG_INFINITY;
            for x in xs
                .iter()
                .zip(center.iter())
                .zip(evec.iter().map(|&x| x.sqrt()))
                .map(|((&x, &c), e)| ((x - c) * e).abs())
            {
                if x > max {
                    max = x;
                }
            }
            max
        })
    }
}

#[derive(Debug)]
pub struct McCourt01;
impl McCourt01 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[f64]] = &[
            &[0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1],
            &[0.3, 0.1, 0.5, 0.1, 0.8, 0.8, 0.6],
            &[0.6, 0.7, 0.8, 0.3, 0.7, 0.8, 0.6],
            &[0.4, 0.7, 0.4, 0.9, 0.4, 0.1, 0.9],
            &[0.9, 0.3, 0.3, 0.5, 0.2, 0.7, 0.2],
            &[0.5, 0.5, 0.2, 0.8, 0.5, 0.3, 0.4],
        ];
        let e_mat: &[&[f64]] = &[
            &[5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0],
            &[5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0],
            &[5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0],
            &[5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0],
            &[5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0],
            &[5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0],
        ];
        McCourtBase::dist_sq_2(xs, centers, e_mat).map(|r2| 1.0 / (1.0 + r2).sqrt())
    }
}
impl TestFunction for McCourt01 {
    fn default_dimension(&self) -> usize {
        7
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 7, ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[1.0, 1.0, -2.0, 1.0, 1.0, 1.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt02;
impl McCourt02 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[
            &[0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1],
            &[0.3, 0.1, 0.5, 0.1, 0.8, 0.8, 0.6],
            &[0.6, 0.7, 0.8, 0.3, 0.7, 0.8, 0.6],
            &[0.4, 0.7, 0.4, 0.9, 0.4, 0.1, 0.9],
            &[0.9, 0.3, 0.3, 0.5, 0.2, 0.7, 0.2],
            &[0.5, 0.5, 0.2, 0.8, 0.5, 0.3, 0.4],
        ];
        let e_mat: &[&[_]] = &[
            &[5., 5., 5., 5., 5., 5., 5.],
            &[1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 1.5],
            &[1., 1., 1., 1., 1., 1., 1.],
            &[5., 5., 5., 5., 5., 5., 5.],
            &[5., 5., 5., 5., 5., 5., 5.],
            &[5., 5., 5., 5., 5., 5., 5.],
        ];
        McCourtBase::dist_sq_2(xs, centers, e_mat).map(|r2| 1.0 / (1.0 + r2).sqrt())
    }
}
impl TestFunction for McCourt02 {
    fn default_dimension(&self) -> usize {
        7
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 7, ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[-1.0, -1.0, -2.0, 1.0, 1.0, -1.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt03;
impl McCourt03 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[
            &[0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1],
            &[0.3, 0.1, 0.5, 0.1, 0.8, 0.8, 0.6, 0.4, 0.2],
            &[0.6, 0.7, 0.8, 0.3, 0.7, 0.8, 0.6, 0.9, 0.1],
            &[0.7, 0.2, 0.7, 0.7, 0.3, 0.3, 0.8, 0.6, 0.4],
            &[0.4, 0.6, 0.4, 0.9, 0.4, 0.1, 0.9, 0.3, 0.3],
            &[0.5, 0.5, 0.2, 0.8, 0.5, 0.3, 0.4, 0.5, 0.8],
            &[0.8, 0.3, 0.3, 0.5, 0.2, 0.7, 0.2, 0.4, 0.6],
            &[0.8, 0.3, 0.3, 0.5, 0.2, 0.7, 0.2, 0.4, 0.6],
            &[0.8, 0.3, 0.3, 0.5, 0.2, 0.7, 0.2, 0.4, 0.6],
        ];
        let e_mat: &[&[_]] = &[
            &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
            &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
            &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
            &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
            &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
            &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
            &[0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1],
            &[0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        ];
        McCourtBase::dist_sq_2(xs, centers, e_mat).map(|r2| (-r2).exp())
    }
}
impl TestFunction for McCourt03 {
    fn default_dimension(&self) -> usize {
        9
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 9, ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[1.0, -1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -2.0, -1.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt06;
impl McCourt06 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[
            &[0.1, 0.1, 0.1, 0.1, 0.1],
            &[0.3, 0.8, 0.8, 0.6, 0.9],
            &[0.6, 0.1, 0.2, 0.5, 0.2],
            &[0.7, 0.2, 0.1, 0.8, 0.9],
            &[0.4, 0.6, 0.5, 0.3, 0.8],
            &[0.9, 0.5, 0.3, 0.2, 0.4],
            &[0.2, 0.8, 0.6, 0.4, 0.6],
        ];
        let e_mat: &[&[_]] = &[
            &[0.4, 0.4, 0.4, 0.4, 0.4],
            &[0.2, 0.2, 0.2, 0.2, 0.2],
            &[0.4, 0.4, 0.4, 0.4, 0.4],
            &[0.08, 0.08, 0.08, 0.08, 0.08],
            &[0.2, 0.2, 0.2, 0.2, 0.2],
            &[0.4, 0.4, 0.4, 0.4, 0.4],
            &[0.4, 0.4, 0.4, 0.4, 0.4],
        ];
        McCourtBase::dist_sq_2(xs, centers, e_mat).map(|r2| (1.0 + r2).sqrt())
    }
}
impl TestFunction for McCourt06 {
    fn default_dimension(&self) -> usize {
        5
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 5, ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[-3.0, 2.0, -2.0, 4.0, -1.0, 5.0, -1.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt07;
impl McCourt07 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[
            &[0.1, 0.1, 0.1, 0.1, 0.1, 0.1],
            &[0.3, 0.8, 0.8, 0.6, 0.9, 0.4],
            &[0.6, 1.0, 0.2, 0.0, 1.0, 0.3],
            &[0.7, 0.2, 0.1, 0.8, 0.9, 0.2],
            &[0.4, 0.6, 0.5, 0.3, 0.8, 0.3],
            &[0.9, 0.5, 0.3, 0.2, 0.4, 0.8],
            &[0.2, 0.8, 0.6, 0.4, 0.6, 0.9],
        ];
        let e_mat: &[&[_]] = &[
            &[0.7, 0.7, 0.7, 0.7, 0.7, 0.7],
            &[0.35, 0.35, 0.35, 0.35, 0.35, 0.35],
            &[0.7, 0.7, 0.7, 0.7, 0.7, 0.7],
            &[0.14, 0.14, 0.14, 0.14, 0.14, 0.14],
            &[0.35, 0.35, 0.35, 0.35, 0.35, 0.35],
            &[0.7, 0.7, 0.7, 0.7, 0.7, 0.7],
            &[0.49, 0.49, 0.49, 0.49, 0.49, 0.49],
        ];
        McCourtBase::dist_sq_2(xs, centers, e_mat)
            .map(|r2| r2.sqrt())
            .map(|r| (1.0 + r) * (-r).exp())
    }
}
impl TestFunction for McCourt07 {
    fn default_dimension(&self) -> usize {
        6
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, self.default_dimension(), ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[2.0, 2.0, -4.0, 1.0, -2.0, 4.0, -2.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt08;
impl McCourt08 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[
            &[0.1, 0.1, 0.1, 0.1],
            &[0.3, 0.8, 0.9, 0.4],
            &[0.6, 1.0, 0.2, 0.0],
            &[0.7, 0.2, 0.1, 0.8],
            &[0.4, 0.0, 0.8, 1.0],
            &[0.9, 0.5, 0.3, 0.2],
            &[0.2, 0.8, 0.6, 0.4],
            &[0.1, 0.1, 0.1, 0.1],
            &[0.3, 0.8, 0.9, 0.4],
            &[0.6, 1.0, 0.2, 0.0],
            &[0.7, 0.2, 0.1, 0.8],
            &[0.4, 0.0, 0.8, 1.0],
            &[0.9, 0.5, 0.3, 0.2],
            &[0.2, 0.8, 0.6, 0.4],
        ];
        let e_mat: &[&[_]] = &[
            &[0.7, 0.7, 0.7, 0.7],
            &[0.35, 0.35, 0.35, 0.35],
            &[0.7, 2.1, 0.7, 2.1],
            &[0.35, 0.35, 0.35, 0.35],
            &[1.4, 0.7, 1.4, 0.7],
            &[0.7, 0.7, 0.7, 0.7],
            &[0.49, 0.49, 0.49, 0.49],
        ];
        McCourtBase::dist_sq_2(xs, centers, e_mat)
            .map(|r2| r2.sqrt())
            .map(|r| (1.0 + r + 0.333 * r * r) * (-r).exp())
    }
}
impl TestFunction for McCourt08 {
    fn default_dimension(&self) -> usize {
        4
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, self.default_dimension(), ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[2.0, 1.0, -8.0, 1.0, -5.0, 3.0, 2.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt09;
impl McCourt09 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[
            &[0.1, 0.1, 0.1],
            &[0.3, 0.8, 0.9],
            &[0.6, 1.0, 0.2],
            &[0.6, 1.0, 0.2],
            &[0.7, 0.2, 0.1],
            &[0.4, 0.0, 0.8],
            &[0.9, 0.5, 1.0],
            &[0.0, 0.8, 0.6],
        ];
        let e_mat: &[&[_]] = &[
            &[0.6, 0.6, 0.6],
            &[0.36, 0.36, 0.36],
            &[0.6, 0.3, 0.6],
            &[2.4, 6.0, 2.4],
            &[0.3, 0.3, 0.3],
            &[0.3, 0.6, 0.3],
            &[0.6, 0.6, 0.6],
            &[0.18, 0.3, 0.3],
        ];
        McCourtBase::dist_sq_2(xs, centers, e_mat).map(|r2| (PI * r2.sqrt()).cos() * (-r2).exp())
    }
}
impl TestFunction for McCourt09 {
    fn default_dimension(&self) -> usize {
        3
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, self.default_dimension(), ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[4.0, -3.0, -6.0, -2.0, 1.0, -3.0, 6.0, 2.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt10;
impl McCourt10 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[
            &[0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1],
            &[0.3, 0.1, 0.5, 0.1, 0.8, 0.8, 0.6, 0.4],
            &[0.6, 0.7, 0.8, 0.3, 0.7, 0.8, 0.6, 0.9],
            &[0.7, 0.0, 0.7, 1.0, 0.3, 0.0, 0.8, 0.6],
            &[0.4, 0.6, 0.4, 1.0, 0.4, 0.2, 1.0, 0.3],
            &[0.5, 0.5, 0.2, 0.8, 0.5, 0.3, 0.4, 0.5],
            &[0.1, 0.2, 1.0, 0.4, 0.5, 0.6, 0.7, 0.0],
            &[0.9, 0.4, 0.3, 0.5, 0.2, 0.7, 0.2, 0.4],
            &[0.0, 0.5, 0.3, 0.2, 0.1, 0.9, 0.3, 0.7],
            &[0.2, 0.8, 0.6, 0.4, 0.6, 0.6, 0.5, 0.0],
        ];
        let e_mat: &[&[_]] = &[
            &[0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8],
            &[0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8],
            &[0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8],
            &[0.4, 0.4, 0.4, 0.4, 0.4, 0.4, 0.4, 0.4],
            &[0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8],
            &[
                2.4000000000000004,
                2.4000000000000004,
                2.4000000000000004,
                2.4000000000000004,
                2.4000000000000004,
                2.4000000000000004,
                2.4000000000000004,
                2.4000000000000004,
            ],
            &[0.4, 0.4, 0.4, 0.4, 0.4, 0.4, 0.4, 0.4],
            &[0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8],
            &[1.6, 1.6, 1.6, 1.6, 1.6, 1.6, 1.6, 1.6],
            &[0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8],
        ];
        McCourtBase::dist_sq_2(xs, centers, e_mat).map(|r2| 1.0 / (1.0 + r2).sqrt())
    }
}
impl TestFunction for McCourt10 {
    fn default_dimension(&self) -> usize {
        8
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, self.default_dimension(), ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[5.0, -2.0, 5.0, -5.0, -12.0, -2.0, 10.0, 2.0, -5.0, 5.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt11;
impl McCourt11 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[
            &[0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1],
            &[0.3, 0.1, 0.5, 0.1, 0.8, 0.8, 0.6, 0.4],
            &[0.6, 0.7, 0.8, 0.3, 0.7, 0.8, 0.6, 0.9],
            &[0.7, 0.0, 0.7, 1.0, 0.3, 0.0, 0.8, 0.6],
            &[0.4, 0.6, 0.4, 1.0, 0.4, 0.2, 1.0, 0.3],
            &[0.5, 0.5, 0.2, 0.8, 0.5, 0.3, 0.4, 0.5],
            &[0.1, 0.2, 1.0, 0.4, 0.5, 0.6, 0.7, 0.0],
            &[0.9, 0.4, 0.3, 0.5, 0.2, 0.7, 0.2, 0.4],
            &[0.0, 0.5, 0.3, 0.2, 0.1, 0.9, 0.3, 0.7],
            &[0.2, 0.8, 0.6, 0.4, 0.6, 0.6, 0.5, 0.0],
        ];
        let e_mat: &[&[_]] = &[
            &[0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            &[0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            &[0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            &[0.25, 0.25, 0.25, 0.25, 0.25, 0.25, 0.25, 0.25],
            &[0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            &[1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 1.5],
            &[0.25, 0.25, 0.25, 0.25, 0.25, 0.25, 0.25, 0.25],
            &[0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
            &[0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
        ];
        McCourtBase::dist_sq_2(xs, centers, e_mat)
            .map(|r2| r2.sqrt())
            .map(|r| (-r).exp())
    }
}
impl TestFunction for McCourt11 {
    fn default_dimension(&self) -> usize {
        8
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, self.default_dimension(), ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[5.0, -2.0, 5.0, -5.0, -7.0, -2.0, 10.0, 2.0, -5.0, 5.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt12;
impl McCourt12 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[
            &[0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1],
            &[0.3, 0.1, 0.5, 0.1, 0.8, 0.8, 0.6],
            &[0.6, 0.7, 0.8, 0.3, 0.7, 0.8, 0.6],
            &[0.7, 0.0, 0.7, 1.0, 0.3, 0.0, 0.8],
            &[0.4, 0.6, 0.4, 1.0, 0.4, 0.2, 1.0],
            &[0.5, 0.5, 0.2, 0.8, 0.5, 0.3, 0.4],
            &[0.1, 0.2, 1.0, 0.4, 0.5, 0.6, 0.7],
            &[0.9, 0.4, 0.3, 0.5, 0.2, 0.7, 0.2],
            &[0.0, 0.5, 0.3, 0.2, 0.1, 0.9, 0.3],
            &[0.2, 0.8, 0.6, 0.4, 0.6, 0.6, 0.5],
        ];
        let e_mat: &[&[_]] = &[
            &[0.7, 0.7, 0.7, 0.7, 0.7, 0.7, 0.7],
            &[0.7, 0.7, 0.7, 0.7, 0.7, 0.7, 0.7],
            &[0.7, 0.7, 0.7, 0.7, 0.7, 0.7, 0.7],
            &[0.35, 0.35, 0.35, 0.35, 0.35, 0.35, 0.35],
            &[0.7, 0.7, 0.7, 0.7, 0.7, 0.7, 0.7],
            &[7.0, 7.0, 7.0, 7.0, 7.0, 7.0, 7.0],
            &[0.35, 0.35, 0.35, 0.35, 0.35, 0.35, 0.35],
            &[0.7, 0.7, 0.7, 0.7, 0.7, 0.7, 0.7],
            &[1.4, 1.4, 1.4, 1.4, 1.4, 1.4, 1.4],
            &[0.7, 0.7, 0.7, 0.7, 0.7, 0.7, 0.7],
        ];
        McCourtBase::dist_sq_2(xs, centers, e_mat)
            .map(|r2| r2.sqrt())
            .map(|r| bessel0(r))
    }
}
impl TestFunction for McCourt12 {
    fn default_dimension(&self) -> usize {
        7
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, self.default_dimension(), ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[5.0, -4.0, 5.0, -5.0, -7.0, -2.0, 10.0, 2.0, -5.0, 5.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt13;
impl McCourt13 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[
            &[0.9, 0.9, 0.9],
            &[0.9, 0.9, 1.0],
            &[0.9, 1.0, 0.9],
            &[1.0, 0.9, 0.9],
            &[1.0, 1.0, 1.0],
            &[1.0, 0.0, 0.0],
            &[0.5, 0.0, 0.0],
            &[0.0, 1.0, 0.0],
            &[0.0, 0.7, 0.0],
            &[0.0, 0.0, 0.0],
            &[0.4, 0.3, 0.6],
            &[0.7, 0.7, 0.7],
            &[0.7, 0.7, 1.0],
            &[1.0, 0.7, 0.7],
            &[0.7, 1.0, 0.7],
        ];
        let e_mat: &[&[_]] = &[
            &[7.6000000000000005, 7.6000000000000005, 7.6000000000000005],
            &[7.6000000000000005, 7.6000000000000005, 7.6000000000000005],
            &[7.6000000000000005, 7.6000000000000005, 7.6000000000000005],
            &[7.6000000000000005, 7.6000000000000005, 7.6000000000000005],
            &[7.6000000000000005, 7.6000000000000005, 7.6000000000000005],
            &[0.8, 0.4, 0.8],
            &[1.6, 0.4, 0.8],
            &[0.4, 0.4, 0.4],
            &[0.4, 0.8, 0.4],
            &[0.8, 0.8, 0.8],
            &[1.6, 1.6, 2.8000000000000003],
            &[6.800000000000001, 6.800000000000001, 6.800000000000001],
            &[6.800000000000001, 6.800000000000001, 6.800000000000001],
            &[6.800000000000001, 6.800000000000001, 6.800000000000001],
            &[6.800000000000001, 6.800000000000001, 6.800000000000001],
        ];
        McCourtBase::dist_sq_2(xs, centers, e_mat).map(|r2| (-r2).exp())
    }
}
impl TestFunction for McCourt13 {
    fn default_dimension(&self) -> usize {
        3
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, self.default_dimension(), ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[
            4.0, 4.0, 4.0, 4.0, -12.0, 1.0, 3.0, -2.0, 5.0, -2.0, 1.0, -2.0, -2.0, -2.0, -2.0,
        ];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt14;
impl McCourt14 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[&[0.1, 0.8, 0.3]];
        let e_mat: &[&[_]] = &[&[5.0, 5.0, 5.0]];
        McCourtBase::dist_sq_2(xs, centers, e_mat).map(|r2| (-r2).exp())
    }
}
impl TestFunction for McCourt14 {
    fn default_dimension(&self) -> usize {
        3
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, self.default_dimension(), ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[-5.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt16;
impl McCourt16 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[&[0.3, 0.8, 0.3, 0.6], &[0.4, 0.9, 0.4, 0.7]];
        let e_mat: &[&[_]] = &[&[5.0, 5.0, 5.0, 5.0], &[5.0, 5.0, 5.0, 5.0]];
        McCourtBase::dist_sq_2(xs, centers, e_mat).map(|r2| 1.0 / (1.0 + r2).sqrt())
    }
}
impl TestFunction for McCourt16 {
    fn default_dimension(&self) -> usize {
        4
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, self.default_dimension(), ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[-5.0, 5.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt17;
impl McCourt17 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[
            &[0.3, 0.8, 0.3, 0.6, 0.2, 0.8, 0.5],
            &[0.8, 0.3, 0.8, 0.2, 0.5, 0.2, 0.8],
            &[0.2, 0.7, 0.2, 0.5, 0.4, 0.7, 0.3],
        ];
        let e_mat: &[&[_]] = &[
            &[4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0],
            &[4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0],
            &[4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0],
        ];
        McCourtBase::dist_sq_2(xs, centers, e_mat).map(|r2| 1.0 / (1.0 + r2).sqrt())
    }
}
impl TestFunction for McCourt17 {
    fn default_dimension(&self) -> usize {
        7
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, self.default_dimension(), ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[-5.0, 5.0, 5.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt18;
impl McCourt18 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[
            &[0.3, 0.8, 0.3, 0.6, 0.2, 0.8, 0.2, 0.4],
            &[0.3, 0.8, 0.3, 0.6, 0.2, 0.8, 0.2, 0.4],
            &[0.3, 0.8, 0.3, 0.6, 0.2, 0.8, 0.2, 0.4],
            &[0.8, 0.3, 0.8, 0.2, 0.5, 0.2, 0.5, 0.7],
            &[0.2, 0.7, 0.2, 0.5, 0.4, 0.3, 0.8, 0.8],
        ];
        let e_mat: &[&[_]] = &[
            &[0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
            &[4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0],
            &[4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0],
            &[4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0],
        ];
        McCourtBase::dist_sq_2(xs, centers, e_mat)
            .map(|r2| r2.sqrt())
            .map(|r| (1.0 + r) * (-r).exp())
    }
}
impl TestFunction for McCourt18 {
    fn default_dimension(&self) -> usize {
        8
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, self.default_dimension(), ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[-1.0, 2.0, -5.0, 4.0, 4.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt19;
impl McCourt19 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[
            &[0.1, 0.1],
            &[0.3, 0.8],
            &[0.6, 0.7],
            &[0.7, 0.1],
            &[0.4, 0.3],
            &[0.2, 0.8],
            &[0.1, 0.2],
            &[0.9, 0.4],
            &[0.5, 0.5],
            &[0.0, 0.8],
        ];
        let e_mat: &[&[_]] = &[
            &[3.0, 3.0],
            &[3.0, 3.0],
            &[3.0, 3.0],
            &[1.5, 1.5],
            &[3.0, 3.0],
            &[9.0, 9.0],
            &[1.5, 1.5],
            &[3.0, 3.0],
            &[6.0, 6.0],
            &[3.0, 3.0],
        ];
        McCourtBase::dist_sq_1(xs, centers, e_mat)
    }
}
impl TestFunction for McCourt19 {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, self.default_dimension(), ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[-5.0, 4.0, -5.0, 5.0, 4.0, 2.0, -10.0, -4.0, 5.0, 5.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt20;
impl McCourt20 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[
            &[0.1, 0.1],
            &[0.3, 0.8],
            &[0.6, 0.7],
            &[0.7, 0.1],
            &[0.4, 0.3],
            &[0.2, 0.8],
            &[0.1, 0.2],
            &[0.9, 0.4],
            &[0.5, 0.5],
            &[0.0, 0.8],
        ];
        let e_mat: &[&[_]] = &[
            &[50.0, 50.0],
            &[50.0, 50.0],
            &[50.0, 50.0],
            &[25.0, 25.0],
            &[50.0, 50.0],
            &[150.0, 150.0],
            &[25.0, 25.0],
            &[50.0, 50.0],
            &[100.0, 100.0],
            &[50.0, 50.0],
        ];
        McCourtBase::dist_sq_1(xs, centers, e_mat).map(|rabs| (-rabs).exp())
    }
}
impl TestFunction for McCourt20 {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, self.default_dimension(), ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[5.0, -4.0, 5.0, -7.0, -4.0, -2.0, 10.0, 4.0, -2.0, -5.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt22;
impl McCourt22 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[
            &[1.0, 0.3, 0.1, 0.4, 0.1],
            &[0.9, 0.7, 0.0, 0.5, 0.8],
            &[0.5, 0.6, 0.6, 0.5, 0.5],
            &[0.2, 0.2, 0.4, 0.0, 0.3],
            &[0.0, 0.6, 1.0, 0.1, 0.8],
            &[0.3, 0.5, 0.8, 0.0, 0.2],
            &[0.8, 1.0, 0.1, 0.1, 0.5],
        ];
        let e_mat: &[&[_]] = &[
            &[5.0, 30.0, 25.0, 5.0, 15.0],
            &[10.0, 30.0, 10.0, 5.0, 5.0],
            &[5.0, 10.0, 5.0, 10.0, 5.0],
            &[20.0, 5.0, 20.0, 5.0, 5.0],
            &[25.0, 30.0, 5.0, 15.0, 10.0],
            &[20.0, 10.0, 15.0, 5.0, 20.0],
            &[15.0, 25.0, 5.0, 20.0, 25.0],
        ];
        McCourtBase::dist_sq_inf(xs, centers, e_mat).map(|rmax| (-rmax).exp())
    }
}
impl TestFunction for McCourt22 {
    fn default_dimension(&self) -> usize {
        5
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, self.default_dimension(), ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[3.0, 4.0, -4.0, 2.0, -3.0, -2.0, 6.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt23;
impl McCourt23 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[
            &[0.1, 0.1, 1.0, 0.3, 0.4, 0.1],
            &[0.0, 0.0, 0.1, 0.6, 0.0, 0.7],
            &[0.1, 0.5, 0.7, 0.0, 0.7, 0.3],
            &[0.9, 0.6, 0.2, 0.9, 0.3, 0.8],
            &[0.8, 0.3, 0.7, 0.7, 0.2, 0.7],
            &[0.7, 0.6, 0.5, 1.0, 1.0, 0.7],
            &[0.8, 0.9, 0.5, 0.0, 0.0, 0.5],
            &[0.3, 0.0, 0.3, 0.2, 0.1, 0.8],
        ];
        let e_mat: &[&[_]] = &[
            &[0.4, 0.5, 0.5, 0.4, 0.1, 0.5],
            &[0.2, 0.4, 0.5, 0.1, 0.2, 0.2],
            &[0.1, 0.4, 0.30000000000000004, 0.2, 0.2, 0.30000000000000004],
            &[0.4, 0.2, 0.30000000000000004, 0.4, 0.1, 0.4],
            &[
                0.2,
                0.30000000000000004,
                0.6000000000000001,
                0.6000000000000001,
                0.4,
                0.1,
            ],
            &[0.5, 0.4, 0.1, 0.4, 0.1, 0.1],
            &[0.2, 0.2, 0.2, 0.5, 0.4, 0.2],
            &[
                0.1,
                0.4,
                0.6000000000000001,
                0.30000000000000004,
                0.4,
                0.30000000000000004,
            ],
        ];
        McCourtBase::dist_sq_inf(xs, centers, e_mat).map(|rmax| bessel0(rmax))
    }
}
impl TestFunction for McCourt23 {
    fn default_dimension(&self) -> usize {
        6
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, self.default_dimension(), ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[1.0, -2.0, 3.0, -20.0, 5.0, -2.0, -1.0, -2.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt26;
impl McCourt26 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[
            &[0.5, 0.2, 0.0],
            &[0.6, 0.2, 0.5],
            &[0.4, 0.6, 0.5],
            &[0.5, 0.7, 0.3],
            &[0.4, 0.4, 0.4],
            &[0.8, 0.5, 0.8],
            &[0.0, 0.0, 0.8],
            &[0.7, 0.7, 0.2],
            &[0.9, 0.3, 1.0],
            &[0.4, 0.4, 0.8],
            &[0.2, 0.8, 0.8],
        ];
        let e_mat: &[&[_]] = &[
            &[1.0, 1.0, 1.0],
            &[3.0, 2.5, 1.5],
            &[1.5, 1.5, 1.5],
            &[2.5, 1.0, 2.5],
            &[2.0, 3.0, 1.5],
            &[1.0, 1.0, 1.5],
            &[1.0, 2.0, 0.5],
            &[2.0, 3.0, 2.0],
            &[0.5, 1.5, 2.0],
            &[1.5, 1.0, 1.0],
            &[3.0, 1.0, 1.5],
        ];
        McCourtBase::dist_sq_1(xs, centers, e_mat).map(|rmax| (-rmax).exp())
    }
}
impl TestFunction for McCourt26 {
    fn default_dimension(&self) -> usize {
        3
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, self.default_dimension(), ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[1.0, 2.0, 3.0, -5.0, 3.0, -2.0, 1.0, -2.0, 5.0, 2.0, -2.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt27;
impl McCourt27 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[
            &[0.6, 0.3, 0.5],
            &[0.5, 0.2, 0.0],
            &[0.4, 0.6, 0.5],
            &[0.5, 0.7, 0.3],
            &[0.4, 0.4, 0.4],
            &[0.8, 0.5, 0.8],
            &[0.0, 0.0, 0.8],
            &[0.7, 0.0, 0.2],
            &[0.9, 0.3, 1.0],
            &[0.4, 0.4, 0.8],
            &[0.2, 0.8, 0.8],
        ];
        let e_mat: &[&[_]] = &[
            &[2.0, 2.0, 2.0],
            &[6.0, 5.0, 3.0],
            &[3.0, 3.0, 3.0],
            &[5.0, 2.0, 5.0],
            &[4.0, 6.0, 3.0],
            &[2.0, 2.0, 3.0],
            &[2.0, 4.0, 1.0],
            &[4.0, 6.0, 4.0],
            &[1.0, 3.0, 4.0],
            &[3.0, 2.0, 2.0],
            &[6.0, 2.0, 3.0],
        ];
        McCourtBase::dist_sq_1(xs, centers, e_mat).map(|rmax| (-rmax).exp())
    }
}
impl TestFunction for McCourt27 {
    fn default_dimension(&self) -> usize {
        3
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, self.default_dimension(), ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[-10.0, 2.0, 3.0, 5.0, 3.0, 2.0, 1.0, 2.0, 5.0, 2.0, 2.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct McCourt28;
impl McCourt28 {
    fn kernel<'a>(xs: &'a [f64]) -> impl 'a + Iterator<Item = f64> {
        let centers: &[&[_]] = &[
            &[0.6, 0.2, 0.8, 0.4],
            &[0.1, 0.1, 0.7, 0.9],
            &[1.0, 0.1, 0.8, 0.6],
            &[0.0, 0.3, 0.2, 1.0],
            &[0.2, 1.0, 0.8, 0.0],
            &[0.6, 0.9, 0.2, 0.9],
            &[0.1, 0.7, 0.6, 0.8],
            &[0.8, 0.4, 0.3, 0.2],
            &[0.1, 1.0, 0.8, 0.2],
            &[0.3, 0.9, 0.9, 0.0],
            &[0.8, 1.0, 0.6, 0.9],
        ];
        let e_mat: &[&[_]] = &[
            &[1.0, 1.0, 1.0, 1.0],
            &[5.0, 3.0, 3.0, 3.0],
            &[4.0, 6.0, 2.0, 4.0],
            &[4.0, 1.0, 6.0, 3.0],
            &[2.0, 5.0, 3.0, 5.0],
            &[5.0, 4.0, 6.0, 1.0],
            &[6.0, 4.0, 1.0, 6.0],
            &[5.0, 1.0, 2.0, 1.0],
            &[1.0, 5.0, 4.0, 2.0],
            &[1.0, 3.0, 3.0, 2.0],
            &[4.0, 6.0, 6.0, 2.0],
        ];
        McCourtBase::dist_sq_2(xs, centers, e_mat).map(|r2| (-r2).exp())
    }
}
impl TestFunction for McCourt28 {
    fn default_dimension(&self) -> usize {
        4
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, self.default_dimension(), ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let coefs = &[-10.0, 2.0, 3.0, 5.0, 3.0, 2.0, 1.0, 2.0, 5.0, 2.0, 2.0];
        McCourtBase::evaluate(xs, Self::kernel, coefs)
    }
}

#[derive(Debug)]
pub struct Michalewicz;
impl TestFunction for Michalewicz {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        let max_dim = 12;
        track_assert!(dim <= max_dim, ErrorKind::InvalidInput; dim, max_dim);
        Ok(iter::repeat((0.0, PI)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let m = 10;
        -(1..=xs.len())
            .zip(xs.iter())
            .map(|(i, &x)| x.sin() * (i as f64 * x.powi(2) / PI).sin().powi(2 * m))
            .sum::<f64>()
    }
}

#[derive(Debug)]
pub struct Mishra06;
impl TestFunction for Mishra06 {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 2, ErrorKind::InvalidInput);
        Ok(iter::repeat((-10.0, 10.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let x1 = xs[0];
        let x2 = xs[1];
        let a = (x1.cos() + x2.cos()).powi(2).sin().powi(2);
        let b = (x1.sin() + x2.sin()).powi(2).cos().powi(2);
        let c = -(a - b + x1).powi(2).ln();
        let d = (x1 - 1.0).powi(2) + (x2 - 1.0).powi(2);
        c + 0.1 * d
    }
}

#[derive(Debug)]
pub struct Ned01;
impl TestFunction for Ned01 {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 2, ErrorKind::InvalidInput);
        Ok(iter::repeat((-10.0, 10.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        (xs[0].powi(2) + xs[1]).abs().sqrt().cos().abs().powf(0.5) + 0.01 * xs[0] + 0.01 * xs[1]
    }
}

#[derive(Debug)]
pub struct OddSquare;
impl TestFunction for OddSquare {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 2, ErrorKind::InvalidInput);
        Ok(iter::repeat((-3.0 * PI, 3.0 * PI)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let dim = xs.len() as f64;
        let b = [1.0, 1.3];
        let d = dim * (xs[0] - b[0]).max(xs[1] - b[1]).powi(2);
        let h = (xs[0] - b[0]).powi(2) + (xs[1] - b[1]).powi(2);
        -(-d / (2.0 * PI)).exp() * (PI * d).cos() * (1.0 + 0.02 * h / (d + 0.01))
    }
}

#[derive(Debug)]
pub struct Parsopoulos;
impl TestFunction for Parsopoulos {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 2, ErrorKind::InvalidInput);
        Ok(iter::repeat((-5.0, 5.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        xs[0].cos().powi(2) + xs[1].sin().powi(2)
    }
}

#[derive(Debug)]
pub struct Pinter;
impl TestFunction for Pinter {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert!(dim > 1, ErrorKind::InvalidInput; dim);
        Ok(iter::repeat((-5.0, 2.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        (0..xs.len())
            .map(|i| {
                let x_i = xs[i];
                let x_mi = xs
                    .get(i.wrapping_sub(1))
                    .copied()
                    .unwrap_or_else(|| xs[xs.len() - 1]);
                let x_pi = xs.get(i + 1).copied().unwrap_or_else(|| xs[0]);
                let a = x_mi * x_i.sin() + x_pi.sin();
                let b = x_mi.powi(2) - 2.0 * x_i + 3.0 * x_pi - x_i.cos() + 1.0;
                let i = i as f64;
                (i + 1.0) * x_i.powi(2)
                    + 20.0 * (i + 1.0) * a.sin().powi(2)
                    + (i + 1.0) * (1.0 + (i + 1.0) * b.powi(2)).log10()
            })
            .sum()
    }
}

#[derive(Debug)]
pub struct Plateau;
impl TestFunction for Plateau {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        Ok(iter::repeat((-2.34, 5.12)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        30.0 + xs.iter().map(|&x| x.abs().floor()).sum::<f64>()
    }
}

#[derive(Debug)]
pub struct Powell;
impl TestFunction for Powell {
    fn default_dimension(&self) -> usize {
        4
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 4, ErrorKind::InvalidInput);
        Ok(iter::repeat((-4.0, 5.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let x1 = xs[0];
        let x2 = xs[1];
        let x3 = xs[2];
        let x4 = xs[3];
        (x1 + 10.0 * x2).powi(2)
            + 5.0 * (x3 - x4).powi(2)
            + (x2 - 2.0 * x3).powi(4)
            + 10.0 * (x1 - x4).powi(4)
    }
}

#[derive(Debug)]
pub struct Problem03;
impl TestFunction for Problem03 {
    fn default_dimension(&self) -> usize {
        1
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 1, ErrorKind::InvalidInput);
        Ok(iter::repeat((-10.0, 10.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let x = xs[0];
        -(1..6)
            .map(|k| k as f64)
            .map(|k| k * ((k + 1.0) * x + k).sin())
            .sum::<f64>()
    }
}

#[derive(Debug)]
pub struct Rastrigin;
impl TestFunction for Rastrigin {
    fn default_dimension(&self) -> usize {
        8
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 8, ErrorKind::InvalidInput);
        let a = [-5.0, -5.0, -2.0, -2.0, -5.0, -5.0, -2.0, -2.0];
        let b = [2.0, 2.0, 5.0, 5.0, 2.0, 2.0, 5.0, 5.0];
        Ok(a.iter().copied().zip(b.iter().copied()).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let dim = xs.len() as f64;
        10.0 * dim
            + xs.iter()
                .map(|&x| x * x - 10.0 * (2.0 * PI * x).cos())
                .sum::<f64>()
    }
}

#[derive(Debug)]
pub struct RosenbrockLog;
impl TestFunction for RosenbrockLog {
    fn default_dimension(&self) -> usize {
        11
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 11, ErrorKind::InvalidInput);
        Ok(vec![
            (-2.0, 2.0),
            (-2.0, 1.1),
            (0.5, 2.0),
            (-2.0, 2.0),
            (0.8, 2.0),
            (-2.0, 1.5),
            (-2.0, 2.0),
            (-2.0, 1.2),
            (0.7, 2.0),
            (-2.0, 2.0),
            (-2.0, 2.0),
        ])
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let a = xs
            .iter()
            .skip(1)
            .zip(xs.iter().take(xs.len() - 1))
            .map(|(x1, x2)| 100.0 * (x1 - x2.powi(2)).powi(2) + (1.0 - x2).powi(2))
            .sum::<f64>();
        (1.0 + a).ln()
    }
}

#[derive(Debug)]
pub struct Sargan;
impl TestFunction for Sargan {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert!(dim > 1, ErrorKind::InvalidInput; dim);
        Ok(iter::repeat((-2.0, 4.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let a = xs
            .iter()
            .skip(1)
            .zip(xs.iter())
            .map(|(&x1, &x0)| x0 * x1)
            .sum::<f64>();
        let dim = xs.len() as f64;
        xs.iter().map(|&x| dim * (x * x + 0.4 * a)).sum()
    }
}

#[derive(Debug)]
pub struct Schwefel20;
impl TestFunction for Schwefel20 {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        Ok(iter::repeat((-60.0, 100.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        xs.iter().map(|&x| x.abs()).sum()
    }
}

#[derive(Debug)]
pub struct Schwefel36;
impl TestFunction for Schwefel36 {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 2, ErrorKind::InvalidInput);
        Ok(iter::repeat((-0.0, 20.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let x1 = xs[0];
        let x2 = xs[1];
        -x1 * x2 * (72.0 - 2.0 * x1 - 2.0 * x2)
    }
}

#[derive(Debug)]
pub struct Shekel05;
impl TestFunction for Shekel05 {
    fn default_dimension(&self) -> usize {
        4
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 4, ErrorKind::InvalidInput);
        Ok(iter::repeat((-0.0, 10.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let a_mat = [
            [4, 4, 4, 4],
            [1, 1, 1, 1],
            [8, 8, 8, 8],
            [6, 6, 6, 6],
            [3, 7, 3, 7],
        ];
        let c_vec = &[0.1, 0.2, 0.2, 0.4, 0.6];
        -a_mat
            .iter()
            .zip(c_vec.iter())
            .map(|(a, &c)| {
                let b = xs
                    .iter()
                    .zip(a.iter())
                    .map(|(&x, &a)| (x - a as f64).powi(2))
                    .sum::<f64>();
                1.0 / (b + c)
            })
            .sum::<f64>()
    }
}

#[derive(Debug)]
pub struct Shekel07;
impl TestFunction for Shekel07 {
    fn default_dimension(&self) -> usize {
        4
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 4, ErrorKind::InvalidInput);
        Ok(iter::repeat((-0.0, 10.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let a_mat = [
            [4, 4, 4, 4],
            [1, 1, 1, 1],
            [8, 8, 8, 8],
            [6, 6, 6, 6],
            [3, 7, 3, 7],
            [2, 9, 2, 9],
            [5, 5, 3, 3],
        ];
        let c_vec = [0.1, 0.2, 0.2, 0.4, 0.4, 0.6, 0.3];

        -a_mat
            .iter()
            .zip(c_vec.iter())
            .map(|(a, &c)| {
                let b = xs
                    .iter()
                    .zip(a.iter())
                    .map(|(&x, &a)| (x - a as f64).powi(2))
                    .sum::<f64>();
                1.0 / (b + c)
            })
            .sum::<f64>()
    }
}

#[derive(Debug)]
pub struct SixHumpCamel;
impl TestFunction for SixHumpCamel {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 2, ErrorKind::InvalidInput);
        Ok(vec![(-2.0, 2.0), (-1.5, 1.5)])
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let x1 = xs[0];
        let x2 = xs[1];
        (4.0 - 2.1 * x1.powi(2) + x1.powi(4) / 3.0) * x1.powi(2)
            + x1 * x2
            + (4.0 * x2.powi(2) - 4.0) * x2.powi(2)
    }
}

#[derive(Debug)]
pub struct Sphere;
impl TestFunction for Sphere {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        Ok(iter::repeat((-5.12, 2.12)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        xs.iter().map(|&x| x * x).sum()
    }
}

#[derive(Debug)]
pub struct StyblinskiTang;
impl TestFunction for StyblinskiTang {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        Ok(iter::repeat((-5.0, 5.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        xs.iter()
            .map(|&x| x.powi(4) - 16.0 * x.powi(2) + 5.0 * x)
            .sum::<f64>()
            / 2.0
    }
}

#[derive(Debug)]
pub struct Trid;
impl TestFunction for Trid {
    fn default_dimension(&self) -> usize {
        6
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 6, ErrorKind::InvalidInput);
        Ok(iter::repeat((0.0, 20.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let a = xs.iter().map(|&x| (x - 1.0).powi(2)).sum::<f64>();
        let b = xs
            .iter()
            .skip(1)
            .zip(xs.iter())
            .map(|(&x0, &x1)| x0 * x1)
            .sum::<f64>();
        a - b
    }
}

#[derive(Debug)]
pub struct Tripod;
impl TestFunction for Tripod {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 2, ErrorKind::InvalidInput);
        Ok(iter::repeat((-100.0, 100.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let x1 = xs[0];
        let x2 = xs[1];
        let p1 = if x1 >= 0.0 { 1.0 } else { 0.0 };
        let p2 = if x2 >= 0.0 { 1.0 } else { 0.0 };
        p2 * (1.0 + p1)
            + (x1 + 50.0 * p2 * (1.0 - 2.0 * p1)).abs()
            + (x2 + 50.0 * (1.0 - 2.0 * p2)).abs()
    }
}

#[derive(Debug)]
pub struct Weierstrass;
impl TestFunction for Weierstrass {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        Ok(iter::repeat((-0.5, 0.2)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let a = 0.5f64;
        let b = 3.0f64;
        let kmax = 20;
        let ak = (0..=kmax).map(|k| a.powi(k)).collect::<Vec<_>>();
        let bk = (0..=kmax).map(|k| b.powi(k)).collect::<Vec<_>>();
        let dim = xs.len() as f64;
        let c = ak
            .iter()
            .zip(bk.iter())
            .map(|(&a, &b)| a * (PI * b).cos())
            .sum::<f64>();

        xs.iter()
            .map(|&x| {
                let d = ak
                    .iter()
                    .zip(bk.iter())
                    .map(|(&a, &b)| a * (2.0 * PI * b * (x + 0.5)).cos())
                    .sum::<f64>();
                d - dim * c
            })
            .sum()
    }
}

#[derive(Debug)]
pub struct Xor;
impl TestFunction for Xor {
    fn default_dimension(&self) -> usize {
        9
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        track_assert_eq!(dim, 9, ErrorKind::InvalidInput);
        Ok(iter::repeat((-1.0, 1.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let f_11 = xs[6] / (1.0 + (-xs[0] - xs[1] - xs[4]).exp());
        let f_12 = xs[7] / (1.0 + (-xs[2] - xs[3] - xs[5]).exp());
        let f_1 = (1.0 + (-f_11 - f_12 - xs[8]).exp()).powi(-2);
        let f_21 = xs[6] / (1.0 + (-xs[4]).exp());
        let f_22 = xs[7] / (1.0 + (-xs[5]).exp());
        let f_2 = (1.0 + (-f_21 - f_22 - xs[8]).exp()).powi(-2);
        let f_31 = xs[6] / (1.0 + (-xs[0] - xs[4]).exp());
        let f_32 = xs[7] / (1.0 + (-xs[2] - xs[5]).exp());
        let f_3 = (1.0 - (1.0 + (-f_31 - f_32 - xs[8]).exp()).powi(-1)).powi(2);
        let f_41 = xs[6] / (1.0 + (-xs[1] - xs[4]).exp());
        let f_42 = xs[7] / (1.0 + (-xs[3] - xs[5]).exp());
        let f_4 = (1.0 - (1.0 + (-f_41 - f_42 - xs[8]).exp()).powi(-1)).powi(2);
        f_1 + f_2 + f_3 + f_4
    }
}

#[derive(Debug)]
pub struct YaoLiu;
impl TestFunction for YaoLiu {
    fn default_dimension(&self) -> usize {
        2
    }

    fn bounds(&self, dim: usize) -> Result<Vec<(f64, f64)>> {
        Ok(iter::repeat((-5.12, 2.0)).take(dim).collect())
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        xs.iter()
            .map(|&x| x.powi(2) - 10.0 * (2.0 * PI * x).cos() + 10.0)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ackley_works() {
        assert_eq!(Ackley.evaluate(&[1.0]), 3.6253849384403627);
        assert_eq!(Ackley.evaluate(&[1.0, 2.0]), 5.422131717799509);
        assert_eq!(Ackley.evaluate(&[1.0, 2.0, 3.0]), 7.0164536082694);
    }

    #[test]
    fn adjiman_works() {
        assert_eq!(Adjiman.evaluate(&[1.0, 2.0]), 0.2912954964338819);
    }

    #[test]
    fn alpine02_works() {
        assert_eq!(Alpine02.evaluate(&[1.2, 3.4]), -0.48108849403215925);
    }

    #[test]
    fn branin02_works() {
        assert_eq!(Branin02.evaluate(&[1.2, 3.4]), 10.042847240794726);
    }

    #[test]
    fn bukin06_works() {
        assert_eq!(Bukin06.evaluate(&[-7.2, 1.4]), 93.92155675444401);
    }

    #[test]
    fn carromtable_works() {
        assert_eq!(CarromTable.evaluate(&[1.2, 3.4]), -0.005496687092849093);
    }

    #[test]
    fn csendes_works() {
        assert_eq!(Csendes.evaluate(&[0.12]), 8.62141401006505e-06);
        assert_eq!(Csendes.evaluate(&[0.12, 0.34]), 0.0034057655846648);
        assert_eq!(Csendes.evaluate(&[0.12, 0.34, 0.56]), 0.09521917311547695);
    }

    #[test]
    fn deb02_works() {
        assert_eq!(Deb02.evaluate(&[0.12]), -0.08467457488005439);
        assert_eq!(Deb02.evaluate(&[0.12, 0.34]), -0.04233737264763648);
        assert_eq!(Deb02.evaluate(&[0.12, 0.34, 0.56]), -0.02822491681864048);
    }

    #[test]
    fn deflectedcorrugatedspring_works() {
        assert_eq!(
            DeflectedCorrugatedSpring.evaluate(&[1.2]),
            0.45529538181333074
        );
        assert_eq!(
            DeflectedCorrugatedSpring.evaluate(&[1.2, 3.4]),
            1.893939078387994
        );
    }

    #[test]
    fn easom_works() {
        assert_eq!(Easom.evaluate(&[1.2]), 5.62363908902924);
        assert_eq!(Easom.evaluate(&[1.2, 3.4]), 9.928391855906339);
    }

    #[test]
    fn exponential_works() {
        assert_eq!(Exponential.evaluate(&[0.12]), -0.9928258579038134);
        assert_eq!(Exponential.evaluate(&[0.12, -0.34]), -0.9370674633774034);
    }

    #[test]
    fn hartmann3_works() {
        assert_eq!(Hartmann3.evaluate(&[0.12, 0.34, 0.56]), -0.5775714789099738);
        assert_eq!(
            Hartmann3.evaluate(&[0.47, 0.01, 0.98]),
            -0.12200108040740279
        );
        assert_eq!(
            Hartmann3.evaluate(&[0.1, 0.55592003, 0.85218259]),
            -3.8626347486217725
        );
    }

    #[test]
    fn hartmann6_works() {
        assert_eq!(
            Hartmann6.evaluate(&[
                0.20168952, 0.15001069, 0.47687398, 0.27533243, 0.31165162, 0.65730054
            ]),
            -3.3223680114155116
        );
        assert_eq!(
            Hartmann6.evaluate(&[0.1, 0.2, 0.3, 0.4, 0.5, 0.6]),
            -1.4069105761385299
        );
    }

    #[test]
    fn helicalvalley_works() {
        assert_eq!(HelicalValley.evaluate(&[1.0, 0.0, 0.0]), 0.0);
        assert_eq!(
            HelicalValley.evaluate(&[-0.12, 0.34, 0.56]),
            656.2430543456857
        );
    }

    #[test]
    fn himmelblau_works() {
        assert_eq!(HimmelBlau.evaluate(&[3.0, 2.0]), 0.0);
        assert_eq!(HimmelBlau.evaluate(&[0.1, 0.2]), 163.4837);
    }

    #[test]
    fn lennard_jones6_works() {
        assert_eq!(
            LennardJones6.evaluate(&[
                -2.66666470373,
                2.73904387714,
                1.42304625988,
                -1.95553276732,
                2.81714839844,
                2.12175295546
            ]),
            -1.0
        );
        assert_eq!(
            LennardJones6.evaluate(&[-0.12, 0.34, 0.56, 0.12, -0.34, -0.56]),
            -0.3259538442755606
        );
    }

    #[test]
    fn mccourt01_works() {
        assert_eq!(
            McCourt01.evaluate(&[0.6241, 0.7688, 0.8793, 0.2739, 0.7351, 0.8499, 0.6196]),
            -0.08594263064487956
        );
        assert_eq!(
            McCourt01.evaluate(&[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7]),
            1.5047377187821733
        );
    }

    #[test]
    fn mccourt02_works() {
        assert_eq!(
            McCourt02.evaluate(&[0.4068, 0.4432, 0.6479, 0.1978, 0.7660, 0.7553, 0.5640]),
            -2.7416211601265212
        );
        assert_eq!(
            McCourt02.evaluate(&[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7]),
            -2.3839412938903193
        );
    }

    #[test]
    fn mccourt03_works() {
        assert_eq!(
            McCourt03.evaluate(&[
                0.9317, 0.1891, 0.2503, 0.3646, 0.1603, 0.9829, 0.0392, 0.3263, 0.6523
            ]),
            -3.0237963611611614
        );
    }

    #[test]
    fn mccourt06_works() {
        assert_eq!(
            McCourt06.evaluate(&[1.0, 1.0, 0.7636, 0.5268, 1.0]),
            2.807202633080286
        );
    }

    #[test]
    fn mccourt07_works() {
        assert_eq!(
            McCourt07.evaluate(&[0.3811, 1.0, 0.2312, 0.0, 1.0, 0.1403]),
            -0.36321372791219586
        );
    }

    #[test]
    fn mccourt08_works() {
        assert_eq!(
            McCourt08.evaluate(&[0.5067, 1.0, 0.5591, 0.0823]),
            -3.450990971528579
        );
        assert_eq!(
            McCourt08.evaluate(&[0.1, 0.2, 0.3, 0.4]),
            -1.968579289153076
        );
    }

    #[test]
    fn mccourt09_works() {
        assert_eq!(
            McCourt09.evaluate(&[0.594, 1.0, 0.205]),
            -10.171467017140396
        );
    }

    #[test]
    fn mccourt10_works() {
        assert_eq!(
            McCourt10.evaluate(&[0.5085, 0.5433, 0.2273, 1.0, 0.3381, 0.0255, 1.0, 0.5038]),
            -2.519395956637841
        );
    }

    #[test]
    fn mccourt11_works() {
        assert_eq!(
            McCourt11.evaluate(&[0.4, 0.6, 0.4, 1.0, 0.4, 0.2, 1.0, 0.3]),
            -0.3904553256458163
        );
    }

    #[test]
    fn mccourt12_works() {
        assert_eq!(
            McCourt12.evaluate(&[0.4499, 0.4553, 0.0046, 1.0, 0.3784, 0.3067, 0.6173]),
            3.5427498479589827
        );
    }

    #[test]
    fn mccourt13_works() {
        assert_eq!(McCourt13.evaluate(&[1.0, 1.0, 1.0]), 1.490482963587614);
    }

    #[test]
    fn mccourt14_works() {
        assert_eq!(McCourt14.evaluate(&[0.1, 0.8, 0.3]), -5.0);
    }

    #[test]
    fn mccourt16_works() {
        assert_eq!(
            McCourt16.evaluate(&[0.1858, 0.6858, 0.1858, 0.4858]),
            -0.8422170093622552
        );
    }

    #[test]
    fn mccourt17_works() {
        assert_eq!(
            McCourt17.evaluate(&[0.3125, 0.9166, 0.3125, 0.7062, 0.0397, 0.9270, 0.5979]),
            0.4708920110382042
        );
    }

    #[test]
    fn mccourt18_works() {
        assert_eq!(
            McCourt18.evaluate(&[0.2677, 0.8696, 0.2677, 0.6594, 0.1322, 0.9543, 0.0577, 0.295]),
            -1.4290621963312253
        );
    }

    #[test]
    fn mccourt19_works() {
        assert_eq!(McCourt19.evaluate(&[0.4, 0.8]), -8.6726896031426);
    }

    #[test]
    fn mccourt20_works() {
        assert_eq!(McCourt20.evaluate(&[0.7, 0.1]), -6.5976366321635656);
    }

    #[test]
    fn mccourt22_works() {
        assert_eq!(
            McCourt22.evaluate(&[0.2723, 0.4390, 0.8277, 0.3390, 0.3695]),
            -3.080793651834803
        );
    }

    #[test]
    fn mccourt23_works() {
        assert_eq!(
            McCourt23.evaluate(&[0.7268, 0.3914, 0.0, 0.7268, 0.5375, 0.8229]),
            -18.357500661879723
        );
    }

    #[test]
    fn mccourt26_works() {
        assert_eq!(McCourt26.evaluate(&[0.5, 0.8, 0.3]), -1.5534975431190259);
    }

    #[test]
    fn mccourt27_works() {
        assert_eq!(McCourt27.evaluate(&[0.6, 0.3, 0.5]), -1.7690847025557281);
    }

    #[test]
    fn mccourt28_works() {
        assert_eq!(
            McCourt28.evaluate(&[0.4493, 0.0667, 0.9083, 0.2710]),
            -7.694326241956232
        );
    }

    #[test]
    fn michalewicz_works() {
        assert_eq!(
            Michalewicz.evaluate(&[0.4493, 0.0667, 0.9083, 0.2710]),
            -0.0008081861489932879
        );
    }

    #[test]
    fn mishra06_works() {
        assert_eq!(Mishra06.evaluate(&[2.88631, 1.82326]), -2.2839498384520884);
    }

    #[test]
    fn ned01_works() {
        assert_eq!(Ned01.evaluate(&[-8.4666, -9.9988]), -0.17894509347721144);
    }

    #[test]
    fn oddsquare_works() {
        assert_eq!(
            OddSquare.evaluate(&[0.912667308214834, 1.212667322565022]),
            -1.0084672831031778
        );
    }

    #[test]
    fn parsopoulos_works() {
        assert_eq!(
            Parsopoulos.evaluate(&[PI / 2.0, PI]),
            0.00000000000000000000000000000001874699728327322
        );
    }

    #[test]
    fn pinter_works() {
        assert_eq!(Pinter.evaluate(&[0.0, 0.0, 0.0]), 0.0);
        assert_eq!(Pinter.evaluate(&[-1.2, 0.34]), 43.14239703715357);
    }

    #[test]
    fn plateau_works() {
        assert_eq!(Plateau.evaluate(&[0.0, 0.0, 0.0]), 30.0);
        assert_eq!(Plateau.evaluate(&[-1.2, 0.34]), 31.0);
    }

    #[test]
    fn powell_works() {
        assert_eq!(Powell.evaluate(&[0.0, 0.0, 0.0, 0.0]), 0.0);
        assert_eq!(Powell.evaluate(&[-1.2, 3.4, 0.56, -0.78]), 1112.15253216);
    }

    #[test]
    fn problem03_works() {
        assert_eq!(Problem03.evaluate(&[-6.7745761]), -12.031249442166843);
    }

    #[test]
    fn rastrigin_works() {
        assert_eq!(
            Rastrigin.evaluate(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            0.0
        );
        assert_eq!(
            Rastrigin.evaluate(&[0.1, 0.2, -0.3, 0.4, 0.5, 0.6, -0.7, 0.0]),
            92.58033988749895
        );
    }

    #[test]
    fn rosenbrocklog_works() {
        assert_eq!(
            RosenbrockLog.evaluate(&iter::repeat(1.0).take(11).collect::<Vec<_>>()),
            0.0
        );
        assert_eq!(
            RosenbrockLog.evaluate(&[0.1, 0.2, -0.3, 0.4, 0.5, 0.6, -0.7, 0.8, 0.9, 1.0, 1.1]),
            5.245865506545613
        );
    }

    #[test]
    fn sargan_works() {
        assert_eq!(
            Sargan.evaluate(&iter::repeat(0.0).take(5).collect::<Vec<_>>()),
            0.0
        );
        assert_eq!(
            Sargan.evaluate(&[0.1, 0.2, -0.3, 0.4, 0.5]),
            3.1500000000000004
        );
    }

    #[test]
    fn schwefel20_works() {
        assert_eq!(
            Schwefel20.evaluate(&iter::repeat(0.0).take(5).collect::<Vec<_>>()),
            0.0
        );
        assert_eq!(Schwefel20.evaluate(&[0.1, 0.2, -0.3, 0.4, 0.5]), 1.5);
    }

    #[test]
    fn schwefel36_works() {
        assert_eq!(Schwefel36.evaluate(&[12.0, 12.0]), -3456.0);
    }

    #[test]
    fn shekel05_works() {
        assert_eq!(
            Shekel05.evaluate(&[4.0, 4.0, 4.0, 4.0]),
            -10.152719932456289
        );
    }

    #[test]
    fn shekel07_works() {
        assert_eq!(
            Shekel07.evaluate(&[4.0, 4.0, 4.0, 4.0]),
            -10.402818836930305
        );
    }

    #[test]
    fn sixhumpcamel_works() {
        assert_eq!(
            SixHumpCamel.evaluate(&[0.08984201368301331, -0.7126564032704135]),
            -1.0316284534898774
        );
    }

    #[test]
    fn sphere_works() {
        assert_eq!(Sphere.evaluate(&[0.1, 0.2, -0.3]), 0.14);
    }

    #[test]
    fn styblinskitang_works() {
        assert_eq!(
            StyblinskiTang.evaluate(&[-2.903534018185960, -2.903534018185960, -2.903534018185960]),
            -117.49849711131424
        );
    }

    #[test]
    fn trid_works() {
        assert_eq!(Trid.evaluate(&[6.0, 10.0, 12.0, 12.0, 10.0, 6.0]), -50.0);
    }

    #[test]
    fn tripod_works() {
        assert_eq!(Tripod.evaluate(&[0.0, -50.0]), 0.0);
    }

    #[test]
    fn weierstrass_works() {
        assert_eq!(Weierstrass.evaluate(&[0.1, 0.2, -0.3]), 17.127313481390843);
    }

    #[test]
    fn xor_works() {
        assert_eq!(
            Xor.evaluate(&[
                1.0,
                -1.0,
                1.0,
                -1.0,
                -1.0,
                1.0,
                1.0,
                -1.0,
                0.421457080713797
            ]),
            0.9597587570119619
        );
    }

    #[test]
    fn yaoliu_works() {
        assert_eq!(YaoLiu.evaluate(&[0.0, 0.0]), 0.0);
        assert_eq!(YaoLiu.evaluate(&[0.1, 0.2, -0.3, 0.4]), 40.3);
    }
}
