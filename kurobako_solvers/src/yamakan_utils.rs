use kurobako_core::domain::{Distribution, Range};
use kurobako_core::trial::IdGen;
use rand::Rng;

#[derive(Debug)]
pub struct YamakanIdGen<'a>(pub &'a mut IdGen);

impl<'a> yamakan::IdGen for YamakanIdGen<'a> {
    fn generate(&mut self) -> Result<yamakan::ObsId, yamakan::Error> {
        Ok(yamakan::ObsId::new(self.0.generate().get()))
    }
}

#[derive(Debug)]
pub struct KurobakoDomain {
    range: Range,
    distribution: Distribution,
}

impl KurobakoDomain {
    pub fn new(range: Range, distribution: Distribution) -> Self {
        Self {
            range,
            distribution,
        }
    }
}

impl yamakan::Domain for KurobakoDomain {
    type Point = f64;
}

impl rand::distributions::Distribution<f64> for KurobakoDomain {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        match &self.range {
            Range::Continuous { low, high } => match self.distribution {
                Distribution::Uniform => rng.gen_range(low, high),
                Distribution::LogUniform => rng.gen_range(low.log2(), high.log2()).exp2(),
            },
            Range::Discrete { low, high } => match self.distribution {
                Distribution::Uniform => rng.gen_range(low, high) as f64,
                Distribution::LogUniform => rng
                    .gen_range((*low as f64).log2(), (*high as f64).log2())
                    .exp2()
                    .floor(),
            },
            Range::Categorical { choices } => rng.gen_range(0, choices.len()) as f64,
        }
    }
}
