use crate::study::StudyRecord;
use crate::Name;
use kurobako_core::Result;
use rustats::num::NonNanF64;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsSummary(Vec<SolverSummary>);
impl StatsSummary {
    pub fn new(stats: &Stats) -> Self {
        let mut map = BTreeMap::new();
        for p in &stats.0 {
            for o in &p.solvers {
                if !map.contains_key(&o.solver) {
                    map.insert(o.solver.clone(), SolverSummary::new(o.solver.clone()));
                }
            }
        }

        for p in &stats.0 {
            let (worst, best) = p.min_max(|o| o.best_score.avg);
            for o in &p.solvers {
                if o.best_score.avg == worst.best_score.avg {
                    map.get_mut(&o.solver).unwrap().best_score.worsts += 1;
                }
                if o.best_score.avg == best.best_score.avg {
                    map.get_mut(&o.solver).unwrap().best_score.bests += 1;
                }
            }

            let (worst, best) = p.min_max(|o| o.auc.avg);
            for o in &p.solvers {
                if o.auc.avg == worst.auc.avg {
                    map.get_mut(&o.solver).unwrap().auc.worsts += 1;
                }
                if o.auc.avg == best.auc.avg {
                    map.get_mut(&o.solver).unwrap().auc.bests += 1;
                }
            }

            let (best, worst) = p.min_max(|o| o.latency.avg);
            for o in &p.solvers {
                if o.latency.avg == worst.latency.avg {
                    map.get_mut(&o.solver).unwrap().latency.worsts += 1;
                }
                if o.latency.avg == best.latency.avg {
                    map.get_mut(&o.solver).unwrap().latency.bests += 1;
                }
            }
        }

        Self(map.into_iter().map(|(_, v)| v).collect())
    }

    pub fn write_markdown<W: Write>(&self, mut writer: W) -> Result<()> {
        writeln!(writer, "## Statistics Summary")?;
        writeln!(
            writer,
            "| solver | Best Score (o/x) | AUC (o/x) | Latency (o/x) |"
        )?;
        writeln!(
            writer,
            "|:----------|-----------------:|----------:|--------------:|"
        )?;
        for o in &self.0 {
            writeln!(
                writer,
                "| {} | {:03}/{:03} | {:03}/{:03} | {:03}/{:03} |",
                o.name.as_json(),
                o.best_score.bests,
                o.best_score.worsts,
                o.auc.bests,
                o.auc.worsts,
                o.latency.bests,
                o.latency.worsts
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SolverSummary {
    pub name: Name,
    pub best_score: VictoryStats,
    pub auc: VictoryStats,
    pub latency: VictoryStats,
}
impl SolverSummary {
    fn new(name: Name) -> Self {
        Self {
            name,
            best_score: VictoryStats::default(),
            auc: VictoryStats::default(),
            latency: VictoryStats::default(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VictoryStats {
    pub bests: usize,
    pub worsts: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stats(Vec<ProblemStats>);
impl Stats {
    pub fn new(studies: &[StudyRecord]) -> Self {
        let mut problems = BTreeMap::new();
        for s in studies {
            problems.entry(&s.problem).or_insert_with(Vec::new).push(s);
        }
        let problems = problems
            .into_iter()
            .map(|(problem, studies)| ProblemStats::new(problem, &studies))
            .collect();
        Self(problems)
    }

    pub fn write_markdown<W: Write>(&self, mut writer: W) -> Result<()> {
        writeln!(writer, "# Statistics")?;
        for p in &self.0 {
            p.write_markdown(&mut writer)?;
            writeln!(writer)?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemStats {
    pub problem: Name,
    pub solvers: Vec<SolverStats>,
}
impl ProblemStats {
    fn new(name: &Name, studies: &[&StudyRecord]) -> Self {
        let mut solvers = BTreeMap::new();
        for s in studies {
            solvers.entry(&s.solver).or_insert_with(Vec::new).push(*s);
        }
        let solvers = solvers
            .into_iter()
            .map(|(solver, studies)| SolverStats::new(solver, &studies))
            .collect();
        Self {
            problem: name.clone(),
            solvers,
        }
    }

    fn min_max<F>(&self, f: F) -> (&SolverStats, &SolverStats)
    where
        F: Fn(&SolverStats) -> f64,
    {
        let min = self
            .solvers
            .iter()
            .min_by_key(|o| NonNanF64::new(f(o)).unwrap_or_else(|e| panic!("{}", e)))
            .expect("TODO");
        let max = self
            .solvers
            .iter()
            .max_by_key(|o| NonNanF64::new(f(o)).unwrap_or_else(|e| panic!("{}", e)))
            .expect("TODO");
        (min, max)
    }

    fn write_markdown<W: Write>(&self, mut writer: W) -> Result<()> {
        writeln!(writer, "### Problem: {}", self.problem.as_json())?;
        writeln!(writer)?;
        writeln!(writer, "| Solver | Best Score (SD) | AUC (SD) | Latency |")?;
        writeln!(
            writer,
            "|:----------|----------------:|---------:|-------------:|"
        )?;
        for o in &self.solvers {
            o.write_markdown(&mut writer)?;
        }
        writeln!(writer)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SolverStats {
    pub solver: Name,
    pub best_score: BasicStats,
    pub auc: BasicStats,
    pub latency: BasicStats,
}
impl SolverStats {
    fn new(name: &Name, studies: &[&StudyRecord]) -> Self {
        let best_scores = studies.iter().map(|s| s.best_score()).collect::<Vec<_>>();
        let aucs = studies.iter().map(|s| s.auc()).collect::<Vec<_>>();
        let latencies = studies
            .iter()
            .flat_map(|s| s.ack_latencies())
            .collect::<Vec<_>>();

        Self {
            solver: name.clone(),
            best_score: BasicStats::new(&best_scores),
            auc: BasicStats::new(&aucs),
            latency: BasicStats::new(&latencies),
        }
    }

    fn write_markdown<W: Write>(&self, mut writer: W) -> Result<()> {
        write!(writer, "| {} ", self.solver.as_json())?;
        write!(
            writer,
            "| {:.3} ({:.3}) ",
            self.best_score.avg, self.best_score.sd
        )?;
        write!(writer, "| {:.3} ({:.3}) ", self.auc.avg, self.auc.sd)?;
        write!(writer, "| {:.6} ", self.latency.avg)?;
        writeln!(writer, "|")?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicStats {
    pub avg: f64,
    pub sd: f64,
}
impl BasicStats {
    fn new(xs: &[f64]) -> Self {
        let sum = xs.iter().sum::<f64>();
        let avg = sum / xs.len() as f64;
        let sd = (xs.iter().map(|&x| (x - avg).powi(2)).sum::<f64>() / xs.len() as f64).sqrt();
        Self { avg, sd }
    }
}
