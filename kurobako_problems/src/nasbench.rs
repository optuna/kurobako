//! A problem based on the benchmark described in [NAS-Bench-101: Towards Reproducible Neural Architecture Search][nasbench].
//!
//! [nasbench]: https://arxiv.org/abs/1902.09635
use kurobako_core::domain::{self, VariableBuilder};
use kurobako_core::num::OrderedFloat;
use kurobako_core::problem::{
    Evaluator, Problem, ProblemFactory, ProblemRecipe, ProblemSpec, ProblemSpecBuilder,
};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::{ArcRng, Rng};
use kurobako_core::trial::{Params, Values};
use kurobako_core::{Error, ErrorKind, Result};
use nasbench::{AdjacencyMatrix, ModelSpec, NasBench, Op};
use serde::{Deserialize, Serialize};
use std::collections::{Bound, HashSet};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use structopt::StructOpt;

const MAX_EDGES: usize = 9;
const VERTICES: usize = 7;
const EDGE_KINDS: usize = VERTICES * (VERTICES - 1) / 2;

/// Recipe of `NasbenchProblem`.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct NasbenchProblemRecipe {
    /// Path of the NASBench dataset.
    pub dataset: PathBuf,

    /// Encoding type of the NASBench search space.
    #[structopt(
        long,
        default_value = "A",
        possible_values = &Encoding::POSSIBLE_VALUES
    )]
    pub encoding: Encoding,
}
impl ProblemRecipe for NasbenchProblemRecipe {
    type Factory = NasbenchProblemFactory;

    fn create_factory(&self, _registry: &FactoryRegistry) -> Result<Self::Factory> {
        let nasbench = track!(NasBench::new(&self.dataset))?;
        Ok(NasbenchProblemFactory {
            nasbench: Arc::new(nasbench),
            encoding: self.encoding,
        })
    }
}

/// Factory of `NasbenchProblem`.
#[derive(Debug)]
pub struct NasbenchProblemFactory {
    nasbench: Arc<NasBench>,
    encoding: Encoding,
}
impl ProblemFactory for NasbenchProblemFactory {
    type Problem = NasbenchProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        let spec = ProblemSpecBuilder::new(&format!("NASBench ({:?})", self.encoding))
            .attr(
                "paper",
                "Ying, Chris, et al. \"Nas-bench-101: Towards reproducible \
                 neural architecture search.\" arXiv preprint arXiv:1902.09635 (2019).",
            )
            .attr("github", "https://github.com/automl/nas_benchmarks")
            .params(self.encoding.params())
            .value(domain::var("1.0 - Validation Accuracy").continuous(0.0, 1.0))
            .steps(vec![4, 12, 36, 108]);

        track!(spec.finish())
    }

    fn create_problem(&self, rng: ArcRng) -> Result<Self::Problem> {
        Ok(NasbenchProblem {
            nasbench: Arc::clone(&self.nasbench),
            encoding: self.encoding,
            rng,
        })
    }
}

/// NASBench problem.
#[derive(Debug)]
pub struct NasbenchProblem {
    nasbench: Arc<NasBench>,
    encoding: Encoding,
    rng: ArcRng,
}
impl Problem for NasbenchProblem {
    type Evaluator = NasbenchEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        let (ops, edges) = track!(self.encoding.ops_and_edges(&params))?;
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
        track_assert!(
            self.nasbench.models().contains_key(&model_spec),
            ErrorKind::UnevaluableParams,
            "Unknown model: {:?}",
            model_spec
        );

        Ok(NasbenchEvaluator {
            nasbench: Arc::clone(&self.nasbench),
            encoding: self.encoding,
            model_spec,
            sample_index: track!(self.rng.with_lock(|rng| rng.gen()))?,
        })
    }
}

/// Evaluator of `NasbenchProblem`.
#[derive(Debug)]
pub struct NasbenchEvaluator {
    nasbench: Arc<NasBench>,
    encoding: Encoding,
    model_spec: ModelSpec,
    sample_index: usize,
}
impl Evaluator for NasbenchEvaluator {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        let model =
            track_assert_some!(self.nasbench.models().get(&self.model_spec), ErrorKind::Bug);

        let epoch_num = next_step as u8;
        let (current_step, epoch_candidates) = track_assert_some!(
            model
                .epochs
                .range((Bound::Included(epoch_num), Bound::Unbounded))
                .nth(0),
            ErrorKind::InvalidInput
        );
        let epoch = &epoch_candidates[self.sample_index % epoch_candidates.len()];

