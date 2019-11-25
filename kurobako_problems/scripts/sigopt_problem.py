#! /usr/bin/env python
#
# $ pip install git+https://github.com/sigopt/evalset.git
import argparse
from evalset import test_funcs
from kurobako import problem
import numpy as np
from pkg_resources import get_distribution

parser = argparse.ArgumentParser()
parser.add_argument('name')
parser.add_argument('--dim', type=int)
parser.add_argument('--res', type=float)
args = parser.parse_args()

if args.res is None:
    test_function_class = getattr(test_funcs, args.name)
    test_function = test_function_class(args.dim)
else:
    test_function_class = getattr(test_funcs, args.name)
    test_function_base = test_function_class(args.dim)
    test_function = test_funcs.Discretizer(test_function_base, args.res)


class SigoptProblemFactory(problem.ProblemFactory):
    def specification(self):
        if args.res is None:
            name = 'sigopt/evalset/{}(dim={})'.format(args.name, args.dim)
        else:
            name = 'sigopt/evalset/{}(dim={}, res={})'.format(
                args.name, args.dim, args.res)

        attrs = {
            'version': get_distribution('evalset').version,
            'github': 'https://github.com/sigopt/evalset',
            'paper':
            'A Strategy for Ranking Optimizers using Multiple Criteria',
        }

        params = []
        for low, high in test_function.bounds:
            param_name = 'p{}'.format(len(params))
            if args.res is None:
                param_range = problem.ContinuousRange(low, high)
                param = problem.Var(param_name, param_range)
            else:
                high = (high - low) // args.res
                param_range = problem.DiscreteRange(0, high)
                param = problem.Var(param_name, param_range)
            params.append(param)

        return problem.ProblemSpec(name=name,
                                   attrs=attrs,
                                   params=params,
                                   values=[problem.Var('Objective Value')])

    def create_problem(self, seed):
        return SigoptProblem()


class SigoptProblem(problem.Problem):
    def create_evaluator(self, params):
        return SigoptEvaluator(params)


class SigoptEvaluator(problem.Evaluator):
    def __init__(self, params):
        if args.res is None:
            self._params = params
        else:
            self._params = []
            for (low, _), p in zip(test_function.bounds, params):
                self._params.append(low + p * args.res)

        self._current_step = 0

    def current_step(self):
        return self._current_step

    def evaluate(self, next_step):
        self._current_step = 1
        value = test_function.do_evaluate(np.asarray(self._params))
        return [value]


if __name__ == '__main__':
    runner = problem.ProblemRunner(SigoptProblemFactory())
    runner.run()
