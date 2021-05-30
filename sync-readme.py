#!/bin/env python

import os
import itertools

BASE_DIR = os.path.dirname(__file__)

CRATES = [
    '.',
    'postgres',
    'redis',
    'lapin',
    'sqlite',
]

if __name__ == '__main__':
    for crate_name in CRATES:
        readme_filename = os.path.join(BASE_DIR, crate_name, 'README.md')
        src_filename = os.path.join(BASE_DIR, crate_name, 'src', 'lib.rs')
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
