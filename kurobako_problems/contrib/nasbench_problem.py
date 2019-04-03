#! /usr/bin/env python
#
# $ pip install git+https://github.com/google-research/nasbench.git
import argparse
import json
from nasbench import api

INPUT = 'input'
OUTPUT = 'output'
CONV1X1 = 'conv1x1-bn-relu'
CONV3X3 = 'conv3x3-bn-relu'
MAXPOOL3X3 = 'maxpool3x3'

EPOCHS_LIST = [4, 12, 36, 108]
INNER_OPS = [CONV1X1, CONV3X3, MAXPOOL3X3]

parser = argparse.ArgumentParser()
parser.add_argument('--dataset-path', type=str, default="nasbench_full.tfrecord")

args = parser.parse_args()

###
### load
###
nasbench = api.NASBench(args.dataset_path)

###
### problem info
###

# TODO: support categorical
problem_space = []
for i in range(5):
    problem_space.append({"uniform": {"low": 0, "high": len(INNER_OPS)}})

for i in range(6 + 5 + 4 + 3 + 2 + 1):
    problem_space.append({"uniform": {"low": 0, "high": 1}})

problem_info = {
    "problem_space": problem_space,
    "cost": EPOCHS_LIST[-1],
    "value_range": {"min": 0.0, "max": 1.0},
}
print(json.dumps(problem_info))


def params_to_model_spec(params):
    ops = [INPUT]
    for i in range(5):
        if params[i] < 1.0:
            ops.append(CONV1X1)
        elif params[i] < 2.0:
            ops.append(CONV3X3)
        elif params[i] < 3.0:
            ops.append(MAXPOOL3X3)
        else:
            raise ValueError()
    ops.append(OUTPUT)

    p = [round(v) for v in params]
    matrix = [
        [0, p[5], p[6],  p[7],  p[8],  p[9],  p[10]],
        [0, 0,    p[11], p[12], p[13], p[14], p[15]],
        [0, 0,        0, p[16], p[17], p[18], p[19]],
        [0, 0,        0,     0, p[20], p[21], p[22]],
        [0, 0,        0,     0,     0, p[23], p[24]],
        [0, 0,        0,     0,     0,     0, p[25]],
        [0, 0,        0,     0,     0,     0,     0],
    ]

    model_spec = api.ModelSpec(matrix=matrix, ops=ops)
    return model_spec

class Evaluator(object):
    def __init__(self, nasbench):
        self.runnings = {}
        self.nasbench = nasbench

    def handle_start_eval(self, id, req):
        assert id not in self.runnings

        model_spec = params_to_model_spec(req['params'])
        if self.nasbench.is_valid(model_spec):
            self.runnings[id] = model_spec
            print(json.dumps({'ok': True}))
        else:
            print(json.dumps({'ok': False}))

    def handle_finish_eval(self, id, req):
        del self.runnings[id]

    def handle_eval(self, id, req):
        assert id in self.runnings

        model_spec = self.runnings[id]
        budget = req['budget']
        for e in EPOCHS_LIST:
            epochs = e
            if epochs >= budget:
                break

        # TODO(?): handle stop_halfway option
        result = self.nasbench.query(model_spec, epochs=epochs)
        value = 1.0 - result['validation_accuracy']
        print(json.dumps({'value': value, 'cost': epochs}))


evaluator = Evaluator(nasbench)
while True:
    req = json.loads(input())
    if req['kind'] == 'start_eval':
        evaluator.handle_start_eval(req['eval_id'], req)
    elif req['kind'] == 'finish_eval':
        evaluator.handle_finish_eval(req['eval_id'], req)
    elif req['kind'] == 'eval':
        evaluator.handle_eval(req['eval_id'], req)
    else:
        raise ValueError('Unknown message: {}'.format(req))
