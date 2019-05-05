pub use self::embedded_script::{EmbeddedScriptSolver, EmbeddedScriptSolverRecipe};
pub use self::external_program::{
    ExternalProgramSolver, ExternalProgramSolverRecipe, SolverMessage,
};

mod embedded_script;
mod external_program;
