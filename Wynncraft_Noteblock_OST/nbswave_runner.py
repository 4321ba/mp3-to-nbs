#!/bin/env python3
# required: pip3 install nbswave==0.3.0
from nbswave import render_audio
from sys import argv
print(f"Converting {argv[1]} to {argv[2]} with sounds {argv[3]}")
render_audio(argv[1], argv[2], default_sound_path=argv[3])
