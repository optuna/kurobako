use self::rankings::{Borda, Firsts};
use crate::markdown as md;
use crate::markdown::MarkdownWriter;
use crate::record::{ProblemRecord, SolverRecord, StudyRecord};
use kurobako_core::num::OrderedFloat;
use kurobako_core::{Error, ErrorKind, Result};
use rustats::fundamental::{average, stddev};
use rustats::hypothesis_testings::{Alpha, MannWhitneyU};
use serde::Serialize;
use serde_json;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Write;
use std::str::FromStr;
use std::time::Duration;
use structopt::StructOpt;

mod rankings;

#[derive(Debug, StructOpt, Serialize)]
#[structopt(rename_all = "kebab-case")]
pub struct ReportOpt {
    #[structopt(
        long,
        // default_value = "best-value", // TODO: best-value, auc, elapsed-time
        possible_values = Metric::POSSIBLE_VALUES
    )]
    pub metrics: Vec<Metric>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Metric {
    BestValue,
    Auc,
    ElapsedTime,
}
impl Metric {
    const POSSIBLE_VALUES: &'static [&'static str] = &["best-value", "auc", "elapsed-time"];
}
impl FromStr for Metric {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "best-value" => Ok(Metric::BestValue),
            "auc" => Ok(Metric::Auc),
            "elapsed-time" => Ok(Metric::ElapsedTime),
            _ => track_panic!(ErrorKind::InvalidInput, "Unknown metric name: {:?}", s),
        }
    }
}

#[derive(Debug)]
pub struct Reporter {
    studies: Vec<StudyRecord>,
    opt: ReportOpt,
}
impl Reporter {
    pub fn new(studies: Vec<StudyRecord>, mut opt: ReportOpt) -> Self {
        if opt.metrics.is_empty() {
            opt.metrics = vec![Metric::BestValue, Metric::Auc, Metric::ElapsedTime];
        }
        Self { studies, opt }
    }

    pub fn report_all<W: Write>(&self, writer: &mut MarkdownWriter<W>) -> Result<()> {
        let mut writer = track!(writer.heading("Benchmark Result Report"))?;

        let mut list = writer.list();
        track!(list.item(&format!("Report ID: {}", track!(self.id())?)))?;
        track!(list.item(&format!(
            "Kurobako Version: [{}](https://github.com/sile/kurobako/tree/{})",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_VERSION"),
        )))?;
        track!(list.item(&format!(
            "Number of Solvers: {}",
            track!(self.solvers())?.count()
        )))?;
        track!(list.item(&format!(
            "Number of Problems: {}",
            track!(self.problems())?.count()
        )))?;
        let metrics = self
            .opt
            .metrics
            .iter()
            .map(|m| match m {
                Metric::BestValue => "best value",
                Metric::Auc => "AUC",
                Metric::ElapsedTime => "elapsed time",
            })
            .collect::<Vec<_>>();
        track!(list.item(&format!("Metrics Precedence: `{}`", metrics.join(" -> "))))?;
        track_writeln!(writer.inner_mut())?;

        track_writeln!(
            writer.inner_mut(),
            "Please refer to \
             [\"A Strategy for Ranking Optimizers using Multiple Criteria\"]\
             [Dewancker, Ian, et al., 2016] for the ranking strategy used in this report.\n\n\
             [Dewancker, Ian, et al., 2016]: \
             http://proceedings.mlr.press/v64/dewancker_strategy_2016.pdf"
        )?;

        {
            let mut writer = track!(writer.heading("Table of Contents"))?;
            let mut list = writer.list().numbered();
            track!(list.item("[Overall Results](#overall-results)"))?;
            track!(list.item("[Individual Results](#individual-results)"))?;
            track!(list.item("[Solvers](#solvers)"))?;
            track!(list.item("[Problems](#problems)"))?;
            track!(list.item("[Studies](#studies)"))?;
            track_writeln!(writer.inner_mut())?;
        }

        track!(self.report_overall_results(&mut writer))?;
        track!(self.report_individual_results(&mut writer))?;
        track!(self.report_solvers(&mut writer))?;
        track!(self.report_problems(&mut writer))?;
        track!(self.report_studies(&mut writer))?;

        Ok(())
    }

