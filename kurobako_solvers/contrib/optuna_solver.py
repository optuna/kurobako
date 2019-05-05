#! /usr/bin/env python
import json
import optuna

optuna.logging.set_verbosity(optuna.logging.WARNING)

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
                v = trial.suggest_uniform(data['name'], data['range']['low'], data['range']['high'])
                return {'continuous': v}
            elif data['distribution'] == 'log-uniform':
                v = trial.suggest_uniform(data['name'], data['range']['low'], data['range']['high'])
                return {'continuous': v}
            else:
                raise ValueError
        elif kind == 'discrete':
            v = trial.suggest_int(data['name'], data['range']['low'], data['range']['high'])
            return {'discrete': v}
        elif kind == 'categorical':
            category = trial.suggest_categorical(data['name'], data['choices'])
            v = data['choices'].index(category)
            return {'categorical': v}
        elif kind == 'conditional':
            raise NotImplementedError
        else:
            raise ValueError("Unknown parameter domain: {}".format(p))

    def __call__(self, trial):

        params = []
        for p in self.problem['params-domain']:
            params.append(self._suggest(p, trial))

        ask_res = {
            'type': 'ASK_REPLY',
            'id': self.next_id,
            'params': params,
            'budget': {'amount': 1, 'consumption': 0},
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
                    trial.report(value, step)
                    if trial.should_prune():
                        print(json.dumps({'type': 'TELL_REPLY'}))

                        message = json.loads(input())  # 'ASK_CALL'
                        self.next_id = message['id_hint']

                        raise optuna.structs.TrialPruned()

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


study = optuna.create_study()
study.optimize(Objective(), catch=(KurobakoError,))
