# Part II: Programs

This part demonstrates the STARK protocol from Part I on concrete programs.

## Overview

| Program | AIRs | Trace | LogUp | Test Vector |
|---------|------|-------|-------|-------------|
| Fibonacci | 1 (single-AIR) | 2 columns, 256 rows | No | `fibonacci_stark.json` |
| RV32IM Fibonacci | Multiple (multi-AIR) | Various heights | Yes | `rv32im_fibonacci.json` |

The Fibonacci program is the simplest possible instantiation: a single AIR
with 5 constraints and no bus interactions. It serves as a pedagogical
starting point.

The RV32IM Fibonacci program demonstrates multi-AIR proving with matrices of
different heights, LogUp bus interactions, and per-height alpha tracking in the PCS.

```{toctree}
:maxdepth: 2

fibonacci
multi-air
```
