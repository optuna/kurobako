// pub use self::embedded_script::{EmbeddedScriptSolver, EmbeddedScriptSolverRecipe};
// pub use self::external_program::{
//     ExternalProgramSolver, ExternalProgramSolverRecipe, SolverMessage,
// };
pub use self::message::SolverMessage;

// mod embedded_script;
mod external_program;
mod message;
