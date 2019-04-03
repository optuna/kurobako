#! /usr/bin/env python
#
# $ pip install git+https://github.com/google-research/nasbench.git
import argparse
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

# model_spec = api.ModelSpec(
#         # Adjacency matrix of the module
#         matrix=[[0, 1, 1, 1, 0, 1, 0],    # input layer
#                 [0, 0, 0, 0, 0, 0, 1],    # 1x1 conv
#                 [0, 0, 0, 0, 0, 0, 1],    # 3x3 conv
#                 [0, 0, 0, 0, 1, 0, 0],    # 5x5 conv (replaced by two 3x3's)
#                 [0, 0, 0, 0, 0, 0, 1],    # 5x5 conv (replaced by two 3x3's)
#                 [0, 0, 0, 0, 0, 0, 1],    # 3x3 max-pool
#                 [0, 0, 0, 0, 0, 0, 0]],   # output layer
#         # Operations at the vertices of the module, matches order of matrix
#         ops=[INPUT, CONV1X1, CONV3X3, CONV3X3, CONV3X3, MAXPOOL3X3, OUTPUT])

# print(nasbench.query(model_spec))
# print("---------------------------")
# print(nasbench.query(model_spec, epochs=4))
# print("---------------------------")
# print(nasbench.query(model_spec, epochs=4, stop_halfway=True))
