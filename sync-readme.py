#!/bin/env python

import os
import itertools

CRATES = [
    ['README.md', 'src/lib.rs'],
    ['postgres/README.md', 'postgres/src/lib.rs'],
    ['redis/README.md', 'redis/src/lib.rs'],
    ['lapin/README.md', 'lapin/src/lib.rs'],
]

if __name__ == '__main__':
    for readme_filename, src_filename in CRATES:
        with open(readme_filename) as fh:
            readme = fh.readlines()
        with open(src_filename) as fh:
            it = iter(fh.readlines())
            for line in it:
                if not line.startswith("//!"):
                    break
            code = ''.join(itertools.chain([line], it))
        with open(src_filename + '.new', 'w') as fh:
            for line in readme:
                if line.rstrip():
                    fh.write('//! ')
                else:
                    fh.write('//!')
                fh.write(line)
            fh.write(code)
        os.rename(src_filename + '.new', src_filename)
