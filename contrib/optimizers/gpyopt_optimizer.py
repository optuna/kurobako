#! /usr/bin/env python
import json
import GPyOpt
import numpy as np

problem_space = json.loads(input())
space = []

for i, dist in enumerate(problem_space):
    low = dist['uniform']['low']
    high = dist['uniform']['high']
    space.append({
        'name': 'x_{}'.format(i),
        'type': 'continuous',
        'domain': (low, high)
    })


def objective(params):
    params = np.array(params, dtype=np.float).flatten().tolist()
    print(json.dumps(params))

    value = json.loads(input())["value"]
    return value


opt = GPyOpt.methods.BayesianOptimization(objective, space)
opt.run_optimization(10000000)
