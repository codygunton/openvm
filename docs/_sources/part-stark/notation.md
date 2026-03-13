(sec:notation)=
# Primitives

## Fields

The base field is the BabyBear prime field

$$
\F = \mathbb{Z}/p\mathbb{Z}, \qquad p = 2^{31} - 2^{27} + 1 = 2013265921.
$$

({src}`primitives.field.BABYBEAR_PRIME` constant, {src}`primitives.field.FF` type)

The multiplicative group $\F^*$ has order $p - 1 = 2^{27} \cdot 15$, giving a
two-adicity of $27$ ({src}`primitives.field.TWO_ADICITY`).
The canonical multiplicative generator is $g = 31$
({src}`primitives.field.GENERATOR`).

The quartic extension field is

$$
\Fext = \F[x] / (x^4 - 11),
$$

where $x^4 = 11$ in $\Fext$.
An element of $\Fext$ is written as $c_0 + c_1 x + c_2 x^2 + c_3 x^3$ with
$c_i \in \F$, stored as `[c0, c1, c2, c3]` in the `FF4` class
({src}`primitives.field.FF4`).

The Python spec implements base field arithmetic using the
[galois](https://mhostetter.github.io/galois/) library with JIT-compiled numpy
operations. The extension field uses a custom `FF4` class backed by int64 numpy
arrays for performance.

### Montgomery Form

BabyBear serde serialization uses Montgomery form: $\hat{v} = v \cdot 2^{32} \bmod p$.
The Python spec converts at parse time via `from_monty()` so all internal
computation uses canonical values in $[0, p)$
({src}`primitives.field.MONTY_R`, {src}`primitives.field.MONTY_RINV`).

## Domains

Let $N = 2^n$ be the *trace size* (number of rows in the execution trace).
The *trace domain* is a two-adic multiplicative coset

$$
H = \bigl\{s \cdot \omega^i : i = 0, \ldots, N-1\bigr\},
$$

where $\omega \in \F$ is a primitive $N$-th root of unity
({src}`primitives.field.get_omega`)
and $s \in \F$ is the coset shift (typically $s = 1$ for the trace domain)
({src}`protocol.domain.TwoAdicMultiplicativeCoset`).

The *extended evaluation domain* (LDE domain) is a disjoint coset of size
$N_{\text{ext}} = N \cdot \beta$, where $\beta = 2^{\text{log\_blowup}}$ is the
blowup factor from the FRI parameters.

The *quotient domain* is a disjoint coset of size $N \cdot d_Q$, where $d_Q$ is
the quotient degree (maximum constraint degree minus 1).

### Domain Selectors

At a point $\zeta$ outside the domain $H$, the following selectors are defined
({src}`protocol.domain.DomainSelectors`):

| Selector | Formula | Purpose |
|----------|---------|---------|
| `is_first_row` | $Z_H(\zeta) / (\zeta/s - 1)$ | Selects first row |
| `is_last_row` | $Z_H(\zeta) / (\zeta/s - \omega^{-1})$ | Selects last row |
| `is_transition` | $\zeta/s - \omega^{-1}$ | Nonzero on non-last rows |
| `inv_zeroifier` | $1 / Z_H(\zeta)$ | Inverse vanishing polynomial |

where $Z_H(X) = (X/s)^N - 1$ is the vanishing polynomial of the domain $H$
({src}`protocol.domain.get_selectors_at_point`).

## Polynomials

The following polynomial types appear in the protocol:

| Name | Symbol | Degree | Description |
|------|--------|--------|-------------|
| Witness polynomials | $f_1, \ldots, f_m$ | $< N$ | Main trace columns (Stage 1) |
| After-challenge polynomials | $h_1, \ldots, h_k$ | $< N$ | LogUp auxiliary columns (Stage 2) |
| Quotient chunks | $Q_0, \ldots, Q_{d_Q-1}$ | $< N$ | Pieces of the quotient polynomial |
| FRI polynomial | $F$ | $< N \cdot \beta$ | Batched polynomial for FRI input |

All polynomials are committed as evaluations on LDE domains via Merkle trees
(see {ref}`sec:building-blocks`).

## Digest

A *digest* is an 8-element vector over $\F$, produced by the Poseidon2 hash.
The digest width is $8$ ({src}`primitives.field.DIGEST_WIDTH`).

## Notation Summary

| Symbol | Meaning | Python name |
|--------|---------|-------------|
| $p$ | BabyBear prime | `BABYBEAR_PRIME` |
| $\F$ | Base field $\text{GF}(p)$ | `FF` |
| $\Fext$ | Extension field $\text{GF}(p^4)$ | `FF4` |
| $g$ | Multiplicative generator | `GENERATOR` |
| $\omega$ | Primitive $N$-th root of unity | `get_omega(log_n)` |
| $N$ | Trace size ($2^n$) | `domain.size()` |
| $H$ | Trace domain | `TwoAdicMultiplicativeCoset` |
| $s$ | Coset shift | `domain.shift` |
| $Z_H$ | Vanishing polynomial | `domain.zp_at_point(x)` |
| $\alpha$ | Constraint folding challenge | `alpha` |
| $\zeta$ | OOD evaluation point | `zeta` |
| $\tau$ | FRI fold challenges | `betas[i]` |

For the complete glossary mapping symbols to Python names, see {ref}`sec:glossary`.
