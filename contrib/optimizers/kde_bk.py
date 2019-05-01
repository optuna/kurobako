#! /usr/bin/env python
import json
import numpy as np
from scipy import stats

import sys

class Optimizer(object):
    def __init__(self, low, high, index):
        self.index = index

        self.low = low
        self.high = high
        self.xs = []
        self.ys = []
        self.rng = np.random.RandomState()

    def ask(self):
        if len(self.xs) < 10:
            x = self.rng.uniform(self.low, self.high)
            self.xs.append(x)
            if self.index == 0:
                sys.stderr.write('# X={}'.format(x))
            return x

        kde = stats.gaussian_kde(np.asarray([self.xs, self.ys]))
        x = self.xs[-1]
        y = min(self.ys)
        curr_pdf, = kde.pdf([x, y])

        low = self.low
        high = self.high
        height = self.rng.uniform(0.0, curr_pdf)
        if self.index == 0:
            sys.stderr.write('\n# HEIGHT={}\n'.format(height))
        count = 0
        while True:
            count += 1
            next_x = self.rng.uniform(low, high)
            next_pdf, = kde.pdf([next_x, y])
            if next_pdf >= curr_pdf:
                break
            if next_x < x:
                low = next_x
            else:
                high = next_x

        if self.index == 0:
            sys.stderr.write('# X={}, PDF={}, C={}'.format(next_x, next_pdf, count))
        self.xs.append(next_x)
        return next_x

    def tell(self, y):
        self.ys.append(y)
        if self.index == 0:
            sys.stderr.write(', Y={}, MIN={}\n'.format(y, min(self.ys)))

param_space = json.loads(input())
opts = []
for i, dist in enumerate(param_space):
    low = dist['uniform']['low']
    high = dist['uniform']['high']
    opts.append(Optimizer(low, high, i))

while True:
    params = []
    for opt in opts:
        params.append(opt.ask())
    print(json.dumps(params))

    value = json.loads(input()).get("value")
    for opt in opts:
        opt.tell(value)
