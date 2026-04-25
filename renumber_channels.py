#!/usr/bin/env python3
"""Renumber the channel-number column of an FTX-1 memory CSV.

Header row, blank lines, and lines starting with '#' pass through unchanged.
Data rows get sequential 5-digit channel numbers starting at 00001.

Usage:
    python renumber_channels.py input.csv > output.csv
"""
import sys


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit("Usage: renumber_channels.py <input.csv>")

    with open(sys.argv[1]) as f:
        n = 0
        for i, line in enumerate(f):
            if i == 0 or not line.strip() or line.lstrip().startswith("#"):
                sys.stdout.write(line)
                continue
            n += 1
            _, _, rest = line.partition(",")
            sys.stdout.write(f"{n:05d},{rest}")


if __name__ == "__main__":
    main()
