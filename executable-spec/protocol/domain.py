"""Two-adic multiplicative coset domains and Lagrange selectors.

Provides domain computation for the STARK verifier: vanishing polynomials,
Lagrange selectors, domain splitting, and disjoint domain creation.

Reference:
    p3-field-0.4.1/src/coset.rs (TwoAdicMultiplicativeCoset struct)
    p3-commit-0.4.1/src/domain.rs (PolynomialSpace impl for TwoAdicMultiplicativeCoset)
    p3-fri-0.4.1/src/two_adic_pcs.rs (natural_domain_for_degree)
"""

from dataclasses import dataclass

from primitives.field import (
    BABYBEAR_PRIME,
    FF4,
    Fe,
    GENERATOR,
    get_omega,
    inv_mod,
)

p = BABYBEAR_PRIME


# --- Data Structures ---


@dataclass
class DomainSelectors:
    """Lagrange selectors evaluated at a point outside the domain.

    These are NOT normalized Lagrange basis polynomials; they are
    unnormalized selectors as defined in Plonky3.

    Reference:
        p3-commit-0.4.1/src/domain.rs (LagrangeSelectors struct, lines 20-30)
    """
    is_first_row: FF4    # Z_H(x) / (unshifted_x - 1)
    is_last_row: FF4     # Z_H(x) / (unshifted_x - gen^{-1})
    is_transition: FF4   # unshifted_x - gen^{-1}
    inv_zeroifier: FF4   # 1 / Z_H(x)


@dataclass
class TwoAdicMultiplicativeCoset:
    """Coset of a two-adic multiplicative subgroup of BabyBear.

    Represents the set {shift * omega^i : i = 0, ..., 2^log_n - 1}
    where omega = get_omega(log_n) is the canonical 2^log_n-th root of unity.

    Reference:
        p3-field-0.4.1/src/coset.rs (TwoAdicMultiplicativeCoset struct)
    """
    log_n: int    # log2 of domain size
    shift: Fe     # coset shift element (in [0, p))

    def size(self) -> int:
        """Return the number of elements in the domain.

        Reference:
            p3-field coset.rs TwoAdicMultiplicativeCoset::size
        """
        return 1 << self.log_n

    def gen(self) -> Fe:
        """Return the subgroup generator (primitive 2^log_n-th root of unity).

        Reference:
            p3-field coset.rs TwoAdicMultiplicativeCoset::subgroup_generator
        """
        return get_omega(self.log_n)

    def shift_inverse(self) -> Fe:
        """Return the multiplicative inverse of the coset shift.

        Reference:
            p3-field coset.rs TwoAdicMultiplicativeCoset::shift_inverse
        """
        return inv_mod(self.shift)

    def first_point(self) -> Fe:
        """Return the first element of the coset (the shift itself).

        Reference:
            p3-commit domain.rs PolynomialSpace::first_point (line 139-141)
        """
        return self.shift

    def next_point(self, point) -> FF4:
        """Map the i-th element to the (i+1)-th: multiply by the generator.

        For a coset gH with generator h, next_point(x) = x * h.

        Args:
            point: An extension field element (evaluation point).

        Returns:
            point * gen (extension field multiplication by base element).

        Reference:
            p3-commit domain.rs PolynomialSpace::next_point (line 144-146)
        """
        return FF4(point).mul_base(self.gen())

    def vanishing_poly_at_point(self, point) -> FF4:
        """Evaluate the vanishing polynomial Z_{gH}(X) at the given point.

        Z_{gH}(X) = (g^{-1} * X)^|H| - 1

        where g = shift, |H| = 2^log_n.

        Args:
            point: Extension field element to evaluate at.

        Returns:
            Z_{gH}(point) as an extension field element.

        Reference:
            p3-commit domain.rs PolynomialSpace::vanishing_poly_at_point (lines 226-228)
        """
        unshifted = FF4(point).mul_base(self.shift_inverse())
        return unshifted ** (2 ** self.log_n) - FF4.one()

    # <doc-anchor id="get-selectors">
    def selectors_at_point(self, point) -> DomainSelectors:
        """Compute Lagrange selectors at an evaluation point.

        Given the vanishing polynomial Z_{gH}(X) = (g^{-1}X)^|H| - 1, compute:
        - is_first_row:  Z_{gH}(X) / (g^{-1}X - 1)
        - is_last_row:   Z_{gH}(X) / (g^{-1}X - h^{-1})
        - is_transition: g^{-1}X - h^{-1}
        - inv_vanishing: 1 / Z_{gH}(X)

        where g = shift and h = subgroup generator.

        These selectors are NOT normalized (i.e., they are not divided by
        n * shift^{n-1} to make them equal to 1 at the target point).

        Args:
            point: Extension field element (typically zeta, the evaluation point).

        Returns:
            DomainSelectors with all four selectors computed.

        Reference:
            p3-commit domain.rs PolynomialSpace::selectors_at_point (lines 237-245)
        """
        unshifted_point = FF4(point).mul_base(self.shift_inverse())
        gen_inv = inv_mod(self.gen())
        z_h = unshifted_point ** (2 ** self.log_n) - FF4.one()

        return DomainSelectors(
            is_first_row=z_h / (unshifted_point - FF4.one()),
            is_last_row=z_h / (unshifted_point - FF4(gen_inv)),
            is_transition=unshifted_point - FF4(gen_inv),
            inv_zeroifier=z_h ** -1,
        )

    def split_domains(self, num_chunks: int) -> list["TwoAdicMultiplicativeCoset"]:
        """Split this coset into num_chunks smaller cosets of equal size.

        Given coset gH with generator h and num_chunks = 2^k, decompose as:
            gH = gK union ghK union gh^2K union ... union gh^{num_chunks-1}K
        where K = H^{num_chunks} (the unique subgroup of order |H|/num_chunks).

        Each sub-coset has:
            - shift = self.shift * gen^i  (for i = 0, ..., num_chunks - 1)
            - log_n = self.log_n - log2(num_chunks)
            - generator = gen^{num_chunks}

        Args:
            num_chunks: Number of chunks (must be a power of 2 and divide size).

        Returns:
            List of TwoAdicMultiplicativeCoset sub-domains.

        Reference:
            p3-commit domain.rs PolynomialSpace::split_domains (lines 174-186)
        """
        log_chunks = num_chunks.bit_length() - 1
        assert 1 << log_chunks == num_chunks, "num_chunks must be a power of 2"
        assert log_chunks <= self.log_n, "num_chunks must not exceed domain size"

        gen = self.gen()
        new_log_n = self.log_n - log_chunks

        domains = []
        domain_power = 1  # gen^0 = 1
        for _ in range(num_chunks):
            new_shift = (self.shift * domain_power) % p
            domains.append(TwoAdicMultiplicativeCoset(
                log_n=new_log_n,
                shift=new_shift,
            ))
            domain_power = (domain_power * gen) % p

        return domains


