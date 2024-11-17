#!/bin/bash
#N=16
#(
#for f in nbs/*
#do
#((i=i%N)); ((i++==0)) && wait
#../target/debug/mp3-to-nbs --input-file wave/026_Yearning_for_the_Days_of_Glory_Ancient_Nemract.ogg --output-file recognized/026_Yearning_for_the_Days_of_Glory_Ancient_Nemract.nbs --sounds-folder ../SoundsQuiet --tps 6.75
#../target/debug/mp3-to-nbs --input-file wave/001_Luxury_of_the_Cease-Fire_Ragni.ogg --output-file recognized/001_Luxury_of_the_Cease-Fire_Ragni.nbs --sounds-folder ../SoundsQuiet --tps 10.0
../target/debug/mp3-to-nbs --input-file wave/007_Saddle_Up_Ternaves.ogg --output-file recognized/007_Saddle_Up_Ternaves.nbs --sounds-folder ../SoundsQuiet --tps 6.75
#done
#)
# https://unix.stackexchange.com/questions/103920/parallelize-a-bash-for-loop
