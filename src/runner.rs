use crate::problem::FullKurobakoProblemRecipe;
use crate::solver::KurobakoSolverRecipe;
use crate::study::StudyRecord;
use crate::time::Stopwatch;
use crate::trial::{AskRecord, EvalRecord, TrialRecord};
use kurobako_core::problem::{Evaluate, Problem, ProblemRecipe};
use kurobako_core::solver::{Solver, SolverRecipe};
use kurobako_core::Error;
use rand::rngs::ThreadRng;
use rand::{self, Rng};
use yamakan::budget::Budget;
use yamakan::observation::SerialIdGenerator;

#[derive(Debug)]
pub struct Runner<R = ThreadRng> {
    rng: R,
    idgen: SerialIdGenerator,
}
impl Runner<ThreadRng> {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
            idgen: SerialIdGenerator::new(),
        }
    }
}
impl<R: Rng> Runner<R> {
    pub fn run<O, P>(
        &mut self,
        solver_recipe: &O,
        problem_recipe: &P,
        budget_factor: usize,
    ) -> Result<StudyRecord, Error>
    where
        O: SolverRecipe,
        P: ProblemRecipe,
    {
        let mut problem = problem_recipe.create_problem()?;
        let problem_spec = problem.specification();
        let mut solver = solver_recipe.create_solver(problem_spec.clone())?;
        let mut budget = Budget::new(budget_factor as u64 * problem_spec.evaluation_expense.get());

        let mut study_record = StudyRecord::new(
            solver_recipe,
            problem_recipe,
            budget.amount,
            problem_spec.clone(),
            solver.specification(),
        )?;

        let mut curr_id = None;
        let mut evaluator = None;

        let mut errors = 0;
        let watch = Stopwatch::new();
        while !budget.is_consumed() {
            let (ask, mut obs) =
                track!(AskRecord::with(&watch, || solver.ask(&mut self.rng, &mut self.idgen)))?;
            if Some(obs.id) != curr_id {
                // TODO: handle cuncurrent
                curr_id = Some(obs.id);
                evaluator = Some(track!(problem.create_evaluator(obs.id))?);

                study_record.trials.push(TrialRecord {
                    ask: ask.clone(),
                    evals: vec![],
                });
            }

            let eval_result = EvalRecord::with(
                &watch,
                budget.consumption,
                obs.param.budget_mut(),
                |budget| track!(evaluator.as_mut().unwrap().evaluate(&ask.params, budget)),
            );
            match eval_result {
                Ok((eval, values)) => {
                    errors = 0;
                    budget.consumption += eval.cost();
                    let obs = obs.map_value(|()| values);
                    track!(solver.tell(obs))?;

                    study_record.trials.last_mut().unwrap().evals.push(eval);
                }
                Err(e) => {
                    // TODO
                    eprintln!("# Error: {}", e);
                    errors += 1;
                    if errors > 1000 {
                        return Err(track!(e));
                    }

                    curr_id = None;
                    evaluator = None;
                }
            }
        }
        Ok(study_record)
    }
}

#[derive(Debug)]
pub struct RunSpec<'a> {
    pub solver: &'a KurobakoSolverRecipe,
    pub problem: &'a FullKurobakoProblemRecipe,
    pub budget: usize,
}
