use crate::solver::SolverCapabilities;
use crate::{Error, ErrorKind, Result};
use rand::distributions::Distribution as RandDistribution;
use rand::Rng;
use rustats::num::FiniteF64;
use rustats::range::Range;
use serde::{Deserialize, Serialize};
use serde_json;
use std::convert::TryFrom;
use std::fmt::Display;
use std::str::FromStr;

// TODO:
// pub struct ParamDomain {
//    condition: Vec<Condition>,
//    param: Unconditional
// }
// pub enum UnconditionalValue {
//     Continuous(FiniteF64),
//     Discrete(i64),
//     Categorical(usize),
// }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ParamDomain {
    Continuous(Continuous),
    Discrete(Discrete),
    Categorical(Categorical),
    Conditional(Conditional),
}
impl ParamDomain {
    pub fn name(&self) -> &str {
        match self {
            ParamDomain::Continuous(p) => &p.name,
            ParamDomain::Discrete(p) => &p.name,
            ParamDomain::Categorical(p) => &p.name,
            ParamDomain::Conditional(p) => p.param.name(),
        }
    }

    // TODO:
    pub fn range(&self) -> Range<f64> {
        match self {
            ParamDomain::Continuous(p) => {
                Range::new(p.range.low.get(), p.range.high.get()).unwrap()
            }
            ParamDomain::Discrete(p) => {
                Range::new(p.range.low as f64, p.range.high as f64 - 1.0).unwrap()
            }
            ParamDomain::Categorical(p) => Range::new(0.0, p.choices.len() as f64 - 1.0).unwrap(),
            ParamDomain::Conditional(p) => unimplemented!("{:?}", p),
        }
    }

    pub fn required_solver_capabilities(&self) -> SolverCapabilities {
        let mut c = SolverCapabilities::empty();
        match self {
            ParamDomain::Continuous(p) => {
                if p.distribution == Distribution::LogUniform {
                    c = c.log_uniform();
                }
            }
            ParamDomain::Discrete(_) => {
                c = c.discrete();
            }
            ParamDomain::Categorical(_) => {
                c = c.categorical();
            }
            ParamDomain::Conditional(p) => {
                c = c
                    .conditional()
                    .union(p.param.required_solver_capabilities());
            }
        }
        c
    }
}

// TODO:
// pub struct ParamValue(Optiona<UnconditionalValue>);
// pub enum UnconditionalValue {
//     Continuous(FiniteF64),
//     Discrete(i64),
//     Categorical(usize),
// }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ParamValue {
    Continuous(FiniteF64),
    Discrete(i64),
    Categorical(usize),                   // TODO: String(?)
    Conditional(Option<Box<ParamValue>>), // TODO: Conditional(Option<Box<UnconditionalValue>>)
}
impl ParamValue {
    pub fn to_json_value(&self) -> Result<serde_json::Value> {
        match self {
            ParamValue::Continuous(v) => Ok(serde_json::Value::Number(track_assert_some!(
                serde_json::Number::from_f64(v.get()),
                ErrorKind::InvalidInput
            ))),
            ParamValue::Discrete(v) => Ok(serde_json::Value::Number(serde_json::Number::from(*v))),
            ParamValue::Categorical(index) => {
                Ok(serde_json::Value::Number(serde_json::Number::from(*index)))
            }
            ParamValue::Conditional(None) => Ok(serde_json::Value::Null),
            ParamValue::Conditional(Some(v)) => track!(v.to_json_value()),
        }
    }

    pub fn to_f64(&self) -> f64 {
        use std::f64::NAN;

        match self {
            ParamValue::Continuous(v) => v.get(),
            ParamValue::Discrete(v) => *v as f64,
            ParamValue::Categorical(index) => *index as f64,
            ParamValue::Conditional(None) => NAN,
            ParamValue::Conditional(Some(v)) => v.to_f64(),
        }
    }

    pub fn try_map<F>(self, f: F) -> Result<Self>
    where
        F: FnOnce(Self) -> Result<Self>,
    {
        match self {
            ParamValue::Conditional(None) => Ok(ParamValue::Conditional(None)),
            ParamValue::Conditional(Some(p)) => {
                track!(f(*p)).map(|p| ParamValue::Conditional(Some(Box::new(p))))
            }
            p => track!(f(p)),
        }
    }

