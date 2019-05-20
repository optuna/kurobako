#! /usr/bin/env python
#
# $ pip install git+https://github.com/sigopt/evalset.git
import argparse
from evalset import test_funcs
import json
import numpy as np
from pkg_resources import get_distribution

parser = argparse.ArgumentParser()
parser.add_argument('name')
parser.add_argument('--dim', type=int)
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

params_domain = []
for low, high in test_function.bounds:
    params_domain.append({"continuous": {'name': 'p{}'.format(len(params_domain)),
                                         'range': {"low": low, "high": high}}})

message = {
    'type': 'PROBLEM_SPEC_CAST',
    'name': 'sigopt/evalset/{}'.format(args.name),
    'version': get_distribution('evalset').version,
    'params-domain': params_domain,
    'values-domain': [{"min": float(test_function.fmin), "max": float(test_function.fmax)}],
    'evaluation-expense': 1,
    'capabilities': ['CONCURRENT']
}
print(json.dumps(message))

class Evaluator(object):
    def handle_eval(self, req):
        params = [p['continuous'] for p in req['params']]
        budget = req['budget']
        budget['consumption'] += 1

        value = test_function.do_evaluate(np.asarray(params))
        print(json.dumps({'type': 'EVALUATE_OK_REPLY', 'values': [value], 'budget': budget}))

evaluator = Evaluator()
while True:
    req = json.loads(input())
    if req['type'] == 'CREATE_EVALUATOR_CAST':
        pass
    elif req['type'] == 'DROP_EVALUATOR_CAST':
        pass
    elif req['type'] == 'EVALUATE_CALL':
        evaluator.handle_eval(req)
    else:
        raise ValueError('Unknown message: {}'.format(req))
