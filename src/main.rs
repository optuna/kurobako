#[macro_use]
extern crate trackable;

use indicatif::MultiProgress;
use kurobako::problem::KurobakoProblemRecipe;
use kurobako::runner::StudyRunner;
use kurobako::solver::KurobakoSolverRecipe;
use kurobako::study::{StudiesRecipe, StudyRecipe};
use kurobako_core::{Error, Result};
use std::num::NonZeroUsize;
use std::sync::atomic::{self, AtomicUsize};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use structopt::StructOpt;

// use kurobako::exam::ExamRecipe;
// use kurobako::markdown::MarkdownWriter;
// use kurobako::multi_exam::MultiExamRecipe;
// use kurobako::plot::PlotOptions;
// use kurobako::plot_scatter::PlotScatterOptions;
// use kurobako::problem_suites::{KurobakoProblemSuite, ProblemSuite};
// use kurobako::record::{BenchmarkRecord, StudyRecord};
// use kurobako::stats::ranking::{SolverRanking, SolverRankingOptions};
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
enum Opt {
    Solver(KurobakoSolverRecipe),
    Problem(KurobakoProblemRecipe),
    Study(StudyRecipe),
    Studies(StudiesRecipe),
    Run(RunOpt),
    // ProblemSuite(KurobakoProblemSuite),
    // Exam(ExamRecipe),
    // MultiExam(MultiExamRecipe),
    // Stats(StatsOpt),
    // Plot(PlotOpt),
    // PlotScatter(PlotScatterOpt),
    // Var(Variable),
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct RunOpt {
    #[structopt(long, default_value = "1")]
    parallelism: NonZeroUsize,
}

// #[derive(Debug, StructOpt)]
// #[structopt(rename_all = "kebab-case")]
// enum StatsOpt {
//     Ranking(SolverRankingOptions),
// }

// #[derive(Debug, StructOpt)]
// #[structopt(rename_all = "kebab-case")]
// struct PlotOpt {
//     #[structopt(long)]
//     budget: Option<u64>,

//     #[structopt(long, short = "o", default_value = "plot-result/")]
//     output_dir: PathBuf,

//     #[structopt(flatten)]
//     inner: PlotOptions,
// }

// #[derive(Debug, StructOpt)]
// #[structopt(rename_all = "kebab-case")]
// struct PlotScatterOpt {
//     #[structopt(long)]
//     budget: Option<u64>,

//     #[structopt(long, short = "o", default_value = "plot-result/")]
//     output_dir: PathBuf,

//     #[structopt(flatten)]
//     inner: PlotScatterOptions,
// }

fn main() -> trackable::result::TopLevelResult {
    env_logger::init();

    let opt = Opt::from_args();

    match opt {
        Opt::Solver(x) => {
            print_json!(x);
        }
        Opt::Problem(x) => {
            print_json!(x);
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
            handle_run_command(opt)?;
        }
    }

    Ok(())
}

fn handle_run_command(opt: RunOpt) -> Result<()> {
    let mpb = MultiProgress::new();

    let stdin = std::io::stdin();
    let mut studies = Vec::new();
    for study in serde_json::Deserializer::from_reader(stdin.lock()).into_iter() {
        let study: StudyRecipe = track!(study.map_err(Error::from))?;
        studies.push(study);
    }

    let studies = studies
        .iter()
        .map(|study| track!(StudyRunner::new(study, &mpb)).map(Some))
        .collect::<Result<Vec<_>>>()?;

    let studies = Arc::new(Mutex::new(studies));
    let study_index = Arc::new(AtomicUsize::new(0));

    let (tx, rx) = mpsc::channel();
    for _ in 0..opt.parallelism.get() {
        let tx = tx.clone();
        let studies = Arc::clone(&studies);
        let study_index = Arc::clone(&study_index);
        thread::spawn(move || loop {
            let i = study_index.fetch_add(1, atomic::Ordering::SeqCst);
            let mut studies = studies.lock().unwrap_or_else(|e| panic!("{}", e));
            if i < studies.len() {
                break;
            }

            let study = studies[i].take().unwrap_or_else(|| unreachable!());
            let result = track!(study.run());
            let _ = tx.send(result);
        });
    }

    thread::spawn(move || {
        mpb.join_and_clear().unwrap_or_else(|e| panic!("{}", e));
    });

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    while let Ok(result) = rx.recv() {
        match result {
            Ok(record) => {
                print_json!(record, stdout);
            }
            Err(e) => {
                unimplemented!("{}", e);
            }
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
//         Opt::ProblemSuite(p) => {
//             for p in p.problem_specs() {
//                 track!(serde_json::to_writer(std::io::stdout().lock(), &p).map_err(Error::from))?;
//                 println!();
//             }
//         }
//         Opt::Stats(opt) => {
//             handle_stats_command(opt)?;
//         }
//         Opt::Plot(opt) => {
//             handle_plot_command(opt)?;
//         }
//         Opt::PlotScatter(opt) => {
//             handle_plot_scatter_command(opt)?;
//         }
//     }
//     Ok(())
// }

// fn handle_stats_command(opt: StatsOpt) -> Result<()> {
//     let stdin = std::io::stdin();
//     let mut studies = Vec::new();
//     for study in serde_json::Deserializer::from_reader(stdin.lock()).into_iter() {
//         match track!(study.map_err(Error::from)) {
//             Err(e) => {
//                 eprintln!("{}", e);
//             }
//             Ok(study) => {
//                 let study: StudyRecord = study;
//                 studies.push(study);
//             }
//         }
//     }

//     match opt {
//         StatsOpt::Ranking(opt) => {
//             let ranking = SolverRanking::new(BenchmarkRecord::new(studies), opt);
//             track!(ranking.write_markdown(MarkdownWriter::new(&mut std::io::stdout().lock())))?;
//         }
//     }

//     Ok(())
// }

// fn handle_plot_command(opt: PlotOpt) -> Result<()> {
//     use std::fs;

//     track!(fs::create_dir_all(&opt.output_dir).map_err(Error::from); opt.output_dir)?;

//     let stdin = std::io::stdin();
//     let mut studies = Vec::new();
//     for study in serde_json::Deserializer::from_reader(stdin.lock()).into_iter() {
//         match track!(study.map_err(Error::from)) {
//             Err(e) => {
//                 eprintln!("{}", e);
//             }
//             Ok(study) => {
//                 let study: StudyRecord = study;
//                 // TODO
//                 // if let Some(budget) = opt.budget {
//                 //     study.limit_budget(budget);
//                 // }
//                 studies.push(study);
//             }
//         }
//     }

//     track!(opt.inner.plot_problems(&studies, opt.output_dir))?;
//     Ok(())
// }

// fn handle_plot_scatter_command(opt: PlotScatterOpt) -> Result<()> {
//     use std::fs;

//     track!(fs::create_dir_all(&opt.output_dir).map_err(Error::from); opt.output_dir)?;

//     let stdin = std::io::stdin();
//     let mut studies = Vec::new();
//     for study in serde_json::Deserializer::from_reader(stdin.lock()).into_iter() {
//         match track!(study.map_err(Error::from)) {
//             Err(e) => {
//                 eprintln!("{}", e);
//             }
//             Ok(study) => {
//                 let study: StudyRecord = study;
//                 // TODO
//                 // if let Some(budget) = opt.budget {
//                 //     study.limit_budget(budget);
//                 // }
//                 studies.push(study);
//             }
//         }
//     }

//     track!(opt.inner.plot_problems(&studies, opt.output_dir))?;
//     Ok(())
// }