# --- Module-level Helper Functions ---


# <doc-anchor id="natural-domain">
def natural_domain_for_degree(degree: int) -> TwoAdicMultiplicativeCoset:
    """Return the canonical domain (subgroup) for the given degree.

    The natural domain for degree n is the unique two-adic subgroup of order n,
    i.e., the coset with shift = 1.

    Args:
        degree: Must be a power of 2.

    Returns:
        TwoAdicMultiplicativeCoset with shift=1 and log_n = log2(degree).

    Reference:
        p3-fri-0.4.1/src/two_adic_pcs.rs (Pcs::natural_domain_for_degree, line 188-190)
    """
    log_n = degree.bit_length() - 1
    assert 1 << log_n == degree, f"degree must be a power of 2, got {degree}"
    return TwoAdicMultiplicativeCoset(log_n=log_n, shift=1)


# <doc-anchor id="create-disjoint-domain">
def create_disjoint_domain(
    domain: TwoAdicMultiplicativeCoset,
    min_size: int,
) -> TwoAdicMultiplicativeCoset:
    """Create a domain disjoint from the given one with at least min_size elements.

    Given coset gH, returns the coset g*f*K where f is the multiplicative
    generator of BabyBear (GENERATOR = 31) and K is the unique two-adic
    subgroup of order >= min_size. Because f is a generator of the full
    multiplicative group, g*f is not in g*K for any two-adic subgroup K,
    guaranteeing disjointness.

    Args:
        domain: The original domain to be disjoint from.
        min_size: Minimum size of the new domain (will be rounded up to power of 2).

    Returns:
        A new TwoAdicMultiplicativeCoset disjoint from domain.

    Reference:
        p3-commit domain.rs PolynomialSpace::create_disjoint_domain (lines 155-167)
    """
    from math import ceil, log2

    log_n = ceil(log2(min_size)) if min_size > 1 else 0
    new_shift = (domain.shift * GENERATOR) % p
    return TwoAdicMultiplicativeCoset(log_n=log_n, shift=new_shift)


