use kurobako_core::parameter::{choices, uniform, ParamValue};
use kurobako_core::problem::{
    Evaluate, EvaluatorCapability, Problem, ProblemRecipe, ProblemSpec, Values,
};
use kurobako_core::{ErrorKind, Result};
use nasbench::{AdjacencyMatrix, ModelSpec, NasBench, Op};
use rustats::num::FiniteF64;
use rustats::range::MinMax;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::rc::Rc;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct NasbenchProblemRecipe {
    pub dataset_path: PathBuf,
}
impl ProblemRecipe for NasbenchProblemRecipe {
    type Problem = NasbenchProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        let nasbench = track!(NasBench::new(&self.dataset_path))?;
        Ok(NasbenchProblem {
            nasbench: Rc::new(nasbench),
        })
    }
}

#[derive(Debug)]
pub struct NasbenchProblem {
    nasbench: Rc<NasBench>,
}
impl Problem for NasbenchProblem {
    type Evaluator = NasbenchEvaluator;

    fn specification(&self) -> ProblemSpec {
        let mut params_domain = Vec::new();
        for i in 0..5 {
            params_domain.push(choices(
                &format!("op{}", i),
                &["conv1x1-bn-relu", "conv3x3-bn-relu", "maxpool3x3"],
            ));
        }
        for i in 0..(6 + 5 + 4 + 3 + 2 + 1) {
            params_domain.push(
                uniform(&format!("adjacency{}", i), 0.0, 1.0)
                    .unwrap_or_else(|e| unreachable!("{}", e)),
            );
        }

        ProblemSpec {
            name: "nasbench".to_owned(),
            version: None,
            params_domain,
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
            sample_index: id.get() as usize,
        })
    }
}

#[derive(Debug)]
pub struct NasbenchEvaluator {
    nasbench: Rc<NasBench>,
    sample_index: usize,
}
impl Evaluate for NasbenchEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Values> {
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

        let mut edges = Vec::new();
        for (i, p) in params[5..].iter().enumerate() {
            if let ParamValue::Continuous(v) = p {
                edges.push((v, i));
            } else {
                track_panic!(ErrorKind::InvalidInput, "Unexpected parameter: {:?}", p);
            }
        }
        assert_eq!(edges.len(), 21);

        edges.sort();
        let edges = edges.iter().take(9).map(|t| t.1).collect::<HashSet<_>>();
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
            track_panic!(ErrorKind::Other, "Unknown model: {:?}", model_spec);
        }
    }
}
