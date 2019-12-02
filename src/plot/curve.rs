use super::{execute_gnuplot, normalize_filename};
use crate::record::{ProblemRecord, StudyRecord};
use indicatif::{ProgressBar, ProgressStyle};
use kurobako_core::num::OrderedFloat;
use kurobako_core::{Error, ErrorKind, Result};
use rustats::fundamental::{average, stddev};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;
use tempfile::{NamedTempFile, TempPath};

#[derive(Debug, StructOpt, PartialEq, Eq)]
#[structopt(rename_all = "kebab-case")]
pub enum Metric {
    BestValue,
    ElapsedTime,
    SolverElapsedTime,
}
impl Metric {
    const POSSIBLE_VALUES: &'static [&'static str] =
        &["best-value", "elapsed-time", "solver-elapsed-time"];
}
impl FromStr for Metric {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "best-value" => Ok(Metric::BestValue),
            "elapsed-time" => Ok(Metric::ElapsedTime),
            "solver-elapsed-time" => Ok(Metric::SolverElapsedTime),
            _ => track_panic!(ErrorKind::InvalidInput, "Unknown metric name: {:?}", s),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct PlotCurveOpt {
    #[structopt(long, short = "o", default_value = "images/curve/")]
    pub output_dir: PathBuf,

    #[structopt(long, default_value = "800")]
    pub width: usize,

    #[structopt(long, default_value = "600")]
    pub height: usize,

    #[structopt(long)]
    pub ymin: Option<f64>,

    #[structopt(long)]
    pub ymax: Option<f64>,

    #[structopt(long)]
    pub xmin: Option<f64>,

    #[structopt(long)]
    pub xmax: Option<f64>,

    #[structopt(long)]
    pub ylogscale: bool,

    #[structopt(long)]
    pub errorbar: bool,

    #[structopt(
        long,
        default_value = "best-value",
        possible_values = Metric::POSSIBLE_VALUES
    )]
    pub metric: Metric,
}
impl PlotCurveOpt {
    pub fn plot(&self, studies: &[StudyRecord]) -> Result<()> {
        let mut problems = BTreeMap::<_, Vec<_>>::new();
        for study in studies {
            problems
                .entry(track!(study.problem.id())?)
                .or_default()
                .push(study);
        }

        let pb = ProgressBar::new(problems.len() as u64);
        let template =
            "(PLOT) [{elapsed_precise}] [{pos}/{len} {percent:>3}%] [ETA {eta:>3}] {msg}";
        pb.set_style(ProgressStyle::default_bar().template(&template));

        track!(fs::create_dir_all(&self.output_dir).map_err(Error::from); self.output_dir)?;

        for (problem_id, studies) in problems {
            let problem = track!(Problem::new(problem_id, studies, self))?;
            track!(problem.plot())?;
            pb.inc(1);
        }
        pb.finish_with_message(&format!("done (dir={:?})", self.output_dir));

        Ok(())
    }
}

#[derive(Debug)]
struct Problem<'a> {
    problem_id: String,
    problem: &'a ProblemRecord,
    solvers: BTreeMap<(&'a str, String), Solver>,
    opt: &'a PlotCurveOpt,
}
impl<'a> Problem<'a> {
    fn new(
        problem_id: String,
        studies: Vec<&'a StudyRecord>,
        opt: &'a PlotCurveOpt,
    ) -> Result<Self> {
        let problem = &studies[0].problem;
        let mut solvers = BTreeMap::<_, Vec<_>>::new();
        for study in studies {
            let study_id = track!(study.id())?;
            solvers
                .entry((study.solver.spec.name.as_str(), study_id))
                .or_default()
                .push(study);
        }
        Ok(Self {
            problem_id,
            problem,
            solvers: solvers
                .into_iter()
                .map(|(k, v)| (k, Solver::new(v, opt)))
                .collect(),
            opt,
        })
    }

    fn plot(&self) -> Result<bool> {
        if self.problem.spec.values_domain.variables().len() != 1 {
            // This plot doesn't support multi-objective problems.
            return Ok(false);
        }

        let data_path = track!(self.generate_data())?;
        let script = track!(self.make_gnuplot_script(&data_path))?;
        track!(execute_gnuplot(&script))?;
        std::mem::drop(data_path);

        Ok(true)
    }

