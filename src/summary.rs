use crate::study::StudyRecord;
use crate::trial::TrialRecord;
use crate::Name;

#[derive(Debug, Serialize, Deserialize)]
pub struct StudySummary {
    pub optimizer: Name,
    pub problem: Name,
    pub best: Option<TrialSummary>,
    pub trials: usize,
    pub elapsed_time: f64,
}
impl StudySummary {
    pub fn new(study: &StudyRecord) -> Self {
        Self {
            optimizer: study.optimizer.clone(),
            problem: study.problem.clone(),
            best: study.best_trial().map(TrialSummary::new),
            trials: study.trials.len(),
            elapsed_time: study.elapsed_time(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrialSummary {
    pub params: Vec<f64>,
    pub value: f64,
}
impl TrialSummary {
    fn new(trial: &TrialRecord) -> Self {
        let params = trial.ask.params.clone();
        let value = trial.value().unwrap_or_else(|| panic!("never fails"));
        Self { params, value }
    }
}
