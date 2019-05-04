//! **E**xternal **P**rogram **I**nterface.

pub use self::problem::{
    ExternalProgramEvaluator, ExternalProgramProblem, ExternalProgramProblemRecipe,
};

pub mod messages;

mod problem;
