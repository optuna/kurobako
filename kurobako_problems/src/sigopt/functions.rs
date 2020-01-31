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
struct Alpine02;
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
// recipe(McCourt01, 7, Some(10.0)),
// recipe(McCourt02, 7, None),
// recipe(McCourt03, 9, None),
// recipe(McCourt06, 5, Some(12.0)),
// recipe(McCourt07, 6, Some(12.0)),
// recipe(McCourt08, 4, None),
// recipe(McCourt09, 3, None),
// recipe(McCourt10, 8, None),
// recipe(McCourt11, 8, None),
// recipe(McCourt12, 7, None),
// recipe(McCourt13, 3, None),
// recipe(McCourt14, 3, None),
// recipe(McCourt16, 4, Some(10.0)),
// recipe(McCourt17, 7, None),
// recipe(McCourt18, 8, None),
// recipe(McCourt19, 2, None),
// recipe(McCourt20, 2, None),
// recipe(McCourt22, 5, None),
// recipe(McCourt23, 6, None),
// recipe(McCourt26, 3, None),
// recipe(McCourt27, 3, None),
// recipe(McCourt28, 4, None),
// recipe(Michalewicz, 4, None),
// recipe(Mishra06, 2, None),
// recipe(Ned01, 2, None),
// recipe(OddSquare, 2, None),
// recipe(Parsopoulos, 2, None),
// recipe(Pinter, 2, None),
// recipe(Plateau, 2, None),
// recipe(Problem03, 1, None),
// recipe(Rastrigin, 8, None),
// recipe(RosenbrockLog, 11, None),
// recipe(Sargan, 5, None),
// recipe(Schwefel20, 2, None),
// recipe(Schwefel36, 2, None),
// recipe(Shekel05, 4, None),
// recipe(Shekel07, 4, None),
// recipe(Sphere, 7, None),
// recipe(StyblinskiTang, 5, None),
// recipe(Trid, 6, None),
// recipe(Tripod, 2, None),
// recipe(Weierstrass, 3, None),
// recipe(Xor, 9, None),
// recipe(YaoLiu, 5, None),

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
}
