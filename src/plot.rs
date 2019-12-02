use crate::record::StudyRecord;
use kurobako_core::{Error, ErrorKind, Result};
use std::process::Command;
use structopt::StructOpt;

pub mod curve;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum PlotOpt {
    /// Optimization curve.
    Curve(self::curve::PlotCurveOpt),
    Slice,
}
impl PlotOpt {
    pub fn plot(&self, studies: &[StudyRecord]) -> Result<()> {
        match self {
            Self::Curve(opt) => track!(opt.plot(studies)),
            Self::Slice => unimplemented!(),
        }
    }
}

fn execute_gnuplot(script: &str) -> Result<()> {
    let output = track!(Command::new("gnuplot")
        .args(&["-e", script])
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
