use kurobako_core::parameter::{boolean, choices, int, uniform, ParamDomain, ParamValue};
use kurobako_core::problem::{
    Evaluate, EvaluatorCapability, Problem, ProblemRecipe, ProblemSpec, Values,
};
use kurobako_core::{ErrorKind, Result};
use regex;
use rustats::num::FiniteF64;
use rustats::range::MinMax;
use serde::{Deserialize, Serialize};
use serde_json;
use std::cmp;
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::str;
use std::time::Duration;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct FfmpegProblemRecipe {
    pub input_video_path: PathBuf,

    #[structopt(long)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,

    #[structopt(long)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<String>,

    #[structopt(long)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
}
impl FfmpegProblemRecipe {
    fn ffmpeg_version(&self) -> Result<String> {
        let output = track_any_err!(Command::new("ffmpeg").arg("-version").output())?;
        let stdout = track_any_err!(str::from_utf8(&output.stdout))?;
        let version = track_assert_some!(stdout.split(' ').nth(2), ErrorKind::InvalidInput);
        Ok(version.to_owned())
    }

    fn video_duration(&self) -> Result<Duration> {
        if let Some(seconds) = self.duration {
            return Ok(Duration::from_secs(cmp::max(1, seconds)));
        }

        let path = track_assert_some!(self.input_video_path.to_str(), ErrorKind::InvalidInput);
        let output = track_any_err!(Command::new("ffprobe")
            .arg(path)
            .arg("-loglevel")
            .arg("quiet")
            .arg("-show_format")
            .arg("-print_format")
            .arg("json")
            .output())?;

        #[derive(Deserialize)]
        #[serde(rename_all = "lowercase")]
        struct Info {
            format: Format,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "lowercase")]
        struct Format {
            duration: String,
        }

        let info: Info = track_any_err!(serde_json::from_slice(&output.stdout))?;
        let seconds: f64 = track_any_err!(info.format.duration.parse())?;
        Ok(Duration::from_secs(cmp::max(1, seconds.ceil() as u64)))
    }
}
impl ProblemRecipe for FfmpegProblemRecipe {
    type Problem = FfmpegProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        Ok(FfmpegProblem {
            input_video_path: self.input_video_path.clone(),
            version: track!(self.ffmpeg_version())?,
            duration: track!(self.video_duration())?,
            bitrate: self.bitrate.clone(),
            resolution: self.resolution.clone(),
        })
    }
}

#[derive(Debug)]
pub struct FfmpegProblem {
    input_video_path: PathBuf,
    version: String,
    duration: Duration,
    bitrate: Option<String>,
    resolution: Option<String>,
}
impl Problem for FfmpegProblem {
    type Evaluator = FfmpegEvaluator;

    fn specification(&self) -> ProblemSpec {
        ProblemSpec {
            name: "ffmpeg".to_owned(),
            version: Some(self.version.clone()),
            params_domain: Ffmpeg::params_domain(),
            values_domain: unsafe {
                vec![MinMax::new_unchecked(
                    FiniteF64::new_unchecked(0.0),
                    FiniteF64::new_unchecked(1.0),
                )]
            },
            evaluation_expense: unsafe { NonZeroU64::new_unchecked(self.duration.as_secs()) },
            capabilities: vec![EvaluatorCapability::Concurrent].into_iter().collect(),
        }
    }

    fn create_evaluator(&mut self, _id: ObsId) -> Result<Self::Evaluator> {
        Ok(FfmpegEvaluator {
            input_video_path: self.input_video_path.clone(),
            bitrate: self.bitrate.clone(),
            resolution: self.resolution.clone(),
        })
    }
}

#[derive(Debug)]
pub struct FfmpegEvaluator {
    input_video_path: PathBuf,
    bitrate: Option<String>,
    resolution: Option<String>,
}
impl Evaluate for FfmpegEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Values> {
        let duration = Duration::from_secs(budget.amount);
        let ffmpeg = track!(Ffmpeg::new(
            params,
            &self.input_video_path,
            duration,
            self.bitrate.as_ref(),
            self.resolution.as_ref()
        ))?;
        let ssim = track!(ffmpeg.run())?;
        budget.consumption = budget.amount;

        Ok(vec![track!(FiniteF64::new(1.0 - ssim))?])
    }
}

