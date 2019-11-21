#[macro_use]
extern crate trackable;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
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

    let runners = studies
        .iter()
        .map(|study| track!(StudyRunner::new(study, &mpb)).map(Some))
        .collect::<Result<Vec<_>>>()?;

    let pb = mpb.add(ProgressBar::new(runners.len() as u64));

    let runners = Arc::new(Mutex::new(runners));
    let study_index = Arc::new(AtomicUsize::new(0));

    let pb_style = ProgressStyle::default_bar()
        .template("!! [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .progress_chars("##-");
    pb.set_style(pb_style.clone());
    pb.tick();
    let pb = Arc::new(pb);

    let pb_style = ProgressStyle::default_bar()
        .template("-- [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .progress_chars("##-");

    let (tx, rx) = mpsc::channel();
    for _ in 0..opt.parallelism.get() {
        let pb = Arc::clone(&pb);
        let tx = tx.clone();
        let runners = Arc::clone(&runners);
        let study_index = Arc::clone(&study_index);
        thread::spawn(move || loop {
            let i = study_index.fetch_add(1, atomic::Ordering::SeqCst);
            let study = {
                let mut runners = runners.lock().unwrap_or_else(|e| panic!("{}", e));
                if i >= runners.len() {
                    break;
                }
                runners[i].take().unwrap_or_else(|| unreachable!())
            };
            pb.inc(1);
            let result = track!(study.run());
            let _ = tx.send(result);
        });
    }
    std::mem::drop(tx);

    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    thread::spawn(move || {
        mpb.join().unwrap_or_else(|e| panic!("{}", e));
        std::mem::drop(stop_tx);
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
    pb.finish_with_message("done");

    let _ = stop_rx.recv();

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
