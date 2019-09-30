use crate::markdown::{Align, ColumnHeader, MarkdownWriter, Table};
use crate::rankings::{Borda, Firsts};
use crate::record::{BenchmarkRecord, SolverRecord};
use kurobako_core::{Error, ErrorKind, Result};
use rustats::hypothesis_testings::{Alpha, MannWhitneyU};
use serde::Deserialize;
use std::cmp::Ordering;
use std::fmt;
use std::io::Write;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
pub enum Metric {
    Best,
    Auc,
    Wallclock,
}
impl Metric {
    const POSSIBLE_VALUES: &'static [&'static str] = &["best", "auc", "wallclock"];
}
impl FromStr for Metric {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "best" => Ok(Metric::Best),
            "auc" => Ok(Metric::Auc),
            "wallclock" => Ok(Metric::Wallclock),
            _ => track_panic!(ErrorKind::InvalidInput, "Unknown metric name: {:?}", s),
        }
    }
}
impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Metric::Best => write!(f, "best"),
            Metric::Auc => write!(f, "auc"),
            Metric::Wallclock => write!(f, "wallclock"),
        }
    }
}

#[derive(Debug, StructOpt, Deserialize)]
pub struct SolverRankingOptions {
    #[structopt(
        long,
        default_value = "best",
        possible_values = Metric::POSSIBLE_VALUES
    )]
    pub metrics: Vec<Metric>,
}

#[derive(Debug)]
pub struct SolverRanking {
    benchmark: BenchmarkRecord,
    options: SolverRankingOptions,
}
impl SolverRanking {
    pub fn new(benchmark: BenchmarkRecord, options: SolverRankingOptions) -> Self {
        Self { benchmark, options }
    }

    pub fn write_markdown<W: Write>(&self, mut writer: MarkdownWriter<W>) -> Result<()> {
        let mut writer = track!(writer.heading("Solver Ranking"))?;

        let problems = self.benchmark.problems();
        let metrics = self
            .options
            .metrics
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>()
            .join(" -> ");
        track!(writer.writeln("Settings:"))?;
        track!(writer.writeln(&format!("- Problems: {}", problems.len())))?;
        track!(writer.writeln(&format!("- Metrics: {}", metrics)))?;
        track!(writer.newline())?;

        let solver_ids = self.benchmark.solver_ids();
        let mut borda_ranking = Borda::new(solver_ids.iter().cloned());
        let mut firsts_ranking = Firsts::new(solver_ids.iter().cloned());
        for problem in problems.values() {
            track!(borda_ranking.try_compete(|&a, &b| {
                let a = track!(problem.fetch_solver(a))?;
                let b = track!(problem.fetch_solver(b))?;
                Ok(self.compete(a, b))
            }))?;
            track!(firsts_ranking.try_compete(|&a, &b| {
                let a = track!(problem.fetch_solver(a))?;
                let b = track!(problem.fetch_solver(b))?;
                Ok(self.compete(a, b))
            }))?;
        }

        let mut table = Table::new(
            vec![
                ColumnHeader::new("Solver", Align::Left),
                ColumnHeader::new("Borda", Align::Right),
                ColumnHeader::new("Firsts", Align::Right),
            ]
            .into_iter(),
        );
        for (i, (id, (borda, firsts))) in solver_ids
            .iter()
            .zip(borda_ranking.scores().zip(firsts_ranking.scores()))
            .enumerate()
        {
            table
                .row()
                .item(format!("({}) {}", (b'a' + i as u8) as char, id.name))
                .item(borda)
                .item(firsts);
        }
        track!(writer.write_table(&table))?;
        track!(writer.newline())?;

        Ok(())
    }

    fn compete(&self, a: &SolverRecord, b: &SolverRecord) -> Ordering {
        for metric in &self.options.metrics {
            let order = match metric {
                Metric::Best => {
                    MannWhitneyU::new(a.best_values(), b.best_values()).order(Alpha::P05)
                }
                Metric::Auc => MannWhitneyU::new(a.aucs(), b.aucs()).order(Alpha::P05),
                Metric::Wallclock => {
                    MannWhitneyU::new(a.elapsed_times(), b.elapsed_times()).order(Alpha::P05)
                }
            };
            if order != Ordering::Equal {
                return order;
            }
        }
        Ordering::Equal
    }
}
