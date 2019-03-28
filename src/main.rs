#[macro_use]
extern crate structopt;
#[macro_use]
extern crate trackable;

use kurobako::benchmark::BenchmarkSpec;
use kurobako::optimizer::OptimizerSpec;
use kurobako::optimizer_suites::{BuiltinOptimizerSuite, OptimizerSuite};
use kurobako::problem_suites::{BuiltinProblemSuite, ProblemSuite};
use kurobako::problems::BuiltinProblemSpec;
use kurobako::runner::Runner;
use kurobako::stats::{Stats, StatsSummary};
use kurobako::study::StudyRecord;
use kurobako::summary::StudySummary;
use kurobako::{Error, ErrorKind, Result};
use std::path::PathBuf;
use structopt::StructOpt as _;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Opt {
    Optimizer(OptimizerSpec),
    OptimizerSuite(BuiltinOptimizerSuite),
    Problem(BuiltinProblemSpec),
    ProblemSuite(BuiltinProblemSuite),
    Benchmark(BenchmarkSpec),
    Run,
    Summary,
    Stats(StatsOpt),
    Plot(PlotOpt),
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct StatsOpt {
    #[structopt(
        long,
        default_value = "json",
        raw(possible_values = "&[\"json\", \"markdown\"]")
    )]
    format: OutputFormat,

    #[structopt(long)]
    summary: bool,

    #[structopt(long)]
    budget: Option<usize>,
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct PlotOpt {
    #[structopt(long)]
    budget: Option<usize>,

    #[structopt(long, default_value = "plot/")]
    output_dir: PathBuf,
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum OutputFormat {
    Json,
    Markdown,
}
impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Json
    }
}
impl std::str::FromStr for OutputFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "json" => Ok(OutputFormat::Json),
            "markdown" => Ok(OutputFormat::Markdown),
            _ => track_panic!(ErrorKind::Other, "Uknown output format: {:?}", s),
        }
    }
}

fn main() -> trackable::result::MainResult {
    let opt = Opt::from_args();
    match opt {
        Opt::Optimizer(o) => {
            track!(serde_json::to_writer(std::io::stdout().lock(), &o).map_err(Error::from))?
        }
        Opt::OptimizerSuite(o) => track!(serde_json::to_writer(
            std::io::stdout().lock(),
            &o.suite().collect::<Vec<_>>()
        )
        .map_err(Error::from))?,
        Opt::Problem(p) => {
            track!(serde_json::to_writer(std::io::stdout().lock(), &p).map_err(Error::from))?
        }
        Opt::ProblemSuite(p) => track!(serde_json::to_writer(
            std::io::stdout().lock(),
            &p.problem_specs().collect::<Vec<_>>(),
        )
        .map_err(Error::from))?,
        Opt::Benchmark(b) => {
            track!(serde_json::to_writer(std::io::stdout().lock(), &b).map_err(Error::from))?
        }
        Opt::Run => {
            handle_run_command()?;
        }
        Opt::Summary => {
            handle_summary_command()?;
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

fn handle_run_command() -> Result<()> {
    let benchmark_spec: BenchmarkSpec = serde_json::from_reader(std::io::stdin().lock())?;

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    for (i, spec) in benchmark_spec.run_specs().enumerate() {
        eprintln!("# [{}/{}] {:?}", i + 1, benchmark_spec.len(), spec);
        let mut runner = Runner::new();
        match track!(runner.run(spec.optimizer, spec.problem, spec.budget)) {
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

fn handle_summary_command() -> Result<()> {
    let studies: Vec<StudyRecord> = serde_json::from_reader(std::io::stdin().lock())?;
    let mut summaries = Vec::new();
    for study in studies {
        summaries.push(StudySummary::new(&study));
    }
    serde_json::to_writer(std::io::stdout().lock(), &summaries)?;
    Ok(())
}

fn handle_stats_command(opt: StatsOpt) -> Result<()> {
    let stdin = std::io::stdin();
    let mut studies = Vec::new();
    for study in serde_json::Deserializer::from_reader(stdin.lock()).into_iter() {
        let mut study: StudyRecord = track!(study.map_err(Error::from))?;
        if let Some(budget) = opt.budget {
            study.limit_budget(budget);
        }
        studies.push(study);
    }

    let stats = Stats::new(&studies);
    if opt.summary {
        let summary = StatsSummary::new(&stats);
        match opt.format {
            OutputFormat::Json => {
                serde_json::to_writer(std::io::stdout().lock(), &summary)?;
            }
            OutputFormat::Markdown => {
                summary.write_markdown(std::io::stdout().lock())?;
            }
        }
    } else {
        match opt.format {
            OutputFormat::Json => {
                serde_json::to_writer(std::io::stdout().lock(), &stats)?;
            }
            OutputFormat::Markdown => {
                stats.write_markdown(std::io::stdout().lock())?;
            }
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
        let mut study: StudyRecord = track!(study.map_err(Error::from))?;
        if let Some(budget) = opt.budget {
            study.limit_budget(budget);
        }
        studies.push(study);
    }

    track!(kurobako::plot::plot_problems(&studies, opt.output_dir))?;
    Ok(())
}
