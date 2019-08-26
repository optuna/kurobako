use kurobako_core::{ErrorKind, Result};
use rand::distributions::Distribution;
use rand::Rng;
use rand_distr::Cauchy;

/// https://coco.gforge.inria.fr/downloads/download16.00/bbobdocfunctions.pdf
#[derive(Debug)]
pub struct TestFunction {
    opt_xs: Vec<f64>,
    opt_y: f64,
}
impl TestFunction {
    pub fn new(opt_xs: Vec<f64>, opt_y: f64) -> Result<Self> {
        for &x in &opt_xs {
            track_assert!(-5.0 <= x && x < 5.0, ErrorKind::InvalidInput; x);
        }
        track_assert!(-1000.0 <= opt_y && opt_y <= 1000.0, ErrorKind::InvalidInput; opt_y);

        Ok(Self { opt_xs, opt_y })
    }

    pub fn new_random<R: Rng + ?Sized>(rng: &mut R, dim: usize) -> Self {
        let opt_xs = (0..dim).map(|_| rng.gen_range(-4.0, 4.0)).collect();

        let cauchy = Cauchy::new(0.0, 100.0).unwrap_or_else(|e| unreachable!("{:?}", e));
        let opt_y: f64 = cauchy.sample(rng);
        let opt_y = (opt_y * 100.0).round() / 100.0;
        let opt_y = opt_y.min(1000.0).max(-1000.0);
        Self { opt_xs, opt_y }
    }
}

// [1.2 Ellipsoidal Function](https://coco.gforge.inria.fr/downloads/download16.00/bbobdocfunctions.pdf#page=10)
// pub fn ellipsoidal(xs: &[f64]) -> f64 {
//     //let d = xs.len() as f64;
//     panic!()
// }
