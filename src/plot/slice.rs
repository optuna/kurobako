use super::execute_gnuplot;
use crate::record::StudyRecord;
use indicatif::{ProgressBar, ProgressStyle};
use kurobako_core::Result;
use std::collections::BTreeMap;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct PlotSliceOpt {
    #[structopt(long, short = "o", default_value = "images/slice/")]
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
}
impl PlotSliceOpt {
    pub fn plot(&self, study_records: &[StudyRecord]) -> Result<()> {
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
        for param in self.instances[0].problem.spec.params_domain.variables() {
            // let data_path = track!(self.generate_data(param))?;
            // let script = track!(self.make_gnuplot_script(&data_path))?;
            // track!(execute_gnuplot(&script))?;
            // std::mem::drop(data_path);
        }
        Ok(())
    }
}
