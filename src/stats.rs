use crate::float::NonNanF64;
use crate::study::StudyRecord;
use crate::Name;
use failure::Fallible;
use std::collections::BTreeMap;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsSummary(Vec<OptimizerSummary>);
impl StatsSummary {
    pub fn new(stats: &Stats) -> Self {
        let mut map = BTreeMap::new();
        for p in &stats.0 {
            for o in &p.optimizers {
                if !map.contains_key(&o.optimizer) {
                    map.insert(
                        o.optimizer.clone(),
                        OptimizerSummary::new(o.optimizer.clone()),
                    );
                }
            }
        }

        for p in &stats.0 {
            let (best, worst) = p.min_max(|o| o.best_value.avg);
            map.get_mut(&best.optimizer).unwrap().best_value.bests += 1;
            map.get_mut(&worst.optimizer).unwrap().best_value.worsts += 1;

            let (worst, best) = p.min_max(|o| o.auc.avg);
            map.get_mut(&best.optimizer).unwrap().auc.bests += 1;
            map.get_mut(&worst.optimizer).unwrap().auc.worsts += 1;

            let (best, worst) = p.min_max(|o| o.latency.avg);
            map.get_mut(&best.optimizer).unwrap().latency.bests += 1;
            map.get_mut(&worst.optimizer).unwrap().latency.worsts += 1;
        }

        Self(map.into_iter().map(|(_, v)| v).collect())
    }

    pub fn write_markdown<W: Write>(&self, mut writer: W) -> Fallible<()> {
        writeln!(writer, "## Statistics Summary")?;
        writeln!(
            writer,
            "| optimizer | Best Value (o/x) | AUC (o/x) | Latency (o/x) |"
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
                o.best_value.bests,
                o.best_value.worsts,
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
pub struct OptimizerSummary {
    pub name: Name,
    pub best_value: VictoryStats,
    pub auc: VictoryStats,
    pub latency: VictoryStats,
}
impl OptimizerSummary {
    fn new(name: Name) -> Self {
        Self {
            name,
            best_value: VictoryStats::default(),
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

    pub fn write_markdown<W: Write>(&self, mut writer: W) -> Fallible<()> {
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
    pub optimizers: Vec<OptimizerStats>,
}
impl ProblemStats {
    fn new(name: &Name, studies: &[&StudyRecord]) -> Self {
        let mut optimizers = BTreeMap::new();
        for s in studies {
            optimizers
                .entry(&s.optimizer)
                .or_insert_with(Vec::new)
                .push(*s);
        }
        let optimizers = optimizers
            .into_iter()
            .map(|(optimizer, studies)| OptimizerStats::new(optimizer, &studies))
            .collect();
        Self {
            problem: name.clone(),
            optimizers,
        }
    }

    fn min_max<F>(&self, f: F) -> (&OptimizerStats, &OptimizerStats)
    where
        F: Fn(&OptimizerStats) -> f64,
    {
        let min = self
            .optimizers
            .iter()
            .min_by_key(|o| NonNanF64::new(f(o)))
            .expect("TODO");
        let max = self
            .optimizers
            .iter()
            .max_by_key(|o| NonNanF64::new(f(o)))
            .expect("TODO");
        (min, max)
    }

    fn write_markdown<W: Write>(&self, mut writer: W) -> Fallible<()> {
        writeln!(writer, "## Problem: {}", self.problem.as_json())?;
        writeln!(writer)?;
        writeln!(
            writer,
            "| Optimizer | Best Value (SD) | AUC (SD) | Latency (SD) |"
        )?;
        writeln!(
            writer,
            "|:----------|----------------:|---------:|-------------:|"
        )?;
        for o in &self.optimizers {
            o.write_markdown(&mut writer)?;
        }
        writeln!(writer)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizerStats {
    pub optimizer: Name,
    pub best_value: BasicStats,
    pub auc: BasicStats,
    pub latency: BasicStats,
}
impl OptimizerStats {
    fn new(name: &Name, studies: &[&StudyRecord]) -> Self {
        let best_values = studies
            .iter()
            .map(|s| s.normalized_best_value())
            .collect::<Vec<_>>();
        let aucs = studies.iter().map(|s| s.auc()).collect::<Vec<_>>();
        let latencies = studies
            .iter()
            .flat_map(|s| s.ack_latencies())
            .collect::<Vec<_>>();

        Self {
            optimizer: name.clone(),
            best_value: BasicStats::new(&best_values),
            auc: BasicStats::new(&aucs),
            latency: BasicStats::new(&latencies),
        }
    }

    fn write_markdown<W: Write>(&self, mut writer: W) -> Fallible<()> {
        write!(writer, "| {} ", self.optimizer.as_json())?;
        write!(
            writer,
            "| {:.3} ({:.3}) ",
            self.best_value.avg, self.best_value.sd
        )?;
        write!(writer, "| {:.3} ({:.3}) ", self.auc.avg, self.auc.sd)?;
        write!(
            writer,
            "| {:.6} ({:.6}) ",
            self.latency.avg, self.latency.sd
        )?;
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
