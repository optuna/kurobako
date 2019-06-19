use crate::record::StudyRecord;
use kurobako_core::{Error, ErrorKind, Result};
use rustats::fundamental::{average, stddev};
use std::collections::BTreeMap;
use std::path::Path;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Metric {
    BestValue,
    AskWallclock,
}
impl Metric {
    pub fn label(&self) -> &str {
        match self {
            Metric::BestValue => "Objective Value",
            Metric::AskWallclock => "Ask Elapsed Seconds",
        }
    }
}
impl FromStr for Metric {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "best-value" => Ok(Metric::BestValue),
            "ask-wallclock" => Ok(Metric::AskWallclock),
            _ => track_panic!(ErrorKind::InvalidInput, "Unknown metric name: {:?}", s),
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct PlotOptions {
    #[structopt(long, default_value = "800")]
    pub width: usize,

    #[structopt(long, default_value = "600")]
    pub height: usize,

    #[structopt(long)]
    pub ymin: Option<f64>,

    #[structopt(long)]
    pub ymax: Option<f64>,

    #[structopt(long)]
    pub xmax: Option<f64>,

    #[structopt(long, default_value = "")]
    pub prefix: String,

    #[structopt(long)]
    pub errorbar: bool,

    #[structopt(long, default_value = "best-value")]
    pub metric: Metric,
}
impl PlotOptions {
    pub fn plot_problems<P: AsRef<Path>>(&self, studies: &[StudyRecord], dir: P) -> Result<()> {
        let mut problems = BTreeMap::new();
        for s in studies {
            problems
                .entry(s.problem.id())
                .or_insert_with(Vec::new)
                .push(s);
        }
        let problems = problems
            .into_iter()
            .map(|(problem, studies)| ProblemPlot::new(problem.name, &studies, &self.metric));
        for (i, problem) in problems.enumerate() {
            track!(problem.plot(dir.as_ref().join(format!("{}{}.dat", self.prefix, i))))?;

            let commands = self.make_gnuplot_commands(
                &problem.problem,
                problem.solvers.len(),
                dir.as_ref().join(format!("{}{}.dat", self.prefix, i)),
                dir.as_ref().join(format!("{}{}.png", self.prefix, i)),
            );
            {
                use std::process::Command;
                println!("# {}", commands);
                track!(Command::new("gnuplot")
                    .args(&["-e", &commands])
                    .output()
                    .map_err(Error::from))?;
            }
        }
        Ok(())
    }

    fn make_gnuplot_commands<P: AsRef<Path>>(
        &self,
        problem: &str,
        solvers: usize,
        input: P,
        output: P,
    ) -> String {
        let mut s = format!(
            "set title {:?} noenhanced; set ylabel \"{}\"; set xlabel \"Budget\"; set grid;",
            problem,
            self.metric.label()
        );
        s += &format!(
            "set key bmargin; set terminal pngcairo size {},{}; set output {:?};",
            self.width,
            self.height,
            output.as_ref().to_str().expect("TODO")
        );
        if self.errorbar {
            s += "set style fill transparent solid 0.15;";
            s += "set style fill noborder;";
        }
        s += &format!(
            "plot [:{}] [{}:{}]",
            self.xmax.map(|v| v.to_string()).unwrap_or("".to_string()),
            self.ymin.map(|v| v.to_string()).unwrap_or("".to_string()),
            self.ymax.map(|v| v.to_string()).unwrap_or("".to_string()),
        );
        for i in 0..solvers {
            if i == 0 {
                s += &format!(" {:?}", input.as_ref().to_str().expect("TODO"));
            } else {
                s += ", \"\"";
            }
            s += &format!(" u 0:{} w l t columnhead lc {}", (i * 2) + 1, i + 1);
            if self.errorbar {
                s += &format!(
                    ", \"\" u 0:(${}-${}):(${}+${}) with filledcurves notitle lc {}",
                    (i * 2) + 1,
                    (i * 2) + 1 + 1,
                    (i * 2) + 1,
                    (i * 2) + 1 + 1,
                    i + 1
                );
            }
        }
        s
    }
}

#[derive(Debug)]
pub struct ProblemPlot {
    problem: String,
    solvers: Vec<SolverPlot>,
}
impl ProblemPlot {
    fn new(name: &str, studies: &[&StudyRecord], metric: &Metric) -> Self {
        let mut solvers = BTreeMap::new();
        for s in studies {
            solvers
                .entry(s.solver.id())
                .or_insert_with(Vec::new)
                .push(*s);
        }
        let solvers = solvers
            .into_iter()
            .map(|(solver, studies)| SolverPlot::new(solver.name, &studies, metric))
            .collect();
        Self {
            problem: name.to_owned(),
            solvers,
        }
    }

    fn plot<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let mut f = track!(File::create(path).map_err(Error::from))?;
        writeln!(f, "# Problem: {}", self.problem)?;

        for o in &self.solvers {
            write!(f, "{:?} {:?} ", o.solver, o.solver)?;
        }
        writeln!(f)?;

        let len = self
            .solvers
            .iter()
            .map(|o| o.scores.len())
            .max()
            .expect("TODO");
        for i in 0..len {
            for o in &self.solvers {
                let s = o.score(i);
                write!(f, "{} {} ", s.avg, s.sd)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct SolverPlot {
    solver: String,
    scores: Vec<Score>,
}
impl SolverPlot {
    fn new(name: &str, studies: &[&StudyRecord], metric: &Metric) -> Self {
        let mut scores = Vec::new();
        let scorers = studies.iter().map(|s| s.scorer()).collect::<Vec<_>>();
        for i in 0..studies[0].study_budget() {
            let values = scorers.iter().map(|s| match metric {
                Metric::BestValue => s.best_value(i).unwrap_or_else(|| unimplemented!()),
                Metric::AskWallclock => s.ask_wallclock(i),
            });
            let avg = average(values.clone());
            let sd = stddev(values);
            scores.push(Score { avg, sd });
        }
        Self {
            solver: name.to_owned(),
            scores,
        }
    }

    fn score(&self, i: usize) -> &Score {
        self.scores
            .get(i)
            .unwrap_or_else(|| self.scores.last().expect("TODO"))
    }
}

#[derive(Debug)]
struct Score {
    avg: f64,
    sd: f64,
}
