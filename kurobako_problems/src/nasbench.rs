use kurobako_core::parameter::{self, choices, uniform, ParamDomain, ParamValue};
use kurobako_core::problem::{
    Evaluate, EvaluatorCapability, Problem, ProblemRecipe, ProblemSpec, Values,
};
use kurobako_core::{Error, ErrorKind, Result};
use nasbench::{AdjacencyMatrix, ModelSpec, NasBench, Op};
use rustats::num::FiniteF64;
use rustats::range::MinMax;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

const MAX_EDGES: usize = 9;
const VERTICES: usize = 7;
const EDGE_KINDS: usize = VERTICES * (VERTICES - 1) / 2;

// https://github.com/automl/nas_benchmarks/blob/c1bae6632bf15d45ba49c269c04dbbeb3f0379f0/tabular_benchmarks/nas_cifar10.py
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Encoding {
    A,
    B,
    C,
}
impl Encoding {
    const POSSIBLE_VALUES: [&'static str; 3] = ["A", "B", "C"];

    fn ops_and_edges(&self, params: &[ParamValue]) -> Result<(Vec<Op>, HashSet<usize>)> {
        let mut ops = vec![Op::Input];
        for p in &params[0..5] {
            let op = match p {
                ParamValue::Categorical(0) => Op::Conv1x1,
                ParamValue::Categorical(1) => Op::Conv3x3,
                ParamValue::Categorical(2) => Op::MaxPool3x3,
                _ => track_panic!(ErrorKind::InvalidInput, "Unexpected parameter: {:?}", p),
            };
            ops.push(op);
        }
        ops.push(Op::Output);

        let edges = track!(self.edges(&params[5..]))?;
        Ok((ops, edges))
    }

    fn edges(&self, params: &[ParamValue]) -> Result<HashSet<usize>> {
        match self {
            Encoding::A => track!(Self::edges_a(params)),
            Encoding::B => track!(Self::edges_b(params)),
            Encoding::C => track!(Self::edges_c(params)),
        }
    }

    fn edges_a(params: &[ParamValue]) -> Result<HashSet<usize>> {
        let mut edges = HashSet::new();
        for (i, p) in params.iter().enumerate() {
            let b = track_assert_some!(p.as_categorical(), ErrorKind::InvalidInput);
            if b == 1 {
                edges.insert(i);
            }
        }
        Ok(edges)
    }

    fn edges_b(params: &[ParamValue]) -> Result<HashSet<usize>> {
        let mut edges = HashSet::new();
        for p in params {
            let i = track_assert_some!(p.as_categorical(), ErrorKind::InvalidInput);
            edges.insert(i);
        }
        Ok(edges)
    }

    fn edges_c(params: &[ParamValue]) -> Result<HashSet<usize>> {
        let num_edges =
            track_assert_some!(params[0].as_discrete(), ErrorKind::InvalidInput) as usize;

        let mut edges = Vec::new();
        for (i, p) in params[1..].iter().enumerate() {
            if let ParamValue::Continuous(v) = p {
                edges.push((v, i));
            } else {
                track_panic!(ErrorKind::InvalidInput, "Unexpected parameter: {:?}", p);
            }
        }
        assert_eq!(edges.len(), EDGE_KINDS);

        edges.sort();
        Ok(edges
            .iter()
            .rev()
            .take(num_edges)
            .map(|t| t.1)
            .collect::<HashSet<_>>())
    }

    fn params_domain(&self) -> Vec<ParamDomain> {
        match self {
            Encoding::A => Self::params_domain_a(),
            Encoding::B => Self::params_domain_b(),
            Encoding::C => Self::params_domain_c(),
        }
    }

    fn common_params_domain() -> Vec<ParamDomain> {
        let mut params_domain = Vec::new();
        for i in 0..5 {
            params_domain.push(choices(
                &format!("op{}", i),
                &["conv1x1-bn-relu", "conv3x3-bn-relu", "maxpool3x3"],
            ));
        }
        params_domain
    }

    fn params_domain_a() -> Vec<ParamDomain> {
        let mut params_domain = Self::common_params_domain();
        for i in 0..EDGE_KINDS {
            params_domain.push(parameter::boolean(&format!("edge{}", i)));
        }
        params_domain
    }

    fn params_domain_b() -> Vec<ParamDomain> {
        let mut params_domain = Self::common_params_domain();

        for i in 0..MAX_EDGES {
            params_domain.push(choices(&format!("edge{}", i), 0..EDGE_KINDS));
        }

        params_domain
    }

    fn params_domain_c() -> Vec<ParamDomain> {
        let mut params_domain = Self::common_params_domain();

        params_domain.push(
            parameter::int("num_edges", 0, MAX_EDGES as i64 + 1)
                .unwrap_or_else(|e| unreachable!("{}", e)),
        );
        for i in 0..EDGE_KINDS {
            params_domain.push(
                uniform(&format!("edge{}", i), 0.0, 1.0).unwrap_or_else(|e| unreachable!("{}", e)),
            );
        }
        params_domain
    }
}
impl FromStr for Encoding {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "A" => Ok(Encoding::A),
            "B" => Ok(Encoding::B),
            "C" => Ok(Encoding::C),
            _ => track_panic!(ErrorKind::InvalidInput, "Unknown encoding: {:?}", s),
        }
    }
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct NasbenchProblemRecipe {
    pub dataset_path: PathBuf,

    #[structopt(
        long,
        default_value = "C",
        raw(possible_values = "&Encoding::POSSIBLE_VALUES")
    )]
    pub encoding: Encoding,
}
impl ProblemRecipe for NasbenchProblemRecipe {
    type Problem = NasbenchProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        let nasbench = track!(NasBench::new(&self.dataset_path))?;
        Ok(NasbenchProblem {
            nasbench: Rc::new(nasbench),
            encoding: self.encoding,
        })
    }
}

