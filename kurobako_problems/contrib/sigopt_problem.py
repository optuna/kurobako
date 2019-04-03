#! /usr/bin/env python
#
# $ pip install git+https://github.com/sigopt/evalset.git
import argparse
from evalset import test_funcs
import json
import numpy as np

parser = argparse.ArgumentParser()
parser.add_argument('name')
parser.add_argument('dim', type=int)
parser.add_argument('--int', type=int, nargs='*')
parser.add_argument('--res', type=float)

args = parser.parse_args()

if args.res is None:
    test_function_class = getattr(test_funcs, args.name)
    test_function = test_function_class(args.dim)
else:
    test_function_class = getattr(test_funcs, args.name)
    test_function_base = test_function_class(args.dim)
    test_function = test_funcs.Discretizer(test_function_base, args.res)

info = {
    "problem_space": [{"uniform": {"low": low, "high": high}} for low, high in test_function.bounds],
    "cost": 1,
    "value_range": {"min": float(test_function.fmin), "max": float(test_function.fmax)},
}
print(json.dumps(info))

class Evaluator(object):
    def __init__(self):
        self.id_to_params = {}
        self.id_to_value = {}

    def handle_start_eval(self, req):
        assert req['eval_id'] not in self.id_to_params

        self.id_to_params[req['eval_id']] = req['params']
        print(json.dumps({'ok': True}))

    def handle_finish_eval(self, req):
        del self.id_to_params[req['eval_id']]
        del self.id_to_value[req['eval_id']]

    def handle_eval(self, req):
        if req['eval_id'] in self.id_to_value:
            y = self.id_to_value[req['eval_id']]
            print(json.dumps({'value': y, 'cost': 0}))
        else:
            params = self.id_to_params[req['eval_id']]
            y = test_function.do_evaluate(np.asarray(params))

            self.id_to_value[req['eval_id']] = y
            print(json.dumps({'value': y, 'cost': 1}))

evaluator = Evaluator()
while True:
    req = json.loads(input())
    if req['kind'] == 'start_eval':
        evaluator.handle_start_eval(req)
    elif req['kind'] == 'finish_eval':
        evaluator.handle_finish_eval(req)
    elif req['kind'] == 'eval':
        evaluator.handle_eval(req)
    else:
        raise ValueError('Unknown message: {}'.format(req))
