#! /usr/bin/env python
from deepobs.tensorflow import runners
import sys
import tensorflow as tf

OPTIMIZERS = {
    # https://www.tensorflow.org/api_docs/python/tf/train/AdadeltaOptimizer
    'adadelta':
    (tf.train.AdadeltaOptimizer,
     [
         {'name': 'rho', 'type': float, 'default': 0.95},
         {'name': 'epsilon', 'type': float, 'default': 1e-08}
     ]),

    # https://www.tensorflow.org/api_docs/python/tf/train/AdagradOptimizer
    'adagrad':
    (tf.train.AdagradOptimizer,
     [
         {'name': 'initial_accumulator_value', 'type': float, 'default': 0.1}
     ]),

    # https://www.tensorflow.org/api_docs/python/tf/train/AdamOptimizer
    'adam':
    (tf.train.AdamOptimizer,
     [
         {'name': 'beta1', 'type': float, 'default': 0.9},
         {'name': 'beta2', 'type': float, 'default': 0.999},
         {'name': 'epsilon', 'type': float, 'default': 1e-08}
     ]),

    # https://www.tensorflow.org/api_docs/python/tf/train/FtrlOptimizer
    'ftrl':
    (tf.train.FtrlOptimizer,
     [
         {'name': 'learning_rate_power', 'type': float, 'default': -0.5},
         {'name': 'initial_accumulator_value', 'type': float, 'default': 0.1},
         {'name': 'l1_regularization_strength', 'type': float, 'default': 0.0},
         {'name': 'l2_regularization_strength', 'type': float, 'default': 0.0},
         {'name': 'l2_shrinkage_regularization_strength', 'type': float, 'default': 0.0}
     ]),

    # https://www.tensorflow.org/api_docs/python/tf/train/GradientDescentOptimizer
    'gradient-descent':
    (tf.train.GradientDescentOptimizer,
     [
     ]),

    # https://www.tensorflow.org/api_docs/python/tf/train/MomentumOptimizer
    'momentum':
    (tf.train.MomentumOptimizer,
     [
         {'name': 'momentum', 'type': float},
         {'name': 'use_nesterov', 'type': bool, 'default': False}
     ]),

    # https://www.tensorflow.org/api_docs/python/tf/train/ProximalAdagradOptimizer
    'proximal-adagrad':
    (tf.train.ProximalAdagradOptimizer,
     [
         {'name': 'initial_accumulator_value', 'type': float, 'default': 0.1},
         {'name': 'l1_regularization_strength', 'type': float, 'default': 0.0},
         {'name': 'l2_regularization_strength', 'type': float, 'default': 0.0}
     ]),

    # https://www.tensorflow.org/api_docs/python/tf/train/ProximalGradientDescentOptimizer
    'proximal-gradient-descent':
    (tf.train.ProximalGradientDescentOptimizer,
     [
         {'name': 'l1_regularization_strength', 'type': float, 'default': 0.0},
         {'name': 'l2_regularization_strength', 'type': float, 'default': 0.0}
     ]),

    # https://www.tensorflow.org/api_docs/python/tf/train/RMSPropOptimizer
    'rms-prop':
    (tf.train.RMSPropOptimizer,
     [
         {'name': 'decay', 'type': float, 'default': 0.9},
         {'name': 'momentum', 'type': float, 'default': 0.0},
         {'name': 'epsilon', 'type': float, 'default': 1e-10},
         {'name': 'centered', 'type': bool, 'default': False}
     ])
}

optimizer = sys.argv[1]
del sys.argv[1]

optimizer_class, hyperparams = OPTIMIZERS[optimizer]
runner = runners.StandardRunner(optimizer_class, hyperparams)
runner.run()
