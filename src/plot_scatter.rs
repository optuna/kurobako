use crate::record::Id;
use crate::record::StudyRecord;
use kurobako_core::{Error, Result};
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
}
impl PlotScatterOptions {
    pub fn plot_problems<P: AsRef<Path>>(&self, studies: &[StudyRecord], dir: P) -> Result<()> {
        let datasets = PlotDatasets::new(studies);
        for (i, (key, dataset)) in datasets.datasets.iter().enumerate() {
            track!(dataset.plot(dir.as_ref().join(format!("{}{}.dat", self.prefix, i)), &key))?;

            let commands = self.make_gnuplot_commands(
                &key,
                dataset.studies[0].problem.spec.params_domain[key.2].name(),
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
        &(problem, _solver, _param): &(Id, Id, usize),
        param_name: &str,
        input: P,
        output: P,
    ) -> String {
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
        Ok(())
    }
}
