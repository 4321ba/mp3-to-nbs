#!/bin/env python3
from sys import argv
# pip3 install deeprhythm
from deeprhythm import DeepRhythmPredictor

model = DeepRhythmPredictor()

tempo = model.predict(argv[1])
tps = tempo * 4 / 60
tps_exact = int(tps * 4 + 0.5) / 4
print(f"{argv[1]},{tps_exact},", end="")
