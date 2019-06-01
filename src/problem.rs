use crate::exam::ExamProblemRecipe;
use crate::multi_exam::MultiExamProblemRecipe;
use kurobako_core::epi::problem::ExternalProgramProblemRecipe;
use kurobako_core::problem::{BoxProblem, ProblemRecipe};
use kurobako_core::Result;
use kurobako_problems::{deepobs, ffmpeg, lightgbm, nasbench, sigopt};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub enum KurobakoProblemRecipe {
    Command(ExternalProgramProblemRecipe),
    Sigopt(sigopt::SigoptProblemRecipe),
    Nasbench(nasbench::NasbenchProblemRecipe),
    Ffmpeg(ffmpeg::FfmpegProblemRecipe),
    Lightgbm(lightgbm::LightgbmProblemRecipe),
    Deepobs(deepobs::DeepobsProblemRecipe),
    Exam(ExamProblemRecipe),
    MultiExam(MultiExamProblemRecipe),
}
impl ProblemRecipe for KurobakoProblemRecipe {
    type Problem = BoxProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        match self {
            KurobakoProblemRecipe::Command(p) => track!(p.create_problem().map(BoxProblem::new)),
            KurobakoProblemRecipe::Sigopt(p) => track!(p.create_problem().map(BoxProblem::new)),
            KurobakoProblemRecipe::Nasbench(p) => track!(p.create_problem().map(BoxProblem::new)),
            KurobakoProblemRecipe::Ffmpeg(p) => track!(p.create_problem().map(BoxProblem::new)),
            KurobakoProblemRecipe::Lightgbm(p) => track!(p.create_problem().map(BoxProblem::new)),
            KurobakoProblemRecipe::Deepobs(p) => track!(p.create_problem().map(BoxProblem::new)),
            KurobakoProblemRecipe::Exam(p) => track!(p.create_problem().map(BoxProblem::new)),
            KurobakoProblemRecipe::MultiExam(p) => track!(p.create_problem().map(BoxProblem::new)),
        }
    }
}
