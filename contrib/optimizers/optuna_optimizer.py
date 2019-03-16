#! /usr/bin/env python
import json
import optuna

optuna.logging.set_verbosity(optuna.logging.WARNING)

param_space = json.loads(input())

def objective(trial):
    params = []
    for i, dist in enumerate(param_space):
        low = dist['uniform']['low']
        high = dist['uniform']['high']
        param = trial.suggest_uniform('p{}'.format(i), low, high)
        params.append(param)
    print(json.dumps(params))

    value = json.loads(input())["value"]
    return value

study = optuna.create_study()
study.optimize(objective)
