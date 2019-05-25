use crate::markdown::{Align, ColumnHeader, MarkdownWriter, Table};
use crate::rankings::{Borda, Firsts};
use crate::record::BenchmarkRecord;
use kurobako_core::Result;
use rustats::fundamental::average;
use rustats::hypothesis_testings::{Alpha, MannWhitneyU};
use std::cmp::Ordering;
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Metric {
    Best,
    Auc,
    Latency,
}

#[derive(Debug)]
pub struct SolverRanking {
    benchmark: BenchmarkRecord,
}
impl SolverRanking {
    pub fn new(benchmark: BenchmarkRecord) -> Self {
        Self { benchmark }
    }

    pub fn write_markdown<W: Write>(&self, mut writer: MarkdownWriter<W>) -> Result<()> {
        let mut writer = track!(writer.heading("Solver Ranking"))?;

        let solver_ids = self.benchmark.solver_ids();
        let mut borda_ranking = Borda::new(solver_ids.iter().cloned());
        let mut firsts_ranking = Firsts::new(solver_ids.iter().cloned());
        for problem in self.benchmark.problems().values() {
            track!(borda_ranking.try_compete(|&a, &b| {
                let a = track!(problem.fetch_solver(a))?;
                let b = track!(problem.fetch_solver(b))?;

                if MannWhitneyU::new(a.best_values(), b.best_values()).test(Alpha::P05) {
                    if average(a.best_values().map(|v| v.get()))
                        < average(b.best_values().map(|v| v.get()))
                    {
                        Ok(Ordering::Less)
                    } else {
                        Ok(Ordering::Greater)
                    }
                } else {
                    Ok(Ordering::Equal)
                }
            }))?;
            track!(firsts_ranking.try_compete(|&a, &b| {
                let a = track!(problem.fetch_solver(a))?;
                let b = track!(problem.fetch_solver(b))?;

                if MannWhitneyU::new(a.best_values(), b.best_values()).test(Alpha::P05) {
                    if average(a.best_values().map(|v| v.get()))
                        < average(b.best_values().map(|v| v.get()))
                    {
                        Ok(Ordering::Less)
                    } else {
                        Ok(Ordering::Greater)
                    }
                } else {
                    Ok(Ordering::Equal)
                }
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
                .item(format!("({}) {}", i, id.name))
                .item(borda)
                .item(firsts);
        }
        track!(writer.write_table(&table))?;

        track!(writer.newline())?;
        Ok(())
    }
}
