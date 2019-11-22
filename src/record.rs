pub use self::study::{StudyRecord, StudyRecordBuilder};
pub use self::trial::{EvaluationRecord, TrialRecord, TrialRecordBuilder};

mod study;
mod trial;

// use kurobako_core::json::JsonValue;
// use kurobako_core::{Error, Result};
// use serde_json;
// use std::fs;
// use std::io::BufReader;
// use std::path::Path;

// pub use self::benchmark::{BenchmarkRecord, ProblemRecord, SolverRecord};
// pub use self::study::{Id, RecipeAndSpec, StudyRecord};
// pub use self::trial::{AskRecord, EvaluateRecord, TellRecord, TrialRecord};

// mod benchmark;
// mod trial;

// pub fn load_studies<P: AsRef<Path>>(dir: P) -> Result<Vec<StudyRecord>> {
//     let mut studies = Vec::new();
//     for entry in track!(fs::read_dir(dir).map_err(Error::from))? {
//         let entry = track!(entry.map_err(Error::from))?;
//         let path = entry.path();
//         if path.is_file() {
//             let file = track!(fs::File::open(path).map_err(Error::from))?;
//             for study in serde_json::Deserializer::from_reader(BufReader::new(file)).into_iter() {
//                 studies.push(track!(study.map_err(Error::from))?);
//             }
//         }
//     }
//     Ok(studies)
// }
