#! /usr/bin/env python3
import argparse
from kurobako import solver
from kurobako.solver.optuna import OptunaSolverFactory
import optuna

##
## (1) Parse command-line arguments
##
parser = argparse.ArgumentParser()
parser.add_argument("--sampler", choices=["tpe", "random", "skopt", "cma-es"], default="tpe")
parser.add_argument("--tpe-startup-trials", type=int, default=10)
parser.add_argument("--tpe-ei-candidates", type=int, default=24)
parser.add_argument("--tpe-prior-weight", type=float, default=1.0)
parser.add_argument("--skopt-base-estimator", choices=["GP", "RF", "ET", "GBRT"], default="GP")
parser.add_argument("--pruner", choices=["median", "asha", "nop", "hyperband"], default="median")
parser.add_argument("--median-startup-trials", type=int, default=5)
parser.add_argument("--median-warmup-steps", type=int, default=0)
parser.add_argument("--asha-min-resource", type=int, default=1)
parser.add_argument("--asha-reduction-factor", type=int, default=4)
parser.add_argument("--hyperband-min-resource", type=int, default=1)
parser.add_argument("--hyperband-reduction-factor", type=int, default=3)
parser.add_argument("--hyperband-n-brackets", type=int, default=4)
parser.add_argument("--loglevel", choices=["debug", "info", "warning", "error"])
parser.add_argument("--direction", choices=["minimize", "maximize"], default="minimize")
parser.add_argument("--use-discrete-uniform", action="store_true")

args = parser.parse_args()


##
## (2) Define `create_study` method
##
def create_study(seed):
    if args.loglevel == "debug":
        optuna.logging.set_verbosity(optuna.logging.DEBUG)
    elif args.loglevel == "info":
        optuna.logging.set_verbosity(optuna.logging.INFO)
    elif args.loglevel == "warning":
        optuna.logging.set_verbosity(optuna.logging.WARNING)
    elif args.loglevel == "error":
        optuna.logging.set_verbosity(optuna.logging.ERROR)

    if args.sampler == "random":
        sampler = optuna.samplers.RandomSampler(seed=seed)
    elif args.sampler == "tpe":
        sampler = optuna.samplers.TPESampler(
            n_startup_trials=args.tpe_startup_trials,
            n_ei_candidates=args.tpe_ei_candidates,
            prior_weight=args.tpe_prior_weight,
            seed=seed,
        )
    elif args.sampler == "skopt":
        skopt_kwargs = {"base_estimator": args.skopt_base_estimator}
        sampler = optuna.integration.SkoptSampler(skopt_kwargs=skopt_kwargs)
    elif args.sampler == "cma-es":
        sampler = optuna.integration.CmaEsSampler(seed=seed)
    else:
        raise ValueError("Unknown sampler: {}".format(args.sampler))

    if args.pruner == "median":
        pruner = optuna.pruners.MedianPruner(
            n_startup_trials=args.median_startup_trials, n_warmup_steps=args.median_warmup_steps
        )
    elif args.pruner == "asha":
        pruner = optuna.pruners.SuccessiveHalvingPruner(
            min_resource=args.asha_min_resource, reduction_factor=args.asha_reduction_factor
        )
    elif args.pruner == "hyperband":
        pruner = optuna.pruners.HyperbandPruner(
            min_resource=args.hyperband_min_resource,
            reduction_factor=args.hyperband_reduction_factor,
            n_brackets=args.hyperband_n_brackets,
        )
    elif args.pruner == "nop":
        pruner = optuna.pruners.NopPruner()
    else:
        raise ValueError("Unknown pruner: {}".format(args.pruner))

    return optuna.create_study(sampler=sampler, pruner=pruner, direction=args.direction)


##
## (3) Solve
##
if __name__ == "__main__":
    factory = OptunaSolverFactory(create_study, use_discrete_uniform=args.use_discrete_uniform)
    runner = solver.SolverRunner(factory)
    runner.run()
