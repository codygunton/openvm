# OpenVM STARK Specification

This document specifies the STARK proving system used by
[OpenVM](https://github.com/openvm-org/openvm), derived from the
[Python executable specification](https://github.com/openvm-org/openvm/tree/main/executable-spec).

The Python code is the normative source of truth. This markdown specification
is a narrative overlay: every formula and algorithm step links to its
implementing Python function via `{src}` references.

## Structure

The specification is organized in two parts:

**Part I: STARK Protocol** — the generic FRI-STARK proving system over the
BabyBear field ($p = 2^{31} - 2^{27} + 1$) with quartic extension
$\mathbb{F}_{p^4}$. This part is independent of any specific program or
AIR definition.

**Part II: Programs** — concrete instantiations demonstrating the protocol:
a single-AIR Fibonacci example and a multi-AIR RV32IM example with LogUp
bus interactions.

## Reading Guide

- Start with {ref}`sec:notation` for the mathematical vocabulary.
- Read {ref}`sec:building-blocks` for the cryptographic primitives.
- Follow {ref}`sec:commitment-phase` and {ref}`sec:verification` for the
  protocol narrative.
- Use {ref}`sec:full-protocol` as a self-contained reference (the most
  important page).
- Consult {ref}`sec:glossary` to map math symbols to Python names.

```{toctree}
:maxdepth: 3
:hidden:

part-stark/index
part-programs/index
```
