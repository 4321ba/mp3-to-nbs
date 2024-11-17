#!/bin/env python3
from sys import argv
import pynbs
from copy import deepcopy
print(f"Comparing original {argv[1]} to recognized {argv[2]}")

INSTR_CNT = 5
PITCH_CNT = 25
original_song = pynbs.read(argv[1])
recognized_song = pynbs.read(argv[2])
end_tick = max(original_song.notes[-1].tick, recognized_song.notes[-1].tick) + 1
original_song_data = [[[0 for _ in range(PITCH_CNT)] for _ in range(INSTR_CNT)] for _ in range(end_tick)]
recognized_song_data = deepcopy(original_song_data)

def fill_list_from_song(volume_list, song):
    for tick, chord in song:
        for note in chord:
            if note.key < 33 or note.key > 57:
                continue
            volume_list[tick][note.instrument][note.key - 33] += note.velocity / 100

fill_list_from_song(original_song_data, original_song)
fill_list_from_song(recognized_song_data, recognized_song)

# https://stackoverflow.com/questions/952914/how-do-i-make-a-flat-list-out-of-a-list-of-lists
def flatten(xsss):
    return [x for xss in xsss for xs in xss for x in xs]

original_flattened = flatten(original_song_data)
recognized_flattened = flatten(recognized_song_data)

acc_squared = 0
for o in original_flattened:
    acc_squared += o * o
print("Sum of squared volume errors compared to silence:", acc_squared)

acc_diff_squared = 0
for (o, r) in zip(original_flattened, recognized_flattened):
    acc_diff_squared += (o-r) * (o-r)
print("Sum of squared volume errors compared to recognized:", acc_diff_squared)

cnt = 0
for o in original_flattened:
    if o > 0:
        cnt += 1
print("Count of notes in the original:", cnt)

cnt = 0
for (o, r) in zip(original_flattened, recognized_flattened):
    if o > 0 and r > 0:
        cnt += 1
print("Count of notes correctly recognized:", cnt)

cnt = 0
for (o, r) in zip(original_flattened, recognized_flattened):
    if o > 0 and r == 0:
        cnt += 1
print("Count of notes not recognized:", cnt)

cnt = 0
for (o, r) in zip(original_flattened, recognized_flattened):
    if o == 0 and r > 0:
        cnt += 1
print("Count of nonexisting notes recognized:", cnt)

cnt = 0
for (o, r) in zip(original_flattened, recognized_flattened):
    if o == 0 and r == 0:
        cnt += 1
print("Count of silences correctly recognized:", cnt)
