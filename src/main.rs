#[macro_use]
extern crate structopt;

use failure::Error;
use kurobako::optimizer::OptimizerSpec;
use kurobako::problem::ProblemSpec;
use kurobako::runner::{RunSpec, Runner};
use structopt::StructOpt as _;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Opt {
    Optimizer(OptimizerSpec),
    Problem(ProblemSpec),
    Run,
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    match opt {
        Opt::Optimizer(o) => serde_json::to_writer(std::io::stdout(), &o)?,
        Opt::Problem(p) => serde_json::to_writer(std::io::stdout(), &p)?,
        Opt::Run => {
            handle_run_command()?;
        }
    }
    Ok(())
}

fn handle_run_command() -> Result<(), Error> {
    let specs: Vec<RunSpec> = serde_json::from_reader(std::io::stdin())?;
    for spec in specs {
        let mut runner = Runner::new();
        let record = runner.run(&spec.optimizer, &spec.problem, spec.budget)?;
        serde_json::to_writer(std::io::stdout(), &record)?;
    }
    Ok(())
}
