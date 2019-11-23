use crate::markdown::MarkdownWriter;
use crate::record::{ProblemRecord, SolverRecord, StudyRecord};
use kurobako_core::{Error, ErrorKind, Result};
use serde::Serialize;
use serde_json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Write;
use std::str::FromStr;
use structopt::StructOpt;

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
        track_writeln!(writer.inner_mut())?;

        track_writeln!(
            writer.inner_mut(),
            "Please refer to \
             [\"A Strategy for Ranking Optimizers using Multiple Criteria\"]\
             [Dewancker, Ian, et al., 2016] for the ranking strategy used in this report.\n\n\
             [Dewancker, Ian, et al., 2016]: \
             http://proceedings.mlr.press/v64/dewancker_strategy_2016.pdf"
        )?;

        track!(self.report_settings(&mut writer))?;
        track!(self.report_solvers(&mut writer))?;
        track!(self.report_problems(&mut writer))?;

        Ok(())
    }

    pub fn report_settings<W: Write>(&self, writer: &mut MarkdownWriter<W>) -> Result<()> {
        let mut writer = track!(writer.heading("Settings"))?;

        {
            let mut writer = track!(writer.heading("Metrics Precedence"))?;
            let mut list = writer.list().numbered();
            for m in &self.opt.metrics {
                let m = match m {
                    Metric::BestValue => "best value",
                    Metric::Auc => "AUC",
                    Metric::ElapsedTime => "elapsed time",
                };
                track!(list.item(m))?;
            }
            track_writeln!(writer.inner_mut())?;
        }
        {
            let mut writer = track!(writer.heading("Solvers"))?;
            let mut list = writer.list().numbered();
            for (id, solver) in track!(self.solvers())? {
                track!(list.item(&format!("[{}](#id-{})", solver.spec.name, id)))?;
            }
            track_writeln!(writer.inner_mut())?;
        }
        {
            let mut writer = track!(writer.heading("Problems"))?;
            let mut list = writer.list().numbered();
            for (id, problem) in track!(self.problems())? {
                track!(list.item(&format!("[{}](#id-{})", problem.spec.name, id)))?;
            }
            track_writeln!(writer.inner_mut())?;
        }
        Ok(())
    }

    pub fn report_solvers<W: Write>(&self, writer: &mut MarkdownWriter<W>) -> Result<()> {
        let mut writer = track!(writer.heading("Solvers"))?;
        for (id, solver) in track!(self.solvers())? {
            let mut writer = track!(writer.heading(&format!("ID: {}", id)))?;
            let json = track!(serde_json::to_string_pretty(solver).map_err(Error::from))?;
            track!(writer.code_block("json", &json))?;
        }
        Ok(())
    }

    pub fn report_problems<W: Write>(&self, writer: &mut MarkdownWriter<W>) -> Result<()> {
        let mut writer = track!(writer.heading("Problems"))?;
        for (id, problem) in track!(self.problems())? {
            let mut writer = track!(writer.heading(&format!("ID: {}", id)))?;
            let json = track!(serde_json::to_string_pretty(problem).map_err(Error::from))?;
            track!(writer.code_block("json", &json))?;
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
}