    pub fn report_overall_results<W: Write>(&self, writer: &mut MarkdownWriter<W>) -> Result<()> {
        let mut writer = track!(writer.heading("Overall Results"))?;
        track_writeln!(writer.inner_mut())?;

        let contests = track!(self.contests())?;
        let (solver_ids, solvers): (Vec<_>, Vec<_>) = track!(self.solvers())?.unzip();
        let mut borda_ranking = Borda::new(solver_ids.iter());
        let mut firsts_ranking = Firsts::new(solver_ids.iter());
        let mut excluded_problems = Vec::new();
        for (problem_id, contest) in contests {
            if !solver_ids
                .iter()
                .all(|s| contest.competitors.contains_key(s))
            {
                excluded_problems.push((problem_id, contest.problem));
                continue;
            }

            borda_ranking.compete(|&a, &b| {
                let a = &contest.competitors[a];
                let b = &contest.competitors[b];
                self.compete(a, b, contest.auc_start_step)
            });
            firsts_ranking.compete(|&a, &b| {
                let a = &contest.competitors[a];
                let b = &contest.competitors[b];
                self.compete(a, b, contest.auc_start_step)
            });
        }

        let mut table = md::Table::new(
            vec![
                md::ColumnHeader::new("Solver", md::Align::Left),
                md::ColumnHeader::new("Borda", md::Align::Right),
                md::ColumnHeader::new("Firsts", md::Align::Right),
            ]
            .into_iter(),
        );

        for (((solver_id, solver), borda), firsts) in solver_ids
            .iter()
            .zip(solvers.iter())
            .zip(borda_ranking.scores())
            .zip(firsts_ranking.scores())
        {
            table
                .row()
                .item(format!("[{}](#id-{})", solver.spec.name, solver_id))
                .item(borda)
                .item(firsts);
        }
        track!(writer.write_table(&table))?;
        track!(writer.newline())?;

        if !excluded_problems.is_empty() {
            let mut writer = track!(writer.heading("Note"))?;
            track!(writer.newline())?;
            track_writeln!(
                writer.inner_mut(),
                "The following problems aren't considered in the above \
                 result because some of the solvers don't participate in the problems:"
            )?;

            let mut list = writer.list();
            for (problem_id, problem) in excluded_problems {
                track!(list.item(&format!("[{}](#id-{})", problem.spec.name, problem_id)))?;
            }
            track!(writer.newline())?;
        }

        Ok(())
    }

    pub fn report_individual_results<W: Write>(
        &self,
        writer: &mut MarkdownWriter<W>,
    ) -> Result<()> {
        let mut writer = track!(writer.heading("Individual Results"))?;
        track_writeln!(writer.inner_mut())?;

        let contests = track!(self.contests())?;
        for (problem_no, (problem_id, contest)) in contests.into_iter().enumerate() {
            let mut writer = track!(writer.heading(&format!(
                "({}) Problem: [{}](#id-{})",
                problem_no + 1,
                contest.problem.spec.name,
                problem_id
            )))?;

            // FIXME: Reduce redundant calculation.
            let auc_start_step = contest.auc_start_step;
            let mut rankings = BTreeMap::new();
            for (solver_id0, competitor0) in &contest.competitors {
                let mut ranking = 1;
                for (solver_id1, competitor1) in &contest.competitors {
                    if solver_id0 == solver_id1 {
                        continue;
                    }

                    if self.compete(competitor0, competitor1, auc_start_step) == Ordering::Greater {
                        ranking += 1;
                    }
                }
                rankings.insert(solver_id0, ranking);
            }
            let mut rankings = rankings.into_iter().map(|x| (x.1, x.0)).collect::<Vec<_>>();
            rankings.sort();

            let mut table = md::Table::new(
                vec![
                    md::ColumnHeader::new("Ranking", md::Align::Right),
                    md::ColumnHeader::new("Solver", md::Align::Left),
                    md::ColumnHeader::new("Best (avg +- sd)", md::Align::Right),
                    md::ColumnHeader::new("AUC (avg +- sd)", md::Align::Right),
                    md::ColumnHeader::new("Elapsed (avg +- sd)", md::Align::Right),
                ]
                .into_iter(),
            );
            for (ranking, solver_id) in rankings {
                let c = &contest.competitors[solver_id];

                let solver = format!(
                    "[{}](#id-{}) ([study](#id-{}))",
                    c.solver.spec.name,
                    solver_id,
                    track!(c.studies[0].id())?
                );

                let best_values = c.best_values().map(|x| x.0).collect::<Vec<_>>();
                let best_value = format!(
                    "{:.06} +- {:.06}",
                    average(best_values.iter().copied()),
                    stddev(best_values.iter().copied())
                );

                let aucs = c.aucs(auc_start_step).map(|x| x.0).collect::<Vec<_>>();
                let auc = format!(
                    "{:.03} +- {:.03}",
                    average(aucs.iter().copied()),
                    stddev(aucs.iter().copied())
                );

                let elapsed_times = c
                    .elapsed_times()
                    .map(|x| x.as_secs_f64())
                    .collect::<Vec<_>>();
                let elapsed_time = format!(
                    "{:.03} +- {:.03}",
                    average(elapsed_times.iter().copied()),
                    stddev(elapsed_times.iter().copied())
                );
                table
                    .row()
                    .item(ranking)
                    .item(solver)
                    .item(best_value)
                    .item(auc)
                    .item(elapsed_time);
            }

            track!(writer.write_table(&table))?;
            track_writeln!(writer.inner_mut())?;
        }

        Ok(())
    }

