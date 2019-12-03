#[macro_use]
extern crate trackable;

use kurobako::markdown::MarkdownWriter;
use kurobako::plot::PlotOpt;
use kurobako::problem::KurobakoProblemRecipe;
use kurobako::problem_suites::ProblemSuite;
use kurobako::report::{ReportOpt, Reporter};
use kurobako::runner::{Runner, RunnerOpt};
use kurobako::solver::KurobakoSolverRecipe;
use kurobako::study::{StudiesRecipe, StudyRecipe};
use kurobako_core::json;
use kurobako_core::Error;
use std::io;
use structopt::StructOpt;

// use kurobako::exam::ExamRecipe;
// use kurobako::multi_exam::MultiExamRecipe;
// use kurobako::plot::PlotOptions;
// use kurobako::plot_scatter::PlotScatterOptions;
// use kurobako::record::{BenchmarkRecord, StudyRecord};
// use kurobako::variable::Variable;
// use kurobako_core::{Error, Result};
// use std::path::PathBuf;

macro_rules! print_json {
    ($x:expr) => {
        track!(serde_json::to_writer(std::io::stdout().lock(), &$x).map_err(Error::from))?;
        println!();
    };
    ($x:expr, $out:expr) => {
        track!(serde_json::to_writer(&mut $out, &$x).map_err(Error::from))?;
        println!();
    };
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
#[allow(clippy::large_enum_variant)]
enum Opt {
    Solver(KurobakoSolverRecipe),
    Problem(KurobakoProblemRecipe),
    ProblemSuite(ProblemSuite),
    Study(StudyRecipe),
    Studies(StudiesRecipe),
    Run(RunnerOpt),
    Report(ReportOpt),
    Plot(PlotOpt),
    // Exam(ExamRecipe),
    // MultiExam(MultiExamRecipe),
    // Var(Variable),
}

fn main() -> trackable::result::TopLevelResult {
    let opt = Opt::from_args();

    match opt {
        Opt::Solver(x) => {
            print_json!(x);
        }
        Opt::Problem(x) => {
            print_json!(x);
        }
        Opt::ProblemSuite(p) => {
            for p in p.recipes() {
                print_json!(p);
            }
        }
        Opt::Study(x) => {
            print_json!(x);
        }
        Opt::Studies(x) => {
            for y in x.studies() {
                print_json!(y);
            }
        }
        Opt::Run(opt) => {
            track!(Runner::new(opt).run())?;
        }
        Opt::Report(opt) => {
            let studies = track!(json::load(io::stdin().lock()))?;
            let reporter = Reporter::new(studies, opt);
            let stdout = io::stdout();
            let mut stdout = stdout.lock();
            let mut writer = MarkdownWriter::new(&mut stdout);
            track!(reporter.report_all(&mut writer))?;
        }
        Opt::Plot(opt) => {
            let studies = track!(json::load(io::stdin().lock()))?;
            track!(opt.plot(&studies))?;
        }
    }

    Ok(())
}

//         Opt::Exam(p) => {
//             track!(serde_json::to_writer(std::io::stdout().lock(), &p).map_err(Error::from))?
//         }
//         Opt::MultiExam(p) => {
//             track!(serde_json::to_writer(std::io::stdout().lock(), &p).map_err(Error::from))?
//         }
//         Opt::Var(p) => {
//             track!(serde_json::to_writer(std::io::stdout().lock(), &p).map_err(Error::from))?
//         }
//     }
//     Ok(())
// }
