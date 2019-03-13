#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;

use structopt::StructOpt as _;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Opt {
    Run(RunCommand),
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum RunCommand {
    Ackley(AckleyProblem),
}

fn main() {
    let opt = Opt::from_args();
    match opt {
        Opt::Run(command) => handle_run_command(command),
    }
}

fn handle_run_command(command: RunCommand) {
    match command {
        RunCommand::Ackley(problem) => {
            run_problem(problem);
        }
    }
}

fn run_problem<P: Problem>(problem: P) {
    use std::io::BufRead as _;

    let budget = 20; // TODO: parameterize
    println!(
        "{{\"space\":{},\"budget\":{}}}",
        serde_json::to_string(&problem.search_space()).expect("TODO"),
        budget
    );
    let stdin = std::io::stdin();
    let mut lines = stdin.lock();
    let mut line = String::new();
    for i in 0..budget {
        line.clear();
        lines.read_line(&mut line).expect("TODO");
        let xs: Vec<f64> = serde_json::from_str(&line).expect("TODO");
        let y = problem.evaluate(&xs);
        println!("{{\"score\":{},\"budget\":{}}}", y, budget - i - 1);
    }
}

#[derive(Debug, StructOpt)]
struct AckleyProblem {
    #[structopt(long, default_value = "2")]
    dim: usize,
}
impl Problem for AckleyProblem {
    fn evaluate(&self, xs: &[f64]) -> f64 {
        let dim = self.dim as f64;
        let a = 20.0;
        let b = 0.2;
        let c = 2.0 * std::f64::consts::PI;
        let d = -a * (-b * (1.0 / dim * xs.iter().map(|&x| x.powi(2)).sum::<f64>()).sqrt()).exp();
        let e = (1.0 / dim * xs.iter().map(|&x| (x * c).cos()).sum::<f64>()).exp() + a + 1f64.exp();
        d - e
    }

    fn search_space(&self) -> Vec<Distribution> {
        (0..self.dim)
            .map(|_| Distribution::Uniform {
                low: -10.0,
                high: 30.0,
            })
            .collect()
    }

    fn tags(&self) -> &[&'static str] {
        &["complicated", "oscillatory", "unimodal", "noisy"]
    }
}

trait Problem {
    // TODO: budget
    fn evaluate(&self, params: &[f64]) -> f64;
    fn search_space(&self) -> Vec<Distribution>;
    fn tags(&self) -> &[&'static str] {
        &[]
    }
    // fn is_dynamic()->bool;
    // fn dimension()->usize;
    // fn is_early_stopping()->bool;
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Distribution {
    Uniform { low: f64, high: f64 },
}
