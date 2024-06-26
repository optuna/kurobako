#[macro_use]
extern crate trackable;

use kurobako::batch_eval::BatchEvaluateOpt;
use kurobako::dataset::DatasetOpt;
use kurobako::evaluate::EvaluateOpt;
use kurobako::plot::PlotOpt;
use kurobako::problem::KurobakoProblemRecipe;
use kurobako::problem_suites::ProblemSuite;
use kurobako::report::{ReportOpt, Reporter};
use kurobako::runner::{Runner, RunnerOpt};
use kurobako::solver::KurobakoSolverRecipe;
use kurobako::spec::SpecOpt;
use kurobako::study::StudiesRecipe;
use kurobako::variable::Var;
use kurobako_core::json;
use kurobako_core::Error;
use std::io;
use structopt::StructOpt;

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
    /// Generates a solver recipe (JSON).
    Solver(KurobakoSolverRecipe),

    /// Generates a problem recipe (JSON).
    Problem(KurobakoProblemRecipe),

    /// Generates problem recipes (JSONs) belong to the specified suite.
    ProblemSuite(ProblemSuite),

    /// Generates a variable recipe (JSON).
    Var(Var),

    /// Generates study recipes (JSONs).
    Studies(StudiesRecipe),

    /// Takes study recipes (JSONs), then runs the studies and outputs the results (JSONs).
    Run(RunnerOpt),

    /// Generates a markdown report from benchmark results (JSONs).
    Report(ReportOpt),

    /// Generates visualization images from benchmark results (JSONs).
    Plot(PlotOpt),

    /// Dataset management.
    Dataset(DatasetOpt),

    /// Evaluates parameters of a problem.
    Evaluate(EvaluateOpt),

    /// Evaluates a set of parameters of a problem through stdio.
    BatchEvaluate(BatchEvaluateOpt),

    /// Show problem or solver specification.
    Spec(SpecOpt),
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
        Opt::Studies(x) => {
            for y in x.studies() {
                print_json!(y);
            }
        }
        Opt::Var(x) => {
            print_json!(x);
        }
        Opt::Run(opt) => {
            track!(Runner::new(opt).run())?;
        }
        Opt::Report(opt) => {
            let studies = track!(json::load(io::stdin().lock()))?;
            let reporter = Reporter::new(studies, opt);
            let stdout = io::stdout();
            let stdout = stdout.lock();
            track!(reporter.report_all(stdout))?;
        }
        Opt::Plot(opt) => {
            let studies = track!(json::load(io::stdin().lock()))?;
            track!(opt.plot(&studies))?;
        }
        Opt::Dataset(opt) => {
            track!(opt.run())?;
        }
        Opt::Evaluate(opt) => {
            let evaluated = track!(opt.evaluate())?;
            print_json!(evaluated);
        }
        Opt::Spec(opt) => {
            let spec = track!(opt.get_spec())?;
            print_json!(spec);
        }
        Opt::BatchEvaluate(opt) => {
            track!(opt.run())?;
        }
    }

    Ok(())
}
