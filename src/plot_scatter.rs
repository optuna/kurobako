use crate::record::Id;
use crate::record::StudyRecord;
use kurobako_core::{Error, Result};
use rusty_machine::learning::gp;
use rusty_machine::learning::SupModel;
use rusty_machine::linalg::Matrix;
use rusty_machine::linalg::Vector;
use std::collections::BTreeMap;
use std::path::Path;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct PlotScatterOptions {
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
    pub gp: bool,
}
impl PlotScatterOptions {
    pub fn plot_problems<P: AsRef<Path>>(&self, studies: &[StudyRecord], dir: P) -> Result<()> {
        let datasets = PlotDatasets::new(studies);
        for (i, (key, dataset)) in datasets.datasets.iter().enumerate() {
            track!(dataset.plot(
                dir.as_ref().join(format!("{}{}.dat", self.prefix, i)),
                &key,
                self.gp
            ))?;

            let commands = self.make_gnuplot_commands(
                &key,
                dataset,
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
        &(problem, _solver, param_index): &(Id, Id, usize),
        dataset: &PlotDataset,
        input: P,
        output: P,
    ) -> String {
        let param_name = dataset.studies[0].problem.spec.params_domain[param_index].name();

        let mut s = format!(
            "set title {:?} noenhanced; set ylabel \"Objective Value\"; set xlabel \"Parameter: {}\"; set grid;",
            problem.name, param_name
        );
        s += &format!(
            "set terminal pngcairo size {},{}; set output {:?};",
            self.width,
            self.height,
            output.as_ref().to_str().expect("TODO")
        );
        s += "set palette defined (0 'blue', 1 'grey', 2 'red');";

        s += &format!(
            "plot [:{}] [{}:{}] {:?} u 3:2:1 palette pt 7 notitle",
            self.xmax.map(|v| v.to_string()).unwrap_or("".to_string()),
            self.ymin.map(|v| v.to_string()).unwrap_or("".to_string()),
            self.ymax.map(|v| v.to_string()).unwrap_or("".to_string()),
            input.as_ref().to_str().expect("TODO")
        );

        if self.gp {
            s += &format!(
                ", {:?} index 1 u 2:1 with lines notitle",
                input.as_ref().to_str().expect("TODO")
            );
        }

        s
    }
}

// TODO: rename
#[derive(Debug)]
pub struct PlotDatasets<'a> {
    datasets: BTreeMap<(Id<'a>, Id<'a>, usize), PlotDataset<'a>>,
}
impl<'a> PlotDatasets<'a> {
    pub fn new(studies: &'a [StudyRecord]) -> Self {
        let mut datasets = BTreeMap::<_, Vec<_>>::new();
        for s in studies {
            for p in 0..s.problem.spec.params_domain.len() {
                let key = (s.problem.id(), s.solver.id(), p);
                datasets.entry(key).or_default().push(s);
            }
        }
        Self {
            datasets: datasets
                .into_iter()
                .map(|(k, studies)| (k, PlotDataset { studies }))
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct PlotDataset<'a> {
    studies: Vec<&'a StudyRecord>,
}
impl<'a> PlotDataset<'a> {
    fn plot<P: AsRef<Path>>(
        &self,
        path: P,
        &(problem, solver, param): &(Id<'a>, Id<'a>, usize),
        gp: bool,
    ) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let mut f = track!(File::create(path).map_err(Error::from))?;
        writeln!(
            f,
            "# Problem={:?}, Solver={:?}, Param={}",
            problem,
            solver,
            self.studies[0].problem.spec.params_domain[param].name()
        )?;

        for study in &self.studies {
            for (budget, trial) in study.complete_trials() {
                let v = trial.evaluate.values[0]; // TODO: support multi-objective
                let p = track!(trial.ask.params[param].to_json_value())?;
                let budget = budget as f64 / study.trial_budget() as f64;
                writeln!(f, "{} {} {}", budget, v, p)?;
            }
        }

        if gp {
            writeln!(f, "\n")?;
            for (p, v) in track!(self.predict(param))? {
                writeln!(f, "{} {}", v, p)?;
            }
        }
        Ok(())
    }

    fn predict(&self, param: usize) -> Result<impl Iterator<Item = (f64, f64)>> {
        let mut train_data = Vec::<f64>::new();
        let mut target = Vec::<f64>::new();

        for study in &self.studies {
            for (_budget, trial) in study.complete_trials() {
                let v = trial.evaluate.values[0].get(); // TODO: support multi-objective
                let p = trial.ask.params[param].to_f64();
                train_data.push(p);
                target.push(v);
            }
        }

        let mut gaussp = gp::GaussianProcess::default();
        gaussp.noise = 1e-3f64;

        let train_data = Matrix::new(train_data.len(), 1, train_data);
        let target = Vector::new(target);
        track_any_err!(gaussp.train(&train_data, &target))?;

        let mut params = Vec::new();
        let range = self.studies[0].problem.spec.params_domain[param].range();
        let width = range.width();
        for i in 0..100 {
            let p = range.low + (width / 100.0) * i as f64;
            params.push(p);
        }
        let predicted = track_any_err!(gaussp.predict(&Matrix::new(params.len(), 1, params)))?;
        Ok(predicted
            .into_vec()
            .into_iter()
            .enumerate()
            .map(move |(i, v)| (range.low + (width / 100.0) * i as f64, v)))
    }
}