    pub fn as_discrete(&self) -> Option<i64> {
        if let ParamValue::Discrete(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_continuous(&self) -> Option<FiniteF64> {
        if let ParamValue::Continuous(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_categorical(&self) -> Option<usize> {
        if let ParamValue::Categorical(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn try_to_string(&self) -> Option<String> {
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Unconditional {
    Continuous(Continuous),
    Discrete(Discrete),
    Categorical(Categorical),
}
impl Unconditional {
    pub fn name(&self) -> &str {
        match self {
            Unconditional::Continuous(p) => &p.name,
            Unconditional::Discrete(p) => &p.name,
            Unconditional::Categorical(p) => &p.name,
        }
    }

    pub fn required_solver_capabilities(&self) -> SolverCapabilities {
        let mut c = SolverCapabilities::empty();
        match self {
            Unconditional::Continuous(p) => {
                if p.distribution == Distribution::LogUniform {
                    c = c.log_uniform();
                }
            }
            Unconditional::Discrete(_) => {
                c = c.discrete();
            }
            Unconditional::Categorical(_) => {
                c = c.categorical();
            }
        }
        c
    }

    pub fn to_domain(&self) -> ParamDomain {
        match self {
            Unconditional::Continuous(p) => ParamDomain::Continuous(p.clone()),
            Unconditional::Discrete(p) => ParamDomain::Discrete(p.clone()),
            Unconditional::Categorical(p) => ParamDomain::Categorical(p.clone()),
        }
    }
}
impl TryFrom<ParamDomain> for Unconditional {
    type Error = Error;

    fn try_from(f: ParamDomain) -> Result<Self> {
        Ok(match f {
            ParamDomain::Categorical(p) => Unconditional::Categorical(p),
            ParamDomain::Conditional(_) => track_panic!(ErrorKind::InvalidInput),
            ParamDomain::Continuous(p) => Unconditional::Continuous(p),
            ParamDomain::Discrete(p) => Unconditional::Discrete(p),
        })
    }
}
impl RandDistribution<ParamValue> for Unconditional {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ParamValue {
        match self {
            Unconditional::Continuous(p) => {
                let v = if p.distribution == Distribution::LogUniform {
                    let low = 1.0;
                    let high = (p.range.high.get() - p.range.low.get()).exp();
                    let v = rng.gen_range(low, high);
                    v.ln() + p.range.low.get()
                } else {
                    rng.gen_range(p.range.low.get(), p.range.high.get())
                };
                ParamValue::Continuous(FiniteF64::new(v).unwrap_or_else(|e| unreachable!("{}", e)))
            }
            Unconditional::Discrete(p) => {
                ParamValue::Discrete(rng.gen_range(p.range.low, p.range.high))
            }
            Unconditional::Categorical(p) => {
                ParamValue::Categorical(rng.gen_range(0, p.choices.len()))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Continuous {
    pub name: String,
    pub range: Range<FiniteF64>,

    #[serde(default)]
    pub distribution: Distribution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Discrete {
    pub name: String,
    pub range: Range<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Categorical {
    pub name: String,
    pub choices: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conditional {
    pub condition: Condition,
    pub param: Box<Unconditional>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Condition {
    // can refer to only preceeding parameters
    Member { name: String, choices: Vec<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")] // TODO: remove kebab-case
pub enum Distribution {
    Uniform,
    LogUniform,
}
impl Distribution {
    pub fn is_uniform(&self) -> bool {
        *self == Distribution::Uniform
    }
    // TODO: possible-values
}
impl Default for Distribution {
    fn default() -> Self {
        Distribution::Uniform
    }
}
impl FromStr for Distribution {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "uniform" => Ok(Distribution::Uniform),
            "loguniform" => Ok(Distribution::LogUniform),
            _ => track_panic!(ErrorKind::InvalidInput, "Unknown distribution: {:?}", s),
        }
    }
}

pub fn when(condition: Condition, param: ParamDomain) -> Result<ParamDomain> {
    let param = Box::new(track!(Unconditional::try_from(param))?);
    Ok(ParamDomain::Conditional(Conditional { condition, param }))
}

pub fn category_eq(name: &str, value: &str) -> Condition {
    Condition::Member {
        name: name.to_owned(),
        choices: vec![value.to_owned()],
    }
}

pub fn boolean(name: &str) -> ParamDomain {
    choices(name, &["false", "true"])
}

pub fn choices<I, C>(name: &str, choices: I) -> ParamDomain
where
    I: IntoIterator<Item = C>,
    C: Display,
{
    ParamDomain::Categorical(Categorical {
        name: name.to_owned(),
        choices: choices.into_iter().map(|c| c.to_string()).collect(),
    })
}

pub fn uniform(name: &str, low: f64, high: f64) -> Result<ParamDomain> {
    let low = track!(FiniteF64::new(low))?;
    let high = track!(FiniteF64::new(high))?;
    let range = track!(Range::new(low, high))?;
    Ok(ParamDomain::Continuous(Continuous {
        name: name.to_owned(),
        range,
        distribution: Distribution::Uniform,
    }))
}

pub fn log_uniform(name: &str, low: f64, high: f64) -> Result<ParamDomain> {
    let low = track!(FiniteF64::new(low))?;
    let high = track!(FiniteF64::new(high))?;
    let range = track!(Range::new(low, high))?;
    Ok(ParamDomain::Continuous(Continuous {
        name: name.to_owned(),
        range,
        distribution: Distribution::LogUniform,
    }))
}

pub fn int(name: &str, low: i64, high: i64) -> Result<ParamDomain> {
    let range = track!(Range::new(low, high))?;
    Ok(ParamDomain::Discrete(Discrete {
        name: name.to_owned(),
        range,
    }))
}
