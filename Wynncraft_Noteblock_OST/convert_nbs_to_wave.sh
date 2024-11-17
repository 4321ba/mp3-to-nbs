#!/bin/bash
# Source: https://github.com/4321ba/Wynncraft_Noteblock_OST
N=16
(
for f in nbs/*
do
((i=i%N)); ((i++==0)) && wait
./nbswave_runner.py "$f" "wave/`basename $f .nbs`.ogg" "../SoundsQuiet" &
done
)
# https://unix.stackexchange.com/questions/103920/parallelize-a-bash-for-loop
