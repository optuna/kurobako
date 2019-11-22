//! A black-box optimization benchmarking framework.
// #![warn(missing_docs)]

#[macro_use]
extern crate trackable;

macro_rules! track_write {
    ($writer:expr, $($arg:tt)*) => {
        track!(write!($writer, $($arg)*).map_err(::kurobako_core::Error::from))
    }
}

macro_rules! track_writeln {
    ($writer:expr) => {
        track!(writeln!($writer).map_err(::kurobako_core::Error::from))
    };
    ($writer:expr, $($arg:tt)*) => {
        track!(writeln!($writer, $($arg)*).map_err(::kurobako_core::Error::from))
    }
}

// pub mod exam;
// pub mod homonym;
// pub mod multi_exam;
// pub mod plot;
// pub mod plot_scatter; // TODO: merge with plot
// pub mod problem_suites;
// pub mod stats;
// pub mod variable;
pub mod markdown;
pub mod problem;
pub mod ranking;
pub mod record;
pub mod runner;
pub mod solver;
pub mod study;
pub mod time;

// mod rankings;
