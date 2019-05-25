#[macro_use]
extern crate trackable;

use kurobako::benchmark::BenchmarkSpec;
use kurobako::markdown::MarkdownWriter;
use kurobako::plot::PlotOptions;
use kurobako::problem::FullKurobakoProblemRecipe;
use kurobako::problem_suites::{KurobakoProblemSuite, ProblemSuite};
use kurobako::record::{BenchmarkRecord, StudyRecord};
use kurobako::runner::StudyRunner;
use kurobako::solver::KurobakoSolverRecipe;
use kurobako::stats::{SolverRanking, SolverRankingOptions};
use kurobako_core::{Error, Result};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Opt {
    Solver(KurobakoSolverRecipe),
    Problem(FullKurobakoProblemRecipe),
    ProblemSuite(KurobakoProblemSuite),
    Benchmark(BenchmarkSpec),
    Run(RunOpt),
    Stats(StatsOpt),
    Plot(PlotOpt),
    // PlotData(PlotDataOpt),
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct RunOpt {}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum StatsOpt {
    Ranking(SolverRankingOptions),
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct PlotOpt {
    #[structopt(long)]
    budget: Option<u64>,

    #[structopt(long, default_value = "plot-result/")]
    output_dir: PathBuf,

    #[structopt(flatten)]
    inner: PlotOptions,
}

// #[derive(Debug, StructOpt)]
// #[structopt(rename_all = "kebab-case")]
// enum PlotDataOpt {
//     Scatter {
//         #[structopt(long)]
//         budget: Option<u64>,
//     },
// }

fn main() -> trackable::result::MainResult {
    let opt = Opt::from_args();
    match opt {
        Opt::Solver(s) => {
            track!(serde_json::to_writer(std::io::stdout().lock(), &s).map_err(Error::from))?
        }
        Opt::Problem(p) => {
            track!(serde_json::to_writer(std::io::stdout().lock(), &p).map_err(Error::from))?
        }
        Opt::ProblemSuite(p) => {
            for p in p.problem_specs() {
                track!(serde_json::to_writer(std::io::stdout().lock(), &p).map_err(Error::from))?;
                println!();
            }
        }
        Opt::Benchmark(b) => {
            track!(serde_json::to_writer(std::io::stdout().lock(), &b).map_err(Error::from))?
        }
        Opt::Run(opt) => {
            handle_run_command(opt)?;
        }
        Opt::Stats(opt) => {
            handle_stats_command(opt)?;
        }
        Opt::Plot(opt) => {
            handle_plot_command(opt)?;
        } // Opt::PlotData(opt) => {
          //     handle_plot_data_command(opt)?;
          // }
    }
    Ok(())
}

fn handle_run_command(_opt: RunOpt) -> Result<()> {
    let benchmark_spec: BenchmarkSpec = serde_json::from_reader(std::io::stdin().lock())?;

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    for (i, spec) in benchmark_spec.studies().enumerate() {
        eprintln!("# [{}/{}] {:?}", i + 1, benchmark_spec.len(), spec);
        let runner = track!(StudyRunner::new(spec.solver, spec.problem, spec.runner))?;
        match track!(runner.run()) {
            Ok(record) => {
                track!(serde_json::to_writer(&mut stdout, &record).map_err(Error::from))?;
                println!();
            }
            Err(e) => {
                eprintln!("[WARN] Failed: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_stats_command(opt: StatsOpt) -> Result<()> {
    let stdin = std::io::stdin();
    let mut studies = Vec::new();
    for study in serde_json::Deserializer::from_reader(stdin.lock()).into_iter() {
        match track!(study.map_err(Error::from)) {
            Err(e) => {
                eprintln!("{}", e);
            }
            Ok(study) => {
                let study: StudyRecord = study;
                studies.push(study);
            }
        }
    }

    match opt {
        StatsOpt::Ranking(opt) => {
            let ranking = SolverRanking::new(BenchmarkRecord::new(studies), opt);
            track!(ranking.write_markdown(MarkdownWriter::new(&mut std::io::stdout().lock())))?;
        }
    }

    Ok(())
}

fn handle_plot_command(opt: PlotOpt) -> Result<()> {
    use std::fs;

    track!(fs::create_dir_all(&opt.output_dir).map_err(Error::from); opt.output_dir)?;

    let stdin = std::io::stdin();
    let mut studies = Vec::new();
    for study in serde_json::Deserializer::from_reader(stdin.lock()).into_iter() {
        match track!(study.map_err(Error::from)) {
            Err(e) => {
                eprintln!("{}", e);
            }
            Ok(study) => {
                let study: StudyRecord = study;
                // TODO
                // if let Some(budget) = opt.budget {
                //     study.limit_budget(budget);
                // }
                studies.push(study);
            }
        }
    }

    track!(opt.inner.plot_problems(&studies, opt.output_dir))?;
    Ok(())
}

// fn handle_plot_data_command(opt: PlotDataOpt) -> Result<()> {
//     let stdin = std::io::stdin();
//     for study in serde_json::Deserializer::from_reader(stdin.lock()).into_iter() {
//         match track!(study.map_err(Error::from)) {
//             Err(e) => {
//                 eprintln!("{}", e);
//             }
//             Ok(study) => {
//                 let mut study: StudyRecord = study;
//                 match opt {
//                     PlotDataOpt::Scatter { budget } => {
//                         if let Some(budget) = budget {
//                             study.limit_budget(budget);
//                         }
//                         output_scatter_data(&study);
//                     }
//                 }
//             }
//         }
//     }
//     Ok(())
// }

// fn output_scatter_data(study: &StudyRecord) {
//     use std::f64::NAN;

//     println!(
//         "# {:?}, {:?}, {:?}, {:?}, {:?}",
//         study.solver,
//         study.problem,
//         study.budget,
//         study.value_range(),
//         study.start_time
//     );
//     println!("# Budget Value Param...");
//     for (i, trial) in study.trials.iter().enumerate() {
//         print!("{} {}", i, trial.value().unwrap_or(NAN));
//         for p in &trial.ask.params {
//             use kurobako_core::parameter::ParamValue;
//             if let ParamValue::Continuous(p) = p {
//                 print!(" {}", p.get());
//             } else {
//                 unimplemented!();
//             }
//         }
//         println!();
//     }
//     println!();
//     println!();
// }
