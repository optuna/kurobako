import argparse
import json
import re
import sys

import optuna

parser = argparse.ArgumentParser()
parser.add_argument("storage", type=str)
parser.add_argument("target_study_name", type=str)
args = parser.parse_args()

study_names = [
    study.study_name
    for study in optuna.get_all_study_summaries(args.storage)
    if re.match(args.target_study_name, study.study_name)
]

print("Target studies: {}".format(study_names), file=sys.stderr)

trials = []
for study_name in study_names:
    study = optuna.load_study(study_name=study_name, storage=args.storage)
    for trial in study.trials:
        values = [
            v if v is None or d == optuna.study.StudyDirection.MINIMIZE else -v
            for v, d in zip(trial.values, study.directions)
        ]
        distributions = {
            name: json.loads(optuna.distributions.distribution_to_json(d))
            for name, d in trial.distributions.items()
        }
        trials.append(
            {
                "params": trial.params,
                "distributions": {
                    name: {d["name"]: d["attributes"]} for name, d in distributions.items()
                },
                "values": values,
            }
        )

print("Number of target trials: {}".format(len(trials)), file=sys.stderr)
print(json.dumps(trials))