    pub fn report_solvers<W: Write>(&self, writer: &mut MarkdownWriter<W>) -> Result<()> {
        let mut writer = track!(writer.heading("Solvers"))?;
        for (id, solver) in track!(self.solvers())? {
            let mut writer = track!(writer.heading(&format!("ID: {}", id)))?;

            track_writeln!(writer.inner_mut(), "recipe:")?;
            let json = track!(serde_json::to_string_pretty(&solver.recipe).map_err(Error::from))?;
            track!(writer.code_block("json", &json))?;
            track_writeln!(writer.inner_mut())?;

            track_writeln!(writer.inner_mut(), "specification:")?;
            let json = track!(serde_json::to_string_pretty(&solver.spec).map_err(Error::from))?;
            track!(writer.code_block("json", &json))?;
            track_writeln!(writer.inner_mut())?;
        }
        Ok(())
    }

    pub fn report_problems<W: Write>(&self, writer: &mut MarkdownWriter<W>) -> Result<()> {
        let mut writer = track!(writer.heading("Problems"))?;
        for (id, problem) in track!(self.problems())? {
            let mut writer = track!(writer.heading(&format!("ID: {}", id)))?;

            track_writeln!(writer.inner_mut(), "recipe:")?;
            let json = track!(serde_json::to_string_pretty(&problem.recipe).map_err(Error::from))?;
            track!(writer.code_block("json", &json))?;
            track_writeln!(writer.inner_mut())?;

            track_writeln!(writer.inner_mut(), "specification:")?;
            let json = track!(serde_json::to_string_pretty(&problem.spec).map_err(Error::from))?;
            track!(writer.code_block("json", &json))?;
            track_writeln!(writer.inner_mut())?;
        }
        Ok(())
    }

    pub fn report_studies<W: Write>(&self, writer: &mut MarkdownWriter<W>) -> Result<()> {
        let mut writer = track!(writer.heading("Studies"))?;
        let mut studies = BTreeMap::<_, Vec<_>>::new();
        for study in &self.studies {
            let id = track!(study.id())?;
            studies
                .entry((&study.problem.spec.name, &study.solver.spec.name, id))
                .or_default()
                .push(study);
        }
        for ((problem_name, solver_name, id), studies) in studies {
            let mut writer = track!(writer.heading(&format!("ID: {}", id)))?;
            let mut list = writer.list();
            track!(list.item(&format!(
                "problem: [{}](id-{})",
                problem_name,
                track!(studies[0].problem.id())?
            )))?;
            track!(list.item(&format!(
                "solver: [{}](id-{})",
                solver_name,
                track!(studies[0].solver.id())?
            )))?;
            track!(list.item(&format!("budget: {}", studies[0].budget)))?;
            track!(list.item(&format!("repeats: {}", studies.len())))?;
            track!(list.item(&format!("concurrency: {}", studies[0].concurrency)))?;
            if studies[0].concurrency.get() > 1 {
                track!(list.item(&format!("scheduling: {}", studies[0].scheduling)))?;
            }
            track_writeln!(writer.inner_mut())?;
        }
        Ok(())
    }

