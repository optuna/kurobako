#! /usr/bin/env python
import json
import numpy as np
from scipy import stats
import math

import sys

class Optimizer(object):
    def __init__(self, low, high, index):
        self.index = 1 #index

        self.low = low
        self.high = high
        self.xs = []
        self.ys = []
        self.rng = np.random.RandomState()

    def sample(self, kde):
        x = self.xs[-1]
        y = min(self.ys)
        curr_pdf, = kde.pdf([x, y])

        low = self.low
        high = self.high
        height = self.rng.uniform(0.0, curr_pdf)
        count = 0
        while True:
            count += 1
            next_x = self.rng.uniform(low, high)
            next_pdf, = kde.pdf([next_x, y])
            if next_pdf > height:
                return next_x, next_pdf
            if next_x < x:
                low = next_x
            else:
                high = next_x

    def ask(self):
        if len(self.xs) < 10: #or self.rng.uniform(0.0, len(self.xs)) < 1:
            x = self.rng.uniform(self.low, self.high)
            self.xs.append(x)
            if self.index == 0:
                sys.stderr.write('# X={} (rand)'.format(x))
            return x

        size = max(20, len(self.xs)//2)
        kde = stats.gaussian_kde(np.asarray([self.xs[-size:], self.ys[-size:]]), bw_method='silverman')
        # xs = np.asarray(self.xs)
        # ys = np.asarray(self.ys)
        # indices = np.argsort(ys)[:40]
        # kde = stats.gaussian_kde(np.vstack([xs[indices], ys[indices]]), bw_method='silverman')

        #kde = stats.gaussian_kde(np.asarray([self.xs, self.ys]))

        next_pdf = 0.0
        next_x = 0
        for _ in range(1): #math.ceil(math.sqrt(len(self.xs)))):
            x, pdf = self.sample(kde)
            if pdf >= next_pdf:
                next_pdf = pdf
                next_x = x

        if self.index == 0:
            sys.stderr.write('# X={}, PDF={}'.format(next_x, next_pdf))
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
