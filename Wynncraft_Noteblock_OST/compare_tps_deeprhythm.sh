#!/bin/bash
for f in wave/*
do
name=$(basename $f .ogg)
./predict_tps.py "$f" 2>/dev/null
./get_tps.py "nbs/$name.nbs"
done