    fn id(&self) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.input(&track!(
            serde_json::to_vec(&self.studies).map_err(Error::from)
        )?);
        hasher.input(&track!(serde_json::to_vec(&self.opt).map_err(Error::from))?);

        let mut id = String::with_capacity(64);
        for b in hasher.result().as_slice() {
            track_write!(&mut id, "{:02x}", b)?;
        }
        Ok(id)
    }

    fn solvers<'a>(&'a self) -> Result<impl 'a + Iterator<Item = (String, &'a SolverRecord)>> {
        let mut map = BTreeMap::new();
        for study in &self.studies {
            let id = track!(study.solver.id())?;
            map.insert((&study.solver.spec.name, id), &study.solver);
        }
        Ok(map.into_iter().map(|(k, v)| (k.1, v)))
    }

    fn problems<'a>(&'a self) -> Result<impl 'a + Iterator<Item = (String, &'a ProblemRecord)>> {
        let mut map = BTreeMap::new();
        for study in &self.studies {
            let id = track!(study.problem.id())?;
            map.insert((&study.problem.spec.name, id), &study.problem);
        }
        Ok(map.into_iter().map(|(k, v)| (k.1, v)))
    }

    fn compete(&self, a: &Competitor, b: &Competitor, auc_start_step: u64) -> Ordering {
        for metric in &self.opt.metrics {
            let order = match metric {
                Metric::BestValue => {
                    MannWhitneyU::new(a.best_values(), b.best_values()).order(Alpha::P05)
                }
                Metric::Auc => MannWhitneyU::new(a.aucs(auc_start_step), b.aucs(auc_start_step))
                    .order(Alpha::P05),
                Metric::ElapsedTime => {
                    MannWhitneyU::new(a.elapsed_times(), b.elapsed_times()).order(Alpha::P05)
                }
            };
            if order != Ordering::Equal {
                return order;
            }
        }
        Ordering::Equal
    }

    fn contests(&self) -> Result<BTreeMap<String, Contest>> {
        let mut contests = BTreeMap::new();
        for study in &self.studies {
            let problem_id = track!(study.problem.id())?;
            let contest = contests.entry(problem_id).or_insert_with(|| Contest {
                problem: &study.problem,
                competitors: BTreeMap::new(),
                auc_start_step: study.problem.spec.steps.last(),
            });
            if let Some(trial) = study.first_complete_trial() {
                if let Some(step) = trial.start_step() {
                    if contest.auc_start_step < step {
                        contest.auc_start_step = step;
                    }
                }
            }

            let solver_id = track!(study.solver.id())?;
            contest
                .competitors
                .entry(solver_id)
                .or_insert_with(|| Competitor {
                    solver: &study.solver,
                    studies: Vec::new(),
                })
                .studies
                .push(study)
        }
        Ok(contests)
    }
}

struct Contest<'a> {
    problem: &'a ProblemRecord,
    competitors: BTreeMap<String, Competitor<'a>>,
    auc_start_step: u64,
}

struct Competitor<'a> {
    solver: &'a SolverRecord,
    studies: Vec<&'a StudyRecord>,
}
impl<'a> Competitor<'a> {
    pub fn best_values<'b>(&'b self) -> impl 'b + Iterator<Item = OrderedFloat<f64>> {
        self.studies
            .iter()
            .filter_map(|s| s.best_value())
            .map(OrderedFloat)
    }

    pub fn aucs<'b>(&'b self, start_step: u64) -> impl 'b + Iterator<Item = OrderedFloat<f64>> {
        self.studies
            .iter()
            .filter_map(move |s| s.auc(start_step))
            .map(OrderedFloat)
    }

    pub fn elapsed_times<'b>(&'b self) -> impl 'b + Iterator<Item = Duration> {
        self.studies.iter().map(|s| s.solver_elapsed())
    }
}
