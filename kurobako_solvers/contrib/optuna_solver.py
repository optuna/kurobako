#! /usr/bin/env python
import argparse
import json
import numpy as np
import optuna

parser = argparse.ArgumentParser()
parser.add_argument('--sampler', choices=['tpe', 'random'], default='tpe')
parser.add_argument('--tpe-startup-trials', type=int, default=10)
parser.add_argument('--tpe-ei-candidates', type=int, default=24)
parser.add_argument('--tpe-prior-weight', type=float, default=1.0)
parser.add_argument('--tpe-gamma-factor', type=float, default=0.25)
parser.add_argument('--pruner',
                    choices=['median', 'asha', 'none'],
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


class KurobakoError(Exception):
    pass


CAPABILITIES = ['CATEGORICAL', 'CONDITIONAL', 'DISCRETE', 'LOG_UNIFORM']
SPEC = {
    'type': 'SOLVER_SPEC_CAST',
    'name': 'optuna',
    'version': optuna.__version__,
    'capabilities': CAPABILITIES
}
print(json.dumps(SPEC))


class Objective(object):
    def __init__(self):
        self.problem = json.loads(input())
        self.next_id = json.loads(input())['id_hint']

    def _suggest(self, p, trial):
        kind, data = list(p.items())[0]
        if kind == 'continuous':
            if data['distribution'] == 'uniform':
                v = trial.suggest_uniform(data['name'], data['range']['low'],
                                          data['range']['high'])
                return {'continuous': v}
            elif data['distribution'] == 'log-uniform':
                v = trial.suggest_uniform(data['name'], data['range']['low'],
                                          data['range']['high'])
                return {'continuous': v}
            else:
                raise ValueError
        elif kind == 'discrete':
            v = trial.suggest_int(data['name'], data['range']['low'],
                                  data['range']['high'])
            return {'discrete': v}
        elif kind == 'categorical':
            category = trial.suggest_categorical(data['name'], data['choices'])
            v = data['choices'].index(category)
            return {'categorical': v}
        elif kind == 'conditional':
            assert 'member' in data['condition']
            dependent_param_name = data['condition']['member']['name']
            dependent_param_choices = data['condition']['member']['choices']
            if dependent_param_name in trial.params:
                if trial.params[dependent_param_name] in dependent_param_choices:
                    return {'conditional': self._suggest(data['param'], trial)}
            return {'conditional': None}
        else:
            raise ValueError("Unknown parameter domain: {}".format(p))

    def __call__(self, trial):
        params = []
        for p in self.problem['params-domain']:
            params.append(self._suggest(p, trial))

        if args.pruner == 'none':
            amount = self.problem['evaluation-expense']
        else:
            amount = 1
        ask_res = {
            'type': 'ASK_REPLY',
            'id': self.next_id,
            'params': params,
            'budget': {
                'amount': amount,
                'consumption': 0
            },
        }
        print(json.dumps(ask_res))

        while True:
            message = json.loads(input())
            if message['type'] == 'ASK_CALL':
                self.next_id = message['id_hint']
                raise KurobakoError("Pruned by problem")
            elif message['type'] == 'TELL_CALL':
                budget = message['budget']
                value = message['values'][0]
                step = budget['consumption']
                if step == self.problem['evaluation-expense']:
                    print(json.dumps({'type': 'TELL_REPLY'}))

                    message = json.loads(input())  # 'ASK_CALL'
                    self.next_id = message['id_hint']

                    return value
                else:
                    assert step < self.problem['evaluation-expense']
                    assert args.pruner != 'none'

                    trial.report(value, step)
                    if trial.should_prune(step):
                        print(json.dumps({'type': 'TELL_REPLY'}))

                        message = json.loads(input())  # 'ASK_CALL'
                        self.next_id = message['id_hint']

                        raise optuna.structs.TrialPruned(
                            'step={}, value={}'.format(step, value))

                    print(json.dumps({'type': 'TELL_REPLY'}))
                    message = json.loads(input())  # 'ASK_CALL'
                    budget['amount'] += 1
                    ask_res = {
                        'type': 'ASK_REPLY',
                        'id': self.next_id,
                        'params': params,
                        'budget': budget,
                    }
                    print(json.dumps(ask_res))
            else:
                raise ValueError("Unknown message: {}".format(message))


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
    pruner = None

study = optuna.create_study(sampler=sampler, pruner=pruner)
study.optimize(Objective(), catch=(KurobakoError, ))
