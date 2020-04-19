//! `kurobako plot slice` command.
use super::{execute_gnuplot, normalize_filename};
use crate::record::StudyRecord;
use indicatif::{ProgressBar, ProgressStyle};
use kurobako_core::domain::Variable;
use kurobako_core::{Error, Result};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use structopt::StructOpt;
use tempfile::{NamedTempFile, TempPath};

/// Options of the `kurobako plot slice` command.
#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct PlotSliceOpt {
    /// Output directory where generated images are stored.
    #[structopt(long, short = "o", default_value = "images/slice/")]
    pub output_dir: PathBuf,

    /// Image width in pixels.
    #[structopt(long, default_value = "800")]
    pub width: usize,

    /// Image height in pixels.
    #[structopt(long, default_value = "600")]
    pub height: usize,

    /// Minimum value of Y axis.
    #[structopt(long)]
    pub ymin: Option<f64>,

    /// Maximum value of Y axis.
    #[structopt(long)]
    pub ymax: Option<f64>,

    /// Minimum value of X axis.
    #[structopt(long)]
    pub xmin: Option<f64>,

    /// Maximum value of X axis.
    #[structopt(long)]
    pub xmax: Option<f64>,
}
impl PlotSliceOpt {
    pub(crate) fn plot(&self, study_records: &[StudyRecord]) -> Result<()> {
        let mut studies = BTreeMap::new();
        for record in study_records {
            let id = track!(record.id())?;
            studies
                .entry(id)
                .or_insert_with(Study::new)
                .instances
                .push(record);
        }

        let pb =
            ProgressBar::new(studies.iter().map(|(_, s)| s.params_len()).sum::<usize>() as u64);
        let template =
            "(PLOT) [{elapsed_precise}] [{pos}/{len} {percent:>3}%] [ETA {eta:>3}] {msg}";
        pb.set_style(ProgressStyle::default_bar().template(&template));

        track!(fs::create_dir_all(&self.output_dir).map_err(Error::from); self.output_dir)?;

        for (_study_id, study) in studies {
            track!(study.plot(self))?;
            pb.inc(study.params_len() as u64);
        }

        pb.finish_with_message(&format!("done (dir={:?})", self.output_dir));

        Ok(())
    }
}

#[derive(Debug)]
struct Study<'a> {
    instances: Vec<&'a StudyRecord>,
}
impl<'a> Study<'a> {
    fn new() -> Self {
        Self {
            instances: Vec::new(),
        }
    }

    fn params_len(&self) -> usize {
        self.instances[0]
            .problem
            .spec
            .params_domain
            .variables()
            .len()
    }

    fn plot(&self, opt: &PlotSliceOpt) -> Result<()> {
        for (param_index, param) in self.instances[0]
            .problem
            .spec
            .params_domain
            .variables()
            .iter()
            .enumerate()
        {
            let data_path = track!(self.generate_data(param_index))?;
            let script = track!(self.make_gnuplot_script(param, &data_path, opt))?;
            track!(execute_gnuplot(&script))?;
            std::mem::drop(data_path);
        }
        Ok(())
    }

    fn make_gnuplot_script(
        &self,
        param: &Variable,
        data_path: &TempPath,
        opt: &PlotSliceOpt,
    ) -> Result<String> {
        let problem = &self.instances[0].problem;
        let solver = &self.instances[0].solver;
        let title = format!(
            "Problem: {}, Solver: {}",
            problem.spec.name, solver.spec.name
        );
        let mut s = format!(
            "set title {:?}; \
             set ylabel {:?}; \
             set xlabel \"Parameter: {}\"; \
             set grid;",
            title,
            problem.spec.values_domain.variables()[0].name(),
            param.name()
        );

        let output = opt.output_dir.join(format!(
            "{}-{}-{}-{}.png",
            normalize_filename(&problem.spec.name),
            normalize_filename(&solver.spec.name),
            normalize_filename(param.name()),
            track!(self.instances[0].id())?
        ));

        s += &format!(
            "set terminal pngcairo size {},{} noenhanced; set output {:?};",
            opt.width, opt.height, output
        );
        s += "set palette defined (0 'blue', 1 'grey', 2 'red');";

        s += &format!(
            "plot [{}:{}] [{}:{}] {:?} u 3:2:1 palette pt 7 notitle",
            opt.xmin.map(|v| v.to_string()).unwrap_or_default(),
            opt.xmax.map(|v| v.to_string()).unwrap_or_default(),
            opt.ymin.map(|v| v.to_string()).unwrap_or_default(),
            opt.ymax.map(|v| v.to_string()).unwrap_or_default(),
            data_path
        );

        Ok(s)
    }

    fn generate_data(&self, param_index: usize) -> Result<TempPath> {
        let mut temp_file = track!(NamedTempFile::new().map_err(Error::from))?;

        let problem_steps = self.instances[0].problem.spec.steps.last();
        for study in &self.instances {
            for trial in &study.trials {
                if let Some(v) = trial.value(problem_steps) {
                    let end_step = trial.end_step().unwrap_or_else(|| unreachable!());
                    let budget = end_step as f64 / problem_steps as f64;
                    let p = trial.params[param_index];
                    if p.is_finite() {
                        track_writeln!(temp_file, "{} {} {}", budget, v, p)?;
                    }
                }
            }
        }

        Ok(temp_file.into_temp_path())
    }
}
