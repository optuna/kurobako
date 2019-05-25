use crate::record::{Id, StudyRecord};
use kurobako_core::num::FiniteF64;
use kurobako_core::{ErrorKind, Result};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub struct BenchmarkRecord {
    studies: Vec<StudyRecord>,
}
impl BenchmarkRecord {
    pub fn new(studies: Vec<StudyRecord>) -> Self {
        Self { studies }
    }

    pub fn solver_ids(&self) -> BTreeSet<Id> {
        self.studies.iter().map(|s| s.solver.id()).collect()
    }

    pub fn problems(&self) -> BTreeMap<Id, ProblemRecord> {
        let mut problems = BTreeMap::<Id, ProblemRecord>::new();
        for study in &self.studies {
            let solvers = &mut problems.entry(study.problem.id()).or_default().solvers;
            solvers
                .entry(study.solver.id())
                .or_default()
                .studies
                .push(study);
        }
        problems
    }
}

#[derive(Debug, Default)]
pub struct ProblemRecord<'a> {
    solvers: BTreeMap<Id<'a>, SolverRecord<'a>>,
}
impl<'a> ProblemRecord<'a> {
    pub fn fetch_solver<'b, 'c: 'b>(&'b self, solver_id: Id<'c>) -> Result<&'b SolverRecord<'a>> {
        let solver = track_assert_some!(
            self.solvers.get(&solver_id),
            ErrorKind::InvalidInput; solver_id
        );
        Ok(solver)
    }
}

#[derive(Debug, Default)]
pub struct SolverRecord<'a> {
    studies: Vec<&'a StudyRecord>,
}
impl<'a> SolverRecord<'a> {
    pub fn best_values<'b>(&'b self) -> impl 'b + Iterator<Item = FiniteF64> {
        self.studies.iter().filter_map(|s| s.best_value())
    }
}
