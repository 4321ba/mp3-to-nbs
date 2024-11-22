#!/bin/env python3
from sys import argv
import pynbs

song = pynbs.read(argv[1])
tps = song.header.tempo
print(tps)
