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
// recipe(Exponential, 6, None),
// recipe(Hartmann3, 3, None),
// recipe(Hartmann6, 6, Some(10.0)),
// recipe(HelicalValley, 3, None),
// recipe(LennardJones6, 6, None),
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
}
