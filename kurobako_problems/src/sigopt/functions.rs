use kurobako_core::{ErrorKind, Result};
use std::f64::consts::PI;
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

// pub fn adjiman(xs: &[f64]) -> Result<f64> {
//     track_assert_eq!(xs.len(), 2, ErrorKind::InvalidInput);
// }

// recipe(Alpine02, 2, None),
// recipe(Branin02, 2, None),
// recipe(Bukin06, 2, None),
// recipe(CarromTable, 2, None),
// recipe(Csendes, 2, None),
// recipe(Deb02, 6, None),
// recipe(DeflectedCorrugatedSpring, 4, None),
// recipe(Easom, 2, None),
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
}
