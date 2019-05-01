#[macro_use]
extern crate structopt;
#[macro_use]
extern crate trackable;

use kurobako::benchmark::BenchmarkSpec;
use kurobako::optimizer::OptimizerSpec;
use kurobako::optimizer_suites::{BuiltinOptimizerSuite, OptimizerSuite};
use kurobako::plot::PlotOptions;
use kurobako::problem_suites::{BuiltinProblemSuite, ProblemSuite};
use kurobako::problems::BuiltinProblemSpec;
use kurobako::runner::Runner;
use kurobako::stats::{Stats, StatsSummary};
use kurobako::study::StudyRecord;
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
    Run(RunOpt),
    Stats(StatsOpt),
    Plot(PlotOpt),
    PlotData(PlotDataOpt),
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct RunOpt {}

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
    budget: Option<u64>,
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct PlotOpt {
    #[structopt(long)]
    budget: Option<u64>,

    #[structopt(long, default_value = "plot/")]
    output_dir: PathBuf,

    #[structopt(flatten)]
    inner: PlotOptions,
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum PlotDataOpt {
    Scatter {
        #[structopt(long)]
        budget: Option<u64>,
    },
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
        }
        Opt::PlotData(opt) => {
            handle_plot_data_command(opt)?;
        }
    }
    Ok(())
}

fn handle_run_command(_opt: RunOpt) -> Result<()> {
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

fn handle_stats_command(opt: StatsOpt) -> Result<()> {
    let stdin = std::io::stdin();
    let mut studies = Vec::new();
    for study in serde_json::Deserializer::from_reader(stdin.lock()).into_iter() {
        match track!(study.map_err(Error::from)) {
            Err(e) => {
                eprintln!("{}", e);
            }
            Ok(study) => {
                let mut study: StudyRecord = study;
                if let Some(budget) = opt.budget {
                    study.limit_budget(budget);
                }
                studies.push(study);
            }
        }
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
        match track!(study.map_err(Error::from)) {
            Err(e) => {
                eprintln!("{}", e);
            }
            Ok(study) => {
                let mut study: StudyRecord = study;
                if let Some(budget) = opt.budget {
                    study.limit_budget(budget);
                }
                studies.push(study);
            }
        }
    }

    track!(opt.inner.plot_problems(&studies, opt.output_dir))?;
    Ok(())
}

fn handle_plot_data_command(opt: PlotDataOpt) -> Result<()> {
    let stdin = std::io::stdin();
    for study in serde_json::Deserializer::from_reader(stdin.lock()).into_iter() {
        match track!(study.map_err(Error::from)) {
            Err(e) => {
                eprintln!("{}", e);
            }
            Ok(study) => {
                let mut study: StudyRecord = study;
                match opt {
                    PlotDataOpt::Scatter { budget } => {
                        if let Some(budget) = budget {
                            study.limit_budget(budget);
                        }
                        output_scatter_data(&study);
                    }
                }
            }
        }
    }
    Ok(())
}

fn output_scatter_data(study: &StudyRecord) {
    use std::f64::NAN;

    println!(
        "# {:?}, {:?}, {:?}, {:?}, {:?}",
        study.optimizer, study.problem, study.budget, study.value_range, study.start_time
    );
    println!("# Budget Value Param...");
    for (i, trial) in study.trials.iter().enumerate() {
        print!("{} {}", i, trial.value().unwrap_or(NAN));
        for p in &trial.ask.params {
            print!(" {}", p);
        }
        println!();
    }
    println!();
    println!();
}
