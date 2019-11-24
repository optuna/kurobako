use super::{execute_gnuplot, normalize_filename};
use crate::record::{ProblemRecord, StudyRecord};
use indicatif::{ProgressBar, ProgressStyle};
use kurobako_core::num::OrderedFloat;
use kurobako_core::{Error, Result};
use rustats::fundamental::{average, stddev};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use structopt::StructOpt;
use tempfile::{NamedTempFile, TempPath};

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
    pub errorbar: bool,
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
                .map(|(k, v)| (k, Solver::new(v)))
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
        let mut s = format!(
            "set title {:?} noenhanced; set ylabel {:?} noenhanced; set xlabel \"Budget\"; set grid;",
            self.problem.spec.name,
            self.problem.spec.values_domain.variables()[0].name()
        );

        let output = self.opt.output_dir.join(format!(
            "{}-{}.png",
            normalize_filename(&self.problem.spec.name),
            self.problem_id
        ));
        s += &format!(
            "set key bmargin; set terminal pngcairo size {},{}; set output {:?};",
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
        } else {
            let max_step = self
                .solvers
                .values()
                .map(|s| s.values.len())
                .max()
                .unwrap_or_else(|| unreachable!());
            let step = max_step / 10;
            if let Some(y) = self
                .solvers
                .values()
                .filter_map(|s| s.value(step).map(|v| OrderedFloat(v.avg)))
                .max()
            {
                y.0.to_string()
            } else {
                "".to_string()
            }
        }
    }

    fn ymin(&self) -> String {
        if let Some(y) = self.opt.ymin {
            y.to_string()
        } else {
            let y = self.problem.spec.values_domain.variables()[0].range().low();
            if y.is_finite() {
                y.to_string()
            } else {
                "".to_string()
            }
        }
    }

    fn xmin(&self) -> String {
        self.opt
            .xmin
            .map(|v| v.to_string())
            .unwrap_or("".to_string())
    }

    fn xmax(&self) -> String {
        self.opt
            .xmax
            .map(|v| v.to_string())
            .unwrap_or("".to_string())
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
            .map(|s| s.values.len())
            .max()
            .unwrap_or_else(|| unreachable!());
        for step in 0..max_step {
            for s in self.solvers.values() {
                if let Some(v) = s.value(step) {
                    track_write!(temp_file, "{} {} ", v.avg, v.sd)?;
                } else {
                    track_write!(temp_file, "  ")?;
                }
            }
            track_writeln!(temp_file)?;
        }

        Ok(temp_file.into_temp_path())
    }
}

#[derive(Debug)]
struct Solver {
    values: Vec<Option<Value>>,
}
impl Solver {
    fn new(studies: Vec<&StudyRecord>) -> Self {
        let study_best_values = studies
            .iter()
            .map(|study| study.best_values())
            .collect::<Vec<_>>();
        let mut best_value_stats = vec![None];
        for step in 1..studies[0].study_steps() {
            let values = study_best_values
                .iter()
                .filter_map(|x| x.range(..=step).last().map(|v| *v.1))
                .collect::<Vec<_>>();
            if values.is_empty() {
                best_value_stats.push(None);
            } else {
                let avg = average(values.iter().copied());
                let sd = stddev(values.into_iter());
                best_value_stats.push(Some(Value { avg, sd }));
            }
        }
        Self {
            values: best_value_stats,
        }
    }

    fn value(&self, step: usize) -> Option<&Value> {
        self.values.get(step).and_then(|v| v.as_ref())
    }
}

#[derive(Debug)]
struct Value {
    avg: f64,
    sd: f64,
}

#[derive(Debug)]
struct BestValues {}
