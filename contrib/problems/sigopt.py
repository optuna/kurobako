#! /usr/bin/env python
#
# $ pip install git+http://github.com/sigopt/evalset.git
import argparse
from evalset import test_funcs
import json
import numpy as np

parser = argparse.ArgumentParser()
parser.add_argument('name')
parser.add_argument('dim', type=int)
parser.add_argument('--int', type=int, nargs='*')
parser.add_argument('--res', type=int)

args = parser.parse_args()

if args.res is None:
    test_function_class = getattr(test_funcs, args.name)
    test_function = test_function_class(args.dim)
else:
    test_function_class = getattr(test_funcs, args.name)
    test_function_base = test_function_class(args.dim)
    test_function = test_funcs.Discretizer(test_function_base, args.res)

print(json.dumps(test_function.bounds))

while True:
    xs = json.loads(input())
    y = test_function.do_evaluate(np.asarray(xs))
    print(y)
