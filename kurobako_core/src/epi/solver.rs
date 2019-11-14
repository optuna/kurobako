//! EPI components for `solver`.
pub use self::embedded_script::{
    EmbeddedScriptSolver, EmbeddedScriptSolverFactory, EmbeddedScriptSolverRecipe,
};
pub use self::external_program::{
    ExternalProgramSolver, ExternalProgramSolverFactory, ExternalProgramSolverRecipe,
};
pub use self::message::SolverMessage;

mod embedded_script;
mod external_program;
mod message;
