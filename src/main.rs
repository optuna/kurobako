#[macro_use]
extern crate structopt;

use failure::Error;
use kurobako::optimizer::OptimizerSpec;
use kurobako::problems::BuiltinProblemSpec;
use kurobako::runner::{RunSpec, Runner};
use kurobako::study::StudyRecord;
use kurobako::summary::StudySummary;
use structopt::StructOpt as _;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Opt {
    Optimizer(OptimizerSpec),
    Problem(BuiltinProblemSpec),
    Run,
    Summary,
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    match opt {
        Opt::Optimizer(o) => serde_json::to_writer(std::io::stdout().lock(), &o)?,
        Opt::Problem(p) => serde_json::to_writer(std::io::stdout().lock(), &p)?,
        Opt::Run => {
            handle_run_command()?;
        }
        Opt::Summary => {
            handle_summary_command()?;
        }
    }
    Ok(())
}

fn handle_run_command() -> Result<(), Error> {
    let specs: Vec<RunSpec> = serde_json::from_reader(std::io::stdin().lock())?;

    // TODO: `stream`
    let mut records = Vec::new();
    for spec in specs {
        let mut runner = Runner::new();
        let record = runner.run(&spec.optimizer, &spec.problem, spec.budget)?;
        records.push(record);
    }
    serde_json::to_writer(std::io::stdout().lock(), &records)?;
    Ok(())
}

fn handle_summary_command() -> Result<(), Error> {
    let studies: Vec<StudyRecord> = serde_json::from_reader(std::io::stdin().lock())?;
    let mut summaries = Vec::new();
    for study in studies {
        summaries.push(StudySummary::new(&study));
    }
    serde_json::to_writer(std::io::stdout().lock(), &summaries)?;
    Ok(())
}
