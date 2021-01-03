use crate::record::{
    EvaluationRecord, ProblemRecord, SolverRecord, TrialRecord, TrialRecordBuilder,
};
use crate::study::{Scheduling, StudyRecipe};
use crate::time::DateTime;
use chrono::Local;
use kurobako_core::hypervolume;
use kurobako_core::num::OrderedFloat;
use kurobako_core::problem::ProblemSpec;
use kurobako_core::solver::SolverSpec;
use kurobako_core::trial::{Params, TrialId, Values};
use kurobako_core::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap};
use std::fmt::Write as _;
use std::num::NonZeroUsize;
use std::time::Duration;

#[derive(Debug)]
pub struct StudyRecordBuilder {
    recipe: StudyRecipe,
    solver: SolverSpec,
    problem: ProblemSpec,
    start_time: DateTime,
    trials: BTreeMap<TrialId, TrialRecord>,
    pareto_frontier: BTreeMap<TrialId, (Params, Values)>,
}
impl StudyRecordBuilder {
    pub fn new(recipe: StudyRecipe, solver: SolverSpec, problem: ProblemSpec) -> Self {
        Self {
            recipe,
            solver,
            problem,
            start_time: Local::now(),
            trials: BTreeMap::new(),
            pareto_frontier: BTreeMap::new(),
        }
    }

    pub fn add_trial(&mut self, trial: TrialRecordBuilder) {
        let t = self.trials.entry(trial.id).or_insert_with(|| TrialRecord {
            thread_id: trial.thread_id,
            params: trial.params.clone(),
            evaluations: Vec::new(),
        });

        t.evaluations.push(EvaluationRecord {
            values: trial.values.clone(),
            start_step: trial.start_step,
            end_step: trial.end_step,
            ask_elapsed: trial.ask_elapsed,
            tell_elapsed: trial.tell_elapsed,
            evaluate_elapsed: trial.evaluate_elapsed,
        });

        if t.steps() == self.problem.steps.last() {
            let is_dominated = self
                .pareto_frontier
                .values()
                .any(|(_, vs)| vs.partial_cmp(&trial.values) == Some(Ordering::Less));
            if !is_dominated {
                let dominated = self
                    .pareto_frontier
                    .iter()
                    .filter(|(_, (_, vs))| trial.values.partial_cmp(vs) == Some(Ordering::Less))
                    .map(|(&id, _)| id)
                    .collect::<Vec<_>>();

                self.pareto_frontier
                    .insert(trial.id, (trial.params, trial.values));
                for id in dominated {
                    self.pareto_frontier.remove(&id);
                }
            }
        }
    }

