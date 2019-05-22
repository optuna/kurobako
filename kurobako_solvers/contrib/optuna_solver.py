#! /usr/bin/env python
import argparse
import kurobako.problem
import kurobako.solver
import kurobako.solvers
import numpy as np
import optuna

##
## (1) Handle command-line arguments
##
parser = argparse.ArgumentParser()
parser.add_argument('--sampler', choices=['tpe', 'random'], default='tpe')
parser.add_argument('--tpe-startup-trials', type=int, default=10)
parser.add_argument('--tpe-ei-candidates', type=int, default=24)
parser.add_argument('--tpe-prior-weight', type=float, default=1.0)
parser.add_argument('--tpe-gamma-factor', type=float, default=0.25)
parser.add_argument('--pruner',
                    choices=['median', 'asha'],
                    default='median')
parser.add_argument('--median-startup-trials', type=int, default=5)
parser.add_argument('--median-warmup-steps', type=int, default=0)
parser.add_argument('--asha-min-resource', type=int, default=1)
parser.add_argument('--asha-reduction-factor', type=int, default=4)
parser.add_argument('--loglevel',
                    choices=['debug', 'info', 'warning', 'error'])
args = parser.parse_args()

if args.loglevel == 'debug':
    optuna.logging.set_verbosity(optuna.logging.DEBUG)
elif args.loglevel == 'info':
    optuna.logging.set_verbosity(optuna.logging.INFO)
elif args.loglevel == 'warning':
    optuna.logging.set_verbosity(optuna.logging.WARNING)
elif args.loglevel == 'error':
    optuna.logging.set_verbosity(optuna.logging.ERROR)

if args.sampler == 'random':
    sampler = optuna.samplers.RandomSampler()
elif args.sampler == 'tpe':

    def gamma(x):
        return min(int(np.ceil(args.tpe_gamma_factor * np.sqrt(x))), 25)

    sampler = optuna.samplers.TPESampler(
        n_startup_trials=args.tpe_startup_trials,
        n_ei_candidates=args.tpe_ei_candidates,
        prior_weight=args.tpe_prior_weight,
        gamma=gamma,
    )
else:
    sampler = None

if args.pruner == 'median':
    pruner = optuna.pruners.MedianPruner(
        n_startup_trials=args.median_startup_trials,
        n_warmup_steps=args.median_warmup_steps)
elif args.pruner == 'asha':
    pruner = optuna.pruners.SuccessiveHalvingPruner(
        min_resource=args.asha_min_resource,
        reduction_factor=args.asha_reduction_factor)
else:
    raise ValueError("Unknown pruner: {}".format(args.pruner))


##
## (2) Send solver specification
##
print(kurobako.solvers.OptunaSolver.specification().to_message())


##
## (3) Receive problem specification
##
problem = kurobako.problem.ProblemSpec.from_message(input())


##
## (4) Solve
##
solver = kurobako.solvers.OptunaSolver(problem, sampler=sampler, pruner=pruner)
runner = kurobako.solver.SolverRunner(solver)
runner.run()