        let value = 1.0 - epoch.complete.validation_accuracy;
        Ok((u64::from(*current_step), Values::new(vec![value])))
    }
}

/// Encoding method of the NASBench search space.
///
/// For the details of each encoding, please see [the paper][paper] and [nas_cifar10.py].
///
/// [paper]: https://arxiv.org/abs/1902.09635
/// [nas_cifar10.py]: https://github.com/automl/nas_benchmarks/blob/c1bae6632bf15d45ba49c269c04dbbeb3f0379f0/tabular_benchmarks/nas_cifar10.py
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum Encoding {
    A,
    B,
    C,
}
impl Encoding {
    const POSSIBLE_VALUES: [&'static str; 3] = ["A", "B", "C"];

    fn params(&self) -> Vec<VariableBuilder> {
        match self {
            Encoding::A => Self::params_a(),
            Encoding::B => Self::params_b(),
            Encoding::C => Self::params_c(),
        }
    }

    fn common_params() -> Vec<VariableBuilder> {
        let mut params = Vec::new();
        for i in 0..5 {
            params.push(domain::var(&format!("op{}", i)).categorical(&[
                "conv1x1-bn-relu",
                "conv3x3-bn-relu",
                "maxpool3x3",
            ]));
        }
        params
    }

    fn params_a() -> Vec<VariableBuilder> {
        let mut params = Self::common_params();
        for i in 0..EDGE_KINDS {
            params.push(domain::var(&format!("edge{}", i)).boolean());
        }
        params
    }

    fn params_b() -> Vec<VariableBuilder> {
        let mut params = Self::common_params();
        for i in 0..MAX_EDGES {
            let edge_kinds = (0..EDGE_KINDS).map(|i| i.to_string());
            params.push(domain::var(&format!("edge{}", i)).categorical(edge_kinds));
        }
        params
    }

    fn params_c() -> Vec<VariableBuilder> {
        let mut params = Self::common_params();

        params.push(domain::var("num_edges").discrete(0, MAX_EDGES as i64 + 1));
        for i in 0..EDGE_KINDS {
            params.push(domain::var(&format!("edge{}", i)).continuous(0.0, 1.0));
        }
        params
    }

    fn ops_and_edges(&self, params: &[f64]) -> Result<(Vec<Op>, HashSet<usize>)> {
        let mut ops = vec![Op::Input];
        for p in &params[0..5] {
            let op = match *p as u8 {
                0 => Op::Conv1x1,
                1 => Op::Conv3x3,
                2 => Op::MaxPool3x3,
                _ => track_panic!(ErrorKind::InvalidInput, "Unexpected parameter: {:?}", p),
            };
            ops.push(op);
        }
        ops.push(Op::Output);

        let edges = track!(self.edges(&params[5..]))?;
        Ok((ops, edges))
    }

    fn edges(&self, params: &[f64]) -> Result<HashSet<usize>> {
        match self {
            Encoding::A => track!(Self::edges_a(params)),
            Encoding::B => track!(Self::edges_b(params)),
            Encoding::C => track!(Self::edges_c(params)),
        }
    }

    fn edges_a(params: &[f64]) -> Result<HashSet<usize>> {
        let mut edges = HashSet::new();
        for (i, p) in params.iter().enumerate() {
            if *p == 1.0 {
                edges.insert(i);
            }
        }
        Ok(edges)
    }

    fn edges_b(params: &[f64]) -> Result<HashSet<usize>> {
        let mut edges = HashSet::new();
        for p in params {
            edges.insert(*p as usize);
        }
        Ok(edges)
    }

    fn edges_c(params: &[f64]) -> Result<HashSet<usize>> {
        let num_edges = params[0] as usize;

        let mut edges = Vec::new();
        for (i, p) in params[1..].iter().enumerate() {
            edges.push((*p, i));
        }
        assert_eq!(edges.len(), EDGE_KINDS);

        edges.sort_by_key(|&(a, b)| (OrderedFloat(a), b));
        Ok(edges
            .iter()
            .rev()
            .take(num_edges)
            .map(|t| t.1)
            .collect::<HashSet<_>>())
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
impl Default for Encoding {
    fn default() -> Self {
        Encoding::A
    }
}
