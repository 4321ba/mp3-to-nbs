#!/bin/bash
N=16
(
for f in wave/*
do
((i=i%N)); ((i++==0)) && wait
name=$(basename $f .ogg)
../target/debug/mp3-to-nbs --input-file "$f" --output-file "recognized/$name.nbs" --sounds-folder "../SoundsQuiet" --tps `./get_tps.py "nbs/$name.nbs"` &
done
)
# https://unix.stackexchange.com/questions/103920/parallelize-a-bash-for-loop
