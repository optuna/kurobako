#! /usr/bin/env python3
import argparse
import json

from kurobako import solver
from kurobako.solver.optuna import OptunaSolverFactory
import optuna
import optuna.integration
import optuna.pruners
import optuna.samplers

##
## (1) Parse command-line arguments
##
parser = argparse.ArgumentParser()
parser.add_argument("--sampler", type=str, default="TPESampler")
parser.add_argument("--sampler-kwargs", type=str, default="{}")
parser.add_argument("--pruner", type=str, default="MedianPruner")
parser.add_argument("--pruner-kwargs", type=str, default="{}")
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

    # Sampler.
    sampler_cls = getattr(
        optuna.samplers, args.sampler, getattr(optuna.integration, args.sampler, None)
    )
    if sampler_cls is None:
        raise ValueError("Unknown sampler: {}.".format(args.sampler))

    sampler_kwargs = json.loads(args.sampler_kwargs)
    try:
        sampler_kwargs["seed"] = seed
        sampler = sampler_cls(**sampler_kwargs)
    except:
        del sampler_kwargs["seed"]
        sampler = sampler_cls(**sampler_kwargs)

    # Pruner.
    pruner_cls = getattr(
        optuna.pruners, args.pruner, getattr(optuna.integration, args.pruner, None)
    )
    if pruner_cls is None:
        raise ValueError("Unknown pruner: {}.".format(args.pruner))

    pruner_kwargs = json.loads(args.pruner_kwargs)
    try:
        pruner_kwargs["seed"] = seed
        pruner = pruner_cls(**pruner_kwargs)
    except:
        del pruner_kwargs["seed"]
        pruner = pruner_cls(**pruner_kwargs)

    return optuna.create_study(sampler=sampler, pruner=pruner, direction=args.direction)


##
## (3) Solve
##
if __name__ == "__main__":
    factory = OptunaSolverFactory(create_study, use_discrete_uniform=args.use_discrete_uniform)
    runner = solver.SolverRunner(factory)
    runner.run()
