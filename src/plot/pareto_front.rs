//! `kurobako plot pareto-front` command.
#![allow(clippy::format_push_string)]
use super::{execute_gnuplot, normalize_filename};
use crate::record::StudyRecord;
use indicatif::{ProgressBar, ProgressStyle};
use kurobako_core::{Error, ErrorKind, Result};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use structopt::StructOpt;
use tempfile::{NamedTempFile, TempPath};

/// Options of the `kurobako plot pareto-front` command.
#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct PlotParetoFrontOpt {
    /// Output directory where generated images are stored.
    #[structopt(long, short = "o", default_value = "images/pareto_front/")]
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
impl PlotParetoFrontOpt {
    pub(crate) fn plot(&self, study_records: &[StudyRecord]) -> Result<()> {
        let mut studies = BTreeMap::new();
        for record in study_records {
            track_assert_eq!(
                record.problem.spec.values_domain.variables().len(),
                2,
                ErrorKind::InvalidInput
            );

            let id = track!(record.id())?;
            studies
                .entry(id)
                .or_insert_with(Study::new)
                .instances
                .push(record);
        }

        let pb = ProgressBar::new(studies.len() as u64);
        let template =
            "(PLOT) [{elapsed_precise}] [{pos}/{len} {percent:>3}%] [ETA {eta:>3}] {msg}";
        pb.set_style(ProgressStyle::default_bar().template(template));

        track!(fs::create_dir_all(&self.output_dir).map_err(Error::from); self.output_dir)?;

        for (_study_id, study) in studies {
            track!(study.plot(self))?;
            pb.inc(1);
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

    fn plot(&self, opt: &PlotParetoFrontOpt) -> Result<()> {
        let data_path = track!(self.generate_data())?;
        let script = track!(self.make_gnuplot_script(&data_path, opt))?;
        track!(execute_gnuplot(&script))?;
        std::mem::drop(data_path);

        Ok(())
    }

    fn make_gnuplot_script(
        &self,
        data_path: &TempPath,
        opt: &PlotParetoFrontOpt,
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
             set xlabel {:?}; \
             set grid;",
            title,
            problem.spec.values_domain.variables()[1].name(),
            problem.spec.values_domain.variables()[0].name(),
        );

        let output = opt.output_dir.join(format!(
            "{}-{}-{}.png",
            normalize_filename(&problem.spec.name),
            normalize_filename(&solver.spec.name),
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

    fn generate_data(&self) -> Result<TempPath> {
        let mut temp_file = track!(NamedTempFile::new().map_err(Error::from))?;

        let problem_steps = self.instances[0].problem.spec.steps.last();
        for study in &self.instances {
            eprintln!("trials: {:?}", study.trials.len());
            dbg!(problem_steps);
            let mut c = 0;
            for trial in &study.trials {
                if let Some(vs) = trial.values(problem_steps) {
                    c += 1;
                    let end_step = trial.end_step().unwrap_or_else(|| unreachable!());
                    let budget = end_step as f64 / problem_steps as f64;
                    track_writeln!(temp_file, "{} {} {}", budget, vs[1], vs[0])?;
                }
            }
            dbg!(c);
        }

        Ok(temp_file.into_temp_path())
    }
}