    pub fn pareto_frontier(&self) -> impl '_ + Iterator<Item = (TrialId, &Params, &Values)> {
        self.pareto_frontier
            .iter()
            .map(|(&id, (params, values))| (id, params, values))
    }

    pub fn finish(self) -> StudyRecord {
        StudyRecord {
            start_time: self.start_time,
            end_time: Local::now(),
            budget: self.recipe.budget,
            seed: self.recipe.seed.unwrap_or_else(|| unreachable!()),
            concurrency: self.recipe.concurrency,
            scheduling: self.recipe.scheduling,
            solver: SolverRecord {
                recipe: self.recipe.solver,
                spec: self.solver,
            },
            problem: ProblemRecord {
                recipe: self.recipe.problem,
                spec: self.problem,
            },
            trials: self.trials.into_iter().map(|(_, v)| v).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyRecord {
    pub start_time: DateTime,
    pub end_time: DateTime,
    pub seed: u64,
    pub budget: u64,
    pub concurrency: NonZeroUsize,
    pub scheduling: Scheduling,
    pub solver: SolverRecord,
    pub problem: ProblemRecord,
    pub trials: Vec<TrialRecord>,
}
impl StudyRecord {
    pub fn id(&self) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.input(&track!(
            serde_json::to_vec(&self.budget).map_err(Error::from)
        )?);
        hasher.input(&track!(
            serde_json::to_vec(&self.concurrency).map_err(Error::from)
        )?);
        hasher.input(&track!(
            serde_json::to_vec(&self.scheduling).map_err(Error::from)
        )?);
        hasher.input(&track!(
            serde_json::to_vec(&self.solver).map_err(Error::from)
        )?);
        hasher.input(&track!(
            serde_json::to_vec(&self.problem).map_err(Error::from)
        )?);

        let mut id = String::with_capacity(64);
        for b in hasher.result().as_slice() {
            track_write!(&mut id, "{:02x}", b)?;
        }
        Ok(id)
    }

    pub fn study_steps(&self) -> u64 {
        self.problem.spec.steps.last() * self.budget
    }

    pub fn best_values(&self) -> BTreeMap<u64, f64> {
        let mut best_values = BTreeMap::new();

        let problem_steps = self.problem.spec.steps.last();
        let mut trials = self
            .trials
            .iter()
            .filter_map(|t| {
                if let (Some(step), Some(value)) = (t.end_step(), t.value(problem_steps)) {
                    Some((step, value))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        trials.sort_by_key(|t| t.0);

        let mut min = std::f64::INFINITY;
        for (step, value) in trials {
            if value < min {
                min = value;
                best_values.insert(step, min);
            }
        }

        best_values
    }

    pub fn hypervolumes(&self) -> BTreeMap<u64, f64> {
        let mut hypervolumes = BTreeMap::new();

        let problem_steps = self.problem.spec.steps.last();
        let mut trials = self
            .trials
            .iter()
            .filter_map(|t| {
                if let (Some(step), Some(value)) = (t.end_step(), t.values(problem_steps)) {
                    Some((step, value))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        trials.sort_by_key(|t| t.0);

        let ref_pt = match self.problem.spec.attrs.get("reference_point") {
            Some(ref_pt_str) => ref_pt_str
                .split(',')
                .map(|cor| {
                    cor.parse()
                        .expect("Could not parse the reference point coordinate to float")
                })
                .collect(),
            None => vec![100.0; self.problem.spec.values_domain.len()],
        };

        let mut pts = Vec::new();
        for (step, values) in trials {
            pts.push(values.to_vec());
            let hv = hypervolume::compute(&pts, &ref_pt);
            hypervolumes.insert(step, hv);
        }

        hypervolumes
    }

    pub fn elapsed_times(&self, include_evaluate_time: bool) -> BTreeMap<u64, f64> {
        let mut times = BTreeMap::new();
        let mut elapsed = 0.0;
        for e in self.evaluations() {
            elapsed += e.ask_elapsed.get() + e.tell_elapsed.get();
            if include_evaluate_time {
                elapsed += e.evaluate_elapsed.get();
            }
            times.insert(e.end_step, elapsed);
        }
        times
    }

    pub fn evaluations(&self) -> impl '_ + Iterator<Item = &EvaluationRecord> {
        struct Entry<'a> {
            trial: &'a TrialRecord,
            index: usize,
        }
        impl<'a> Entry<'a> {
            fn step(&self) -> u64 {
                self.trial.evaluations[self.index].end_step
            }
        }
        impl<'a> PartialEq for Entry<'a> {
            fn eq(&self, other: &Self) -> bool {
                self.step() == other.step()
            }
        }
        impl<'a> Eq for Entry<'a> {}
        impl<'a> PartialOrd for Entry<'a> {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                other.step().partial_cmp(&self.step())
            }
        }
        impl<'a> Ord for Entry<'a> {
            fn cmp(&self, other: &Self) -> Ordering {
                other.step().cmp(&self.step())
            }
        }

        let queue = self
            .trials
            .iter()
            .filter(|t| !t.evaluations.is_empty())
            .map(|trial| Entry { trial, index: 0 })
            .collect::<BinaryHeap<_>>();

        struct Iter<'a> {
            queue: BinaryHeap<Entry<'a>>,
        }
        impl<'a> Iterator for Iter<'a> {
            type Item = &'a EvaluationRecord;

            fn next(&mut self) -> Option<Self::Item> {
                if let Some(mut entry) = self.queue.pop() {
                    let item = &entry.trial.evaluations[entry.index];
                    entry.index += 1;
                    if entry.index < entry.trial.evaluations.len() {
                        self.queue.push(entry);
                    }
                    Some(item)
                } else {
                    None
                }
            }
        }

        Iter { queue }
    }

    pub fn best_value(&self) -> Option<f64> {
        let problem_steps = self.problem.spec.steps.last();
        self.trials
            .iter()
            .filter_map(|t| t.value(problem_steps))
            .map(OrderedFloat)
            .min()
            .map(|x| x.0)
    }

    pub fn auc(&self, start_step: u64) -> Option<f64> {
        let vars = self.problem.spec.values_domain.variables();
        if vars.len() != 1 {
            return None;
        }

        let mut global_min = vars[0].range().low();
        if !global_min.is_finite() {
            global_min = 0.0;
        }

        let problem_steps = self.problem.spec.steps.last();
        let mut trials = self
            .trials
            .iter()
            .filter_map(|t| {
                if let (Some(step), Some(value)) = (t.end_step(), t.value(problem_steps)) {
                    Some((step, value))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        trials.sort_by_key(|t| t.0);

        let mut prev_step = 0;
        let mut current_min = std::f64::INFINITY;
        let mut auc = 0.0;
        for (mut step, value) in trials {
            if step <= start_step {
                step = start_step;
            } else {
                if prev_step == 0 {
                    return None;
                }
                auc += (current_min - global_min) * (step - prev_step) as f64;
            }

            if value < current_min {
                current_min = value;
            }
            prev_step = step;
        }

        let study_steps = self.budget * problem_steps;
        auc += (current_min - global_min) * (study_steps - prev_step) as f64;

        Some(auc / problem_steps as f64)
    }

    pub fn solver_elapsed(&self) -> Duration {
        self.trials.iter().map(|t| t.solver_elapsed()).sum()
    }

    pub fn first_complete_trial(&self) -> Option<&TrialRecord> {
        let problem_steps = self.problem.spec.steps.last();
        self.trials
            .iter()
            .filter(|t| t.steps() != problem_steps)
            .min_by_key(|t| t.start_step())
    }
}
