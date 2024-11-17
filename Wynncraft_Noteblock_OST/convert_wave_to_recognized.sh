#!/bin/bash
#N=16
#(
#for f in nbs/*
#do
#((i=i%N)); ((i++==0)) && wait
../target/debug/mp3-to-nbs --input-file wave/026_Yearning_for_the_Days_of_Glory_Ancient_Nemract.ogg --output-file recognized/026_Yearning_for_the_Days_of_Glory_Ancient_Nemract.nbs --sounds-folder ../SoundsQuiet --tps 6.75
#done
#)
# https://unix.stackexchange.com/questions/103920/parallelize-a-bash-for-loop