    fn make_gnuplot_script(&self, data_path: &TempPath) -> Result<String> {
        let ylabel = match self.opt.metric {
            Metric::BestValue => self.problem.spec.values_domain.variables()[0].name(),
            Metric::ElapsedTime => "Elapsed Seconds (Ask + Evaluate + Tell)",
            Metric::SolverElapsedTime => "Elapsed Seconds (Ask + Tell)",
        };

        let mut s = format!(
            "set title {:?} noenhanced; set ylabel {:?} noenhanced; set xlabel \"Budget\"; set grid;",
            self.problem.spec.name,
            ylabel
        );
        s += "set datafile missing \"NaN\";";

        if self.opt.ylogscale {
            s += "set logscale y;"
        }

        let output = self.opt.output_dir.join(format!(
            "{}-{}.png",
            normalize_filename(&self.problem.spec.name),
            self.problem_id
        ));
        s += &format!(
            "set terminal pngcairo size {},{}; set output {:?};",
            self.opt.width, self.opt.height, output
        );

        if self.opt.errorbar {
            s += "set style fill transparent solid 0.2;";
            s += "set style fill noborder;";
        }

        s += &format!(
            "plot [{}:{}] [{}:{}]",
            self.xmin(),
            self.xmax(),
            self.ymin(),
            self.ymax()
        );

        let problem_steps = self.problem.spec.steps.last();
        for i in 0..self.solvers.len() {
            if i == 0 {
                s += &format!(" {:?}", data_path);
            } else {
                s += ", \"\"";
            }
            s += &format!(
                " u ($0/{}):{} w l t columnhead lc {}",
                problem_steps,
                (i * 2) + 1,
                i + 1
            );
            if self.opt.errorbar {
                s += &format!(
                    ", \"\" u ($0/{}):(${}-${}):(${}+${}) with filledcurves notitle lc {}",
                    problem_steps,
                    (i * 2) + 1,
                    (i * 2) + 1 + 1,
                    (i * 2) + 1,
                    (i * 2) + 1 + 1,
                    i + 1
                );
            }
        }

        Ok(s)
    }

    fn ymax(&self) -> String {
        if let Some(y) = self.opt.ymax {
            y.to_string()
        } else if self.opt.metric == Metric::BestValue {
            let max_step = self
                .solvers
                .values()
                .map(|s| s.ys.len())
                .max()
                .unwrap_or_else(|| unreachable!());
            let step = max_step / 10;
            if let Some(y) = self
                .solvers
                .values()
                .filter_map(|s| s.y(step).map(|v| OrderedFloat(v.avg)))
                .max()
            {
                y.0.to_string()
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        }
    }

    fn ymin(&self) -> String {
        if let Some(y) = self.opt.ymin {
            y.to_string()
        } else if self.opt.metric == Metric::BestValue {
            let y = self.problem.spec.values_domain.variables()[0].range().low();
            if y.is_finite() {
                y.to_string()
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        }
    }

    fn xmin(&self) -> String {
        self.opt
            .xmin
            .map(|v| v.to_string())
            .unwrap_or_else(|| "".to_string())
    }

    fn xmax(&self) -> String {
        self.opt
            .xmax
            .map(|v| v.to_string())
            .unwrap_or_else(|| "".to_string())
    }

    fn generate_data(&self) -> Result<TempPath> {
        let mut temp_file = track!(NamedTempFile::new().map_err(Error::from))?;

        for (name, _) in self.solvers.keys() {
            track_write!(temp_file, "{:?} {:?} ", name, name)?;
        }
        track_writeln!(temp_file)?;

        let max_step = self
            .solvers
            .values()
            .map(|s| s.ys.len())
            .max()
            .unwrap_or_else(|| unreachable!());
        for step in 0..max_step {
            for s in self.solvers.values() {
                if let Some(v) = s.y(step) {
                    track_write!(temp_file, "{} {} ", v.avg, v.sd)?;
                } else {
                    track_write!(temp_file, "NaN NaN ")?;
                }
            }
            track_writeln!(temp_file)?;
        }

        Ok(temp_file.into_temp_path())
    }
}

#[derive(Debug)]
struct Solver {
    ys: Vec<Option<Value>>,
}
impl Solver {
    fn new(studies: Vec<&StudyRecord>, opt: &PlotCurveOpt) -> Self {
        let study_metrics = studies
            .iter()
            .map(|study| match opt.metric {
                Metric::BestValue => study.best_values(),
                Metric::ElapsedTime => study.elapsed_times(true),
                Metric::SolverElapsedTime => study.elapsed_times(false),
            })
            .collect::<Vec<_>>();
        let mut ys = vec![None];
        for step in 1..studies[0].study_steps() {
            let values = study_metrics
                .iter()
                .filter_map(|x| x.range(..=step).last().map(|v| *v.1))
                .collect::<Vec<_>>();
            if values.is_empty() {
                ys.push(None);
            } else {
                let avg = average(values.iter().copied());
                let sd = stddev(values.into_iter());
                ys.push(Some(Value { avg, sd }));
            }
        }
        Self { ys }
    }

    fn y(&self, step: usize) -> Option<&Value> {
        self.ys.get(step).and_then(|v| v.as_ref())
    }
}

#[derive(Debug)]
struct Value {
    avg: f64,
    sd: f64,
}

#[derive(Debug)]
struct BestValues {}
