#!/bin/env python3
# code from gpt
import sys
import re

# Pattern and replacement as defined earlier
pattern = r'(?<=\n)([A-Za-z]+)(\n)(\d+(\.\d+)?)'
replacement = r'\1,\3'

# Read input from stdin until EOF
input_data = sys.stdin.read()

# Apply regex transformation
result = re.sub(pattern, replacement, input_data)

# Output the transformed data
sys.stdout.write(result)
