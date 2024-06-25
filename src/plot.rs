//! `kurobako plot` command.
use crate::record::StudyRecord;
use kurobako_core::{Error, ErrorKind, Result};
use std::process::Command;
use structopt::StructOpt;

pub mod curve;
pub mod pareto_front;
pub mod slice;

/// Options of the `kurobako plot` command.
#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum PlotOpt {
    /// Generates optimization curve plots.
    Curve(self::curve::PlotCurveOpt),

    /// Generates slice plots.
    Slice(self::slice::PlotSliceOpt),

    /// Generates 2D pareto front plots.
    ParetoFront(self::pareto_front::PlotParetoFrontOpt),
}
impl PlotOpt {
    /// Plots a graph.
    pub fn plot(&self, studies: &[StudyRecord]) -> Result<()> {
        match self {
            Self::Curve(opt) => track!(opt.plot(studies)),
            Self::Slice(opt) => track!(opt.plot(studies)),
            Self::ParetoFront(opt) => track!(opt.plot(studies)),
        }
    }
}

fn execute_gnuplot(script: &str) -> Result<()> {
    let output = track!(Command::new("gnuplot")
        .args(["-e", script])
        .output()
        .map_err(Error::from))?;
    if !output.status.success() {
        if let Ok(err) = String::from_utf8(output.stderr) {
            track_panic!(ErrorKind::Other, "Gnuplot error: {}", err);
        }
    }
    Ok(())
}

fn normalize_filename(s: &str) -> String {
    let mut t = String::new();
    let mut replaced = false;
    for c in s.to_ascii_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            replaced = false;
            t.push(c);
        } else if !replaced {
            replaced = true;
            t.push('-');
        }
    }
    t.trim_matches('-').to_owned()
}
