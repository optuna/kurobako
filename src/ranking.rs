use crate::markdown::MarkdownWriter;
use crate::record::StudyRecord;
use kurobako_core::{Error, ErrorKind, Result};
use std::io::Write;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct RankingOpt {
    #[structopt(
        long,
        default_value = "best-value", // TODO: best-value, auc, elapsed-time
        possible_values = Metric::POSSIBLE_VALUES
    )]
    pub metrics: Vec<Metric>,

    #[structopt(long, default_value = "1")]
    pub heading_level: usize,
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
pub struct SolverRanking {
    studies: Vec<StudyRecord>,
    opt: RankingOpt,
}
impl SolverRanking {
    pub fn new(studies: Vec<StudyRecord>, opt: RankingOpt) -> Self {
        Self { studies, opt }
    }

    pub fn output<W: Write>(&self, writer: W) -> Result<()> {
        let mut writer = MarkdownWriter::with_level(writer, self.opt.heading_level);
        let mut writer = track!(writer.heading("Solver Ranking"))?;

        track!(self.output_settings(&mut writer))?;

        // TODO: output link to the paper

        Ok(())
    }

    fn output_settings<W: Write>(&self, writer: &mut MarkdownWriter<W>) -> Result<()> {
        track_writeln!(writer.inner_mut(), "Settings:")?;

        let mut list = writer.list();

        track!(list.item("Metrics Precedence:"))?;
        {
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

        track!(list.item("Problems:"))?;
        {
            // let mut list = list.list();
        }
        track!(writer.newline())?;

        Ok(())
    }
}