#[derive(Debug)]
struct Ffmpeg {
    child: Child,
}
impl Ffmpeg {
    fn new(
        params: &[ParamValue],
        input_video_path: &PathBuf,
        duration: Duration,
        bitrate: Option<&String>,
        resolution: Option<&String>,
    ) -> Result<Self> {
        fn f(key: &str) -> impl Iterator<Item = String> {
            ::std::iter::once(format!("-{}", key))
        }

        fn kv<V: ::std::fmt::Display>(key: &str, val: V) -> impl Iterator<Item = String> {
            f(key).chain(::std::iter::once(val.to_string()))
        }

        fn kv_int(key: &str, val: &ParamValue) -> impl Iterator<Item = String> {
            kv(key, val.as_discrete().unwrap())
        }

        fn kv_uniform(key: &str, val: &ParamValue) -> impl Iterator<Item = String> {
            kv(key, val.as_continuous().unwrap())
        }

        let input_video_path =
            track_assert_some!(input_video_path.to_str(), ErrorKind::InvalidInput);
        let mut args = vec![];
        args.extend(f("y"));
        args.extend(kv("t", duration.as_secs()));
        args.extend(kv("i", input_video_path));
        args.extend(kv("tune", "ssim"));
        args.extend(kv("ssim", 1));
        if let Some(bitrate) = bitrate {
            args.extend(kv("vb", bitrate));
        }
        if let Some(s) = resolution {
            args.extend(kv("s", s));
        }
        args.extend(f("an"));
        args.extend(kv_int("refs", &params[0]));
        args.extend(kv_uniform("qcomp", &params[1]));
        args.extend(kv_int("qdiff", &params[2]));
        args.extend(kv_int("me_range", &params[3]));

        let mut x264opts = vec![
            format!("b-adapt={}", params[4].as_discrete().unwrap()),
            format!("subme={}", params[5].as_discrete().unwrap()),
            format!("rc-lookahead={}", params[6].as_discrete().unwrap()),
            format!("scenecut={}", params[7].as_discrete().unwrap()),
            format!("trellis={}", params[8].as_discrete().unwrap()),
        ];
        if params[8].as_discrete().unwrap() > 0 {
            x264opts.push(format!(
                "psy-rd={},{}",
                params[9].as_continuous().unwrap(),
                params[10].as_continuous().unwrap()
            ));
        }
        let partitions = params[11..16]
            .iter()
            .map(|v| v.as_categorical().unwrap() == 1)
            .zip(["p8x8", "p4x4", "b8x8", "i8x8", "i4x4"].iter())
            .filter(|t| t.0)
            .map(|t| *t.1)
            .collect::<Vec<_>>();
        if partitions.is_empty() {
            x264opts.push("partitions=none".to_owned());
        } else {
            x264opts.push(format!("partitions={}", partitions.join(",")));
        }
        x264opts.push(format!(
            "me={}",
            ["dia", "hex", "umh", "esa"][params[16].as_categorical().unwrap()]
        ));
        x264opts.push(format!(
            "direct={}",
            ["none", "spatial", "temporal", "auto"][params[17].as_categorical().unwrap()]
        ));
        if params[18].as_categorical().unwrap() == 1 {
            x264opts.push(format!(
                "deblock={},{}",
                params[19].as_discrete().unwrap(),
                params[20].as_discrete().unwrap()
            ));
        } else {
            x264opts.push(format!("no-deblock"));
        }
        x264opts.extend(
            params[21..]
                .iter()
                .map(|v| v.as_categorical().unwrap() == 1)
                .zip(
                    [
                        "b-pyramid",
                        "no-cabac",
                        "mixed-refs",
                        "no-chroma-me",
                        "8x8dct",
                        "no-fast-pskip",
                        "no-dct-decimate",
                        "deadzone-inter",
                        "deadzone-intra",
                        "nr",
                    ]
                    .iter(),
                )
                .filter(|t| t.0)
                .map(|t| t.1.to_string()),
        );
        args.extend(kv("x264opts", x264opts.join(":")));

        args.extend(kv("vcodec", "libx264"));
        args.extend(kv("f", "null"));
        args.extend(f(""));

        let child = track_any_err!(Command::new("ffmpeg")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn())?;
        Ok(Self { child })
    }

    fn params_domain() -> Vec<ParamDomain> {
        vec![
            //
            // FFmpeg options
            int("refs", 1, 16).unwrap(), // TODO: error handling
            uniform("qcomp", 0.0, 1.0).unwrap(),
            int("qdiff", 1, 51).unwrap(),
            int("me_range", 4, 16).unwrap(),
            //
            // libx264 options
            int("b-adapt", 0, 2).unwrap(),
            int("subme", 1, 9).unwrap(),
            int("rc-lookahead", 10, 100).unwrap(),
            int("scenecut", 10, 100).unwrap(),
            int("trellis", 0, 2).unwrap(),
            uniform("psy-rd.0", 0.0, 1.0).unwrap(), // TODO: conditional
            uniform("psy-rd.1", 0.0, 1.0).unwrap(), // TODO: conditional
            boolean("p8x8"),
            boolean("p4x4"),
            boolean("b8x8"),
            boolean("i8x8"),
            boolean("i4x4"),
            choices("me", &["dia", "hex", "umh", "esa"]),
            choices("direct", &["none", "spatial", "temporal", "auto"]),
            boolean("deblock"),
            int("deblockalpha", -6, 6).unwrap(), // TODO: conditional
            int("deblockbeta", -6, 6).unwrap(),  // TODO: conditional
            boolean("b-pyramid"),
            boolean("no-cabac"),
            boolean("mixed-refs"),
            boolean("no-chroma-me"),
            boolean("8x8dct"),
            boolean("no-fast-pskip"),
            boolean("no-dct-decimate"),
            boolean("deadzone-inter"),
            boolean("deadzone-intra"),
            boolean("nr"),
        ]
    }

    fn run(self) -> Result<f64> {
        let output = track_any_err!(self.child.wait_with_output())?;
        let text = track_any_err!(str::from_utf8(&output.stderr))?;
        track_assert!(output.status.success(), ErrorKind::InvalidInput, "{}", text);

        let re = track_any_err!(regex::Regex::new("SSIM Mean Y:([0-9.]+)"))?;
        let ssim = track_assert_some!(
            re.captures_iter(text).nth(0).and_then(|t| t.get(1)),
            ErrorKind::InvalidInput
        );
        let ssim = track_any_err!(ssim.as_str().parse(); ssim)?;
        Ok(ssim)
    }
}
