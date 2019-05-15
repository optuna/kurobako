#! /usr/bin/env python
import argparse
import json
import lightgbm as lgb
import os
from pkg_resources import get_distribution

parser = argparse.ArgumentParser()
parser.add_argument('training_data_path')
parser.add_argument('validation_data_path')
parser.add_argument('--num-boost-round', type=int, default=100)
parser.add_argument('--metric', choices=['auc'], default='auc')
parser.add_argument('--min-iterations', type=int, default=10)
args = parser.parse_args()

dtrain = lgb.Dataset(args.training_data_path, free_raw_data=False, silent=True)
dtest = lgb.Dataset(args.validation_data_path, reference=dtrain, free_raw_data=False, silent=True)

##
##
##
def uniform(name, low, high):
    return {'continuous': {'name': name, 'range': {'low': low, 'high': high}}}

def discrete(name, low, high):
    return {'discrete': {'name': name, 'range': {'low': low, 'high': high}}}

params_domain = [
    discrete('max_bin', 4, 512),
    discrete('num_leaves', 4, 512),
    uniform('learning_rate', 0.0001, 1.0),
    discrete('min_data_in_leaf', 0, 100),
    uniform('min_sum_hessian_in_leaf', 0, 100),
    {'categorical': {'name': 'boosting', 'choices': ['gbdt', 'dart']}},
#    {'categorical': {'name': 'boosting', 'choices': ['gbdt', 'rf', 'dart']}},
#    {'categorical': {'name': 'boosting', 'choices': ['gbdt', 'rf', 'dart', 'goss']}},
    uniform('bagging_fraction', 0.0, 1.0), # TOOD: lightgbm.basic.LightGBMError: Cannot use bagging in GOSS
    discrete('bagging_freq', 0, 20),
    uniform('feature_fraction', 0.0, 1.0),
    discrete('max_depth', 1, 1000),
    uniform('lambda_l1', 0.0, 10.0),
    uniform('lambda_l2', 0.0, 10.0),
    uniform('min_gain_to_split', 0.0, 1.0),
]

message = {
    'type': 'PROBLEM_SPEC_CAST',
    'name': 'lightgbm/{}'.format(os.path.basename(args.training_data_path)),
    'version': get_distribution('lightgbm').version,
    'params-domain': params_domain,
    'values-domain': [{"min": 0.0, "max": 1.0}],
    'evaluation-expense': args.num_boost_round,
}
print(json.dumps(message))

##
##
##
class Evaluator(object):
    def __init__(self, raw_params):
        params = {
            'objective': 'binary',
            'verbosity': -1,
            'metric': args.metric,
        }
        for k, v in zip(params_domain, raw_params):
            if 'continuous' in k:
                params[k['continuous']['name']] = v['continuous']
            elif 'discrete' in k:
                params[k['discrete']['name']] = v['discrete']
            elif 'categorical' in k:
                params[k['categorical']['name']] = k['categorical']['choices'][v['categorical']]
            else:
                raise ValueError()

        self.gbm = lgb.Booster(params, dtrain, silent=True)
        self.gbm.add_valid(dtest, 'valid')

    def handle_eval(self, budget):
        num_boost_round = max(args.min_iterations, budget['amount'] - budget['consumption'])

        for _ in range(num_boost_round):
            self.gbm.update() # TODO: check return value

        _, _, value, maximize = self.gbm.eval_valid()[0]

        if maximize:
            value = 1.0 - value

        assert self.gbm.current_iteration() == num_boost_round + budget['consumption']
        budget['consumption'] = self.gbm.current_iteration()

        print(json.dumps({'type': 'EVALUATE_OK_REPLY', 'values': [value], 'budget': budget}))

evaluators = {}
while True:
    req = json.loads(input())
    if req['type'] == 'CREATE_EVALUATOR_CAST':
        assert req['id'] not in evaluators
        evaluators[req['id']] = None
    elif req['type'] == 'DROP_EVALUATOR_CAST':
        del evaluators[req['id']]
    elif req['type'] == 'EVALUATE_CALL':
        if evaluators[req['id']] is None:
            evaluators[req['id']] = Evaluator(req['params'])
        evaluators[req['id']].handle_eval(req['budget'])
    else:
        raise ValueError('Unknown message: {}'.format(req))
