#! /usr/bin/env python
import json
import numpy as np
from scipy import stats

class Optimizer(object):
    def __init__(self, low, high):
        self.low = low
        self.high = high
        self.xs = []
        self.ys = []
        self.rng = np.random.RandomState()

    def ask(self):
        if len(self.xs) < 3:
            x = self.rng.uniform(self.low, self.high)
            self.xs.append(x)
            return x

        kde = stats.gaussian_kde(np.asarray([self.xs, self.ys]))
        x = self.xs[-1]
        y = self.ys[-1]
        curr_pdf, = kde.pdf([x, y])

        low = self.low
        high = self.high
        height = self.rng.uniform(0.0, curr_pdf)
        while True:
            next_x = self.rng.uniform(low, high)
            next_pdf, = kde.pdf([next_x, y])
            if next_pdf >= curr_pdf:
                break
            if next_x < x:
                low = next_x
            else:
                high = next_x

        self.xs.append(next_x)
        return next_x

    def tell(self, y):
        self.ys.append(y)


param_space = json.loads(input())
opts = []
for dist in param_space:
    low = dist['uniform']['low']
    high = dist['uniform']['high']
    opts.append(Optimizer(low, high))

while True:
    params = []
    for opt in opts:
        params.append(opt.ask())
    print(json.dumps(params))

    value = json.loads(input()).get("value")
    for opt in opts:
        opt.tell(value)
