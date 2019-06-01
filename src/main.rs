#[macro_use]
extern crate trackable;

use kurobako::benchmark::BenchmarkRecipe;
use kurobako::exam::ExamRecipe;
use kurobako::filter::KurobakoFilterRecipe;
use kurobako::markdown::MarkdownWriter;
use kurobako::multi_exam::MultiExamRecipe;
use kurobako::plot::PlotOptions;
use kurobako::problem::KurobakoProblemRecipe;
use kurobako::problem_suites::{KurobakoProblemSuite, ProblemSuite};
use kurobako::record::{BenchmarkRecord, StudyRecord};
use kurobako::runner::StudyRunner;
use kurobako::solver::KurobakoSolverRecipe;
use kurobako::stats::ranking::{SolverRanking, SolverRankingOptions};
use kurobako::variable::Variable;
use kurobako_core::{Error, Result};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Opt {
    Solver(KurobakoSolverRecipe),
    Problem(KurobakoProblemRecipe),
    ProblemSuite(KurobakoProblemSuite),
    Filter(KurobakoFilterRecipe),
    Exam(ExamRecipe),
    MultiExam(MultiExamRecipe),
    Benchmark(BenchmarkRecipe),
    Run(RunOpt),
    Stats(StatsOpt),
    Plot(PlotOpt),
    Var(Variable),
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

fn main() -> trackable::result::MainResult {
    env_logger::init();

    let opt = Opt::from_args();
    match opt {
        Opt::Solver(s) => {
            track!(serde_json::to_writer(std::io::stdout().lock(), &s).map_err(Error::from))?
        }
        Opt::Problem(p) => {
            track!(serde_json::to_writer(std::io::stdout().lock(), &p).map_err(Error::from))?
        }
        Opt::Filter(p) => {
            track!(serde_json::to_writer(std::io::stdout().lock(), &p).map_err(Error::from))?
        }
        Opt::Exam(p) => {
            track!(serde_json::to_writer(std::io::stdout().lock(), &p).map_err(Error::from))?
        }
        Opt::MultiExam(p) => {
            track!(serde_json::to_writer(std::io::stdout().lock(), &p).map_err(Error::from))?
        }
        Opt::Var(p) => {
            track!(serde_json::to_writer(std::io::stdout().lock(), &p).map_err(Error::from))?
        }
        Opt::ProblemSuite(p) => {
            for p in p.problem_specs() {
                track!(serde_json::to_writer(std::io::stdout().lock(), &p).map_err(Error::from))?;
                println!();
            }
        }
        Opt::Benchmark(b) => {
            for exam in b.exams() {
                track!(serde_json::to_writer(std::io::stdout().lock(), &exam).map_err(Error::from))?;
                println!();
            }
        }
        Opt::Run(opt) => {
            handle_run_command(opt)?;
        }
        Opt::Stats(opt) => {
            handle_stats_command(opt)?;
        }
        Opt::Plot(opt) => {
            handle_plot_command(opt)?;
        }
    }
    Ok(())
}

fn handle_run_command(_opt: RunOpt) -> Result<()> {
    let stdin = std::io::stdin();
    let mut exams = Vec::new();
    for exam in serde_json::Deserializer::from_reader(stdin.lock()).into_iter() {
        let exam: ExamRecipe = track!(exam.map_err(Error::from))?;
        exams.push(exam);
    }
    let total = exams.len();

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    for (i, exam) in exams.into_iter().enumerate() {
        eprintln!("# [{}/{}] {:?}", i + 1, total, exam);
        let runner = track!(StudyRunner::new(&exam.solver, &exam.problem, &exam.runner))?;
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
