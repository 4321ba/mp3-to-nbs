#!/bin/bash
for f in recognized/*
do
./diff.py "nbs/`basename $f`" "$f" csvoutput
done
