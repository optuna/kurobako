use crate::study::StudyRecord;
use crate::Name;
use std::collections::BTreeMap;

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
