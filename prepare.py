#!/usr/bin/env python3
"""Copy only the immutable task inputs into a new candidate directory."""

import argparse
import hashlib
from pathlib import Path
import sys


INPUTS = ("SPEC.md", "prompt.txt")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path)
    parser.add_argument("--check", action="store_true",
                        help="compare existing candidate inputs without writing")
    args = parser.parse_args()
    source = Path(__file__).resolve().parent
    destination = args.destination.absolute()

    try:
        payloads = {name: (source / name).read_bytes() for name in INPUTS}
        if args.check:
            if destination.is_symlink() or not destination.is_dir():
                raise ValueError("candidate must be an existing, non-symlink directory")
            failures = []
            for name, expected in payloads.items():
                candidate = destination / name
                if candidate.is_symlink() or not candidate.is_file():
                    failures.append(f"{name}: missing, not a file, or symlink")
                elif candidate.read_bytes() != expected:
                    failures.append(f"{name}: changed")
            if failures:
                raise ValueError("; ".join(failures))
            print("PASS: both input files are byte-identical to the assignment.")
        else:
            # Atomic mkdir refuses existing files, directories and symlinks.
            # Parent must exist; existing content is never replaced.
            destination.mkdir()
            for name, payload in payloads.items():
                with (destination / name).open("xb") as output:
                    output.write(payload)
            print(f"Created candidate: {destination}")
            print("Inputs only: SPEC.md, prompt.txt (no source or repository history).")
        for name, payload in payloads.items():
            print(f"sha256 {hashlib.sha256(payload).hexdigest()}  {name}")
        return 0
    except (OSError, ValueError) as error:
        print(f"ERROR: {error}", file=sys.stderr)
        if not args.check:
            print("Nothing was overwritten. Inspect a partial new directory before retrying.",
                  file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