#[derive(Debug)]
pub struct NasbenchProblem {
    nasbench: Rc<NasBench>,
    encoding: Encoding,
}
impl Problem for NasbenchProblem {
    type Evaluator = NasbenchEvaluator;

    fn specification(&self) -> ProblemSpec {
        ProblemSpec {
            name: format!("NASBench{:?}", self.encoding),
            version: None,
            params_domain: self.encoding.params_domain(),
            values_domain: unsafe {
                vec![MinMax::new_unchecked(
                    FiniteF64::new_unchecked(0.0),
                    FiniteF64::new_unchecked(1.0),
                )]
            },
            evaluation_expense: unsafe { NonZeroU64::new_unchecked(108) },
            capabilities: vec![EvaluatorCapability::Concurrent].into_iter().collect(),
        }
    }

    fn create_evaluator(&mut self, id: ObsId) -> Result<Self::Evaluator> {
        Ok(NasbenchEvaluator {
            nasbench: self.nasbench.clone(),
            encoding: self.encoding,
            sample_index: id.get() as usize,
        })
    }
}

#[derive(Debug)]
pub struct NasbenchEvaluator {
    nasbench: Rc<NasBench>,
    encoding: Encoding,
    sample_index: usize,
}
impl Evaluate for NasbenchEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Values> {
        let (ops, edges) = track!(self.encoding.ops_and_edges(params))?;
        let edge = |i| edges.contains(&i);

        let matrix = vec![
            vec![false, edge(0), edge(1), edge(2), edge(3), edge(4), edge(5)],
            vec![false, false, edge(6), edge(7), edge(8), edge(9), edge(10)],
            vec![false, false, false, edge(11), edge(12), edge(13), edge(14)],
            vec![false, false, false, false, edge(15), edge(16), edge(17)],
            vec![false, false, false, false, false, edge(18), edge(19)],
            vec![false, false, false, false, false, false, edge(20)],
            vec![false, false, false, false, false, false, false],
        ];
        let adjacency = track!(AdjacencyMatrix::new(matrix))?;

        let model_spec = track!(ModelSpec::new(ops, adjacency))?;
        if let Some(model) = self.nasbench.models().get(&model_spec) {
            use std::collections::Bound;

            let epoch_num = budget.amount as u8;
            let (consumption, epoch_candidates) = track_assert_some!(
                model
                    .epochs
                    .range((Bound::Included(epoch_num), Bound::Unbounded))
                    .nth(0),
                ErrorKind::InvalidInput
            );
            let epoch = &epoch_candidates[self.sample_index % epoch_candidates.len()];
            budget.consumption = *consumption as u64;

            let value = 1.0 - epoch.complete.validation_accuracy;
            Ok(vec![track!(FiniteF64::new(value))?])
        } else {
            track_panic!(
                ErrorKind::UnevaluableParams,
                "Unknown model: {:?}",
                model_spec
            );
        }
    }
}
