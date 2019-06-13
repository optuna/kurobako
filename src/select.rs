use crate::record::StudyRecord;
use std::collections::HashSet;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct SelectOpt {
    #[structopt(long)]
    pub problems: Vec<String>,
}
impl SelectOpt {
    pub fn build(&self) -> Selector {
        Selector {
            problems: self.problems.iter().cloned().collect(),
        }
    }
}

#[derive(Debug)]
pub struct Selector {
    problems: HashSet<String>,
}
impl Selector {
    pub fn is_selected(&self, study: &StudyRecord) -> bool {
        if !self.problems.is_empty() && !self.problems.contains(&study.problem.spec.name) {
            return false;
        }
        true
    }
}
