use crate::markdown::MarkdownWriter;
use crate::record::{ProblemRecord, SolverRecord, StudyRecord};
use kurobako_core::{Error, ErrorKind, Result};
use std::collections::BTreeMap;
use std::io::Write;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct ReportOpt {
    #[structopt(
        long,
        // default_value = "best-value", // TODO: best-value, auc, elapsed-time
        possible_values = Metric::POSSIBLE_VALUES
    )]
    pub metrics: Vec<Metric>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    pub fn report_all<W: Write>(&self, mut writer: MarkdownWriter<W>) -> Result<()> {
        let writer = track!(writer.heading("Benchmark Result Report"))?;

        track!(self.report_settings(writer))?;

        Ok(())
    }

    pub fn report_settings<W: Write>(&self, mut writer: MarkdownWriter<W>) -> Result<()> {
        let mut writer = track!(writer.heading("Settings"))?;

        let mut list = writer.list();
        {
            track!(list.item("Metrics Precedence:"))?;
            let mut list = list.numbered_list();
            for m in &self.opt.metrics {
                let m = match m {
                    Metric::BestValue => "best value",
                    Metric::Auc => "AUC",
                    Metric::ElapsedTime => "elapsed time",
                };
                track!(list.item(m))?;
            }
        }
        {
            track!(list.item("Solvers:"))?;
            let mut list = list.numbered_list();
            for (id, solver) in track!(self.solvers())? {
                track!(list.item(&format!("[{}](#id-{})", solver.spec.name, id)))?;
            }
        }
        {
            track!(list.item("Problems:"))?;
            let mut list = list.numbered_list();
            for (id, problem) in track!(self.problems())? {
                track!(list.item(&format!("[{}](#id-{})", problem.spec.name, id)))?;
            }
        }
        track!(writer.newline())?;

        track_writeln!(
            writer.inner_mut(),
            "Please refer to \
             [\"A Strategy for Ranking Optimizers using Multiple Criteria\"]\
             [Dewancker, Ian, et al., 2016] for the ranking strategy used in this report.\n\n\
             [Dewancker, Ian, et al., 2016]: \
             http://proceedings.mlr.press/v64/dewancker_strategy_2016.pdf"
        )?;

        Ok(())
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
}
