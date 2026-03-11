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
    EF4Coeffs,
    Fe,
    GENERATOR,
    ff4,
    ff4_coeffs,
    ff4_from_base,
    get_omega,
    inv_mod,
)

p = BABYBEAR_PRIME


# --- Extension Field Helpers ---
# These operate on EF4Coeffs (list[int]) representation, delegating to
# the galois FF4 type for arithmetic.


def ef4_from_base(x: Fe) -> EF4Coeffs:
    """Embed base field element into extension field as (x, 0, 0, 0).

    Reference:
        p3-field ExtensionField::from_base
    """
    return [x % p, 0, 0, 0]


def ef4_mul(a: EF4Coeffs, b: EF4Coeffs) -> EF4Coeffs:
    """Multiply two extension field elements.

    Reference:
        p3-field BinomialExtensionField::mul
    """
    return ff4_coeffs(ff4(a) * ff4(b))


def ef4_mul_base(a: EF4Coeffs, b: Fe) -> EF4Coeffs:
    """Multiply extension field element by a base field element.

    Reference:
        p3-field ExtensionField::mul_base
    """
    return ff4_coeffs(ff4(a) * ff4_from_base(b))


def ef4_add(a: EF4Coeffs, b: EF4Coeffs) -> EF4Coeffs:
    """Add two extension field elements.

    Reference:
        p3-field BinomialExtensionField::add
    """
    return ff4_coeffs(ff4(a) + ff4(b))


def ef4_sub(a: EF4Coeffs, b: EF4Coeffs) -> EF4Coeffs:
    """Subtract two extension field elements.

    Reference:
        p3-field BinomialExtensionField::sub
    """
    return ff4_coeffs(ff4(a) - ff4(b))


def ef4_neg(a: EF4Coeffs) -> EF4Coeffs:
    """Negate an extension field element: -a.

    Reference:
        p3-field BinomialExtensionField::neg
    """
    return [(BABYBEAR_PRIME - c) % BABYBEAR_PRIME for c in a]


def ef4_inv(x: EF4Coeffs) -> EF4Coeffs:
    """Multiplicative inverse in extension field.

    Reference:
        p3-field BinomialExtensionField::inverse
    """
    return ff4_coeffs(ff4(x) ** (-1))


def ef4_div(a: EF4Coeffs, b: EF4Coeffs) -> EF4Coeffs:
    """Division in extension field: a / b.

    Reference:
        p3-field BinomialExtensionField::div
    """
    return ff4_coeffs(ff4(a) * ff4(b) ** (-1))


def ef4_pow(x: EF4Coeffs, n: int) -> EF4Coeffs:
    """Exponentiation in extension field by non-negative integer.

    Uses square-and-multiply.

    Reference:
        p3-field FieldAlgebra::exp_u64
    """
    if n == 0:
        return [1, 0, 0, 0]
    result = ff4(x) ** n
    return ff4_coeffs(result)


def ef4_exp_power_of_2(x: EF4Coeffs, log_power: int) -> EF4Coeffs:
    """Compute x^(2^log_power) by repeated squaring.

    Reference:
        p3-field Field::exp_power_of_2
    """
    result = ff4(x)
    for _ in range(log_power):
        result = result * result
    return ff4_coeffs(result)


# --- Data Structures ---


@dataclass
class DomainSelectors:
    """Lagrange selectors evaluated at a point outside the domain.

    These are NOT normalized Lagrange basis polynomials; they are
    unnormalized selectors as defined in Plonky3.

    Reference:
        p3-commit-0.4.1/src/domain.rs (LagrangeSelectors struct, lines 20-30)
    """
    is_first_row: EF4Coeffs    # Z_H(x) / (unshifted_x - 1)
    is_last_row: EF4Coeffs     # Z_H(x) / (unshifted_x - gen^{-1})
    is_transition: EF4Coeffs   # unshifted_x - gen^{-1}
    inv_zeroifier: EF4Coeffs   # 1 / Z_H(x)


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

    def next_point(self, point: EF4Coeffs) -> EF4Coeffs:
        """Map the i-th element to the (i+1)-th: multiply by the generator.

        For a coset gH with generator h, next_point(x) = x * h.

        Args:
            point: An extension field element (evaluation point).

        Returns:
            point * gen (extension field multiplication by base element).

        Reference:
            p3-commit domain.rs PolynomialSpace::next_point (line 144-146)
        """
        return ef4_mul_base(point, self.gen())

    def vanishing_poly_at_point(self, point: EF4Coeffs) -> EF4Coeffs:
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
        # unshifted = point * shift^{-1}
        unshifted = ef4_mul_base(point, self.shift_inverse())
        # unshifted^(2^log_n) - 1
        powered = ef4_exp_power_of_2(unshifted, self.log_n)
        return ef4_sub(powered, [1, 0, 0, 0])

    def selectors_at_point(self, point: EF4Coeffs) -> DomainSelectors:
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
        # unshifted_point = point * shift^{-1}
        unshifted_point = ef4_mul_base(point, self.shift_inverse())

        # z_h = unshifted_point^(2^log_n) - 1
        z_h = ef4_sub(ef4_exp_power_of_2(unshifted_point, self.log_n), [1, 0, 0, 0])

        # gen_inv = generator^{-1} in base field
        gen_inv = inv_mod(self.gen())

        # is_first_row = z_h / (unshifted_point - 1)
        is_first_row = ef4_div(z_h, ef4_sub(unshifted_point, [1, 0, 0, 0]))

        # is_last_row = z_h / (unshifted_point - gen_inv)
        is_last_row = ef4_div(
            z_h,
            ef4_sub(unshifted_point, ef4_from_base(gen_inv)),
        )

        # is_transition = unshifted_point - gen_inv
        is_transition = ef4_sub(unshifted_point, ef4_from_base(gen_inv))

        # inv_vanishing = 1 / z_h
        inv_zeroifier = ef4_inv(z_h)

        return DomainSelectors(
            is_first_row=is_first_row,
            is_last_row=is_last_row,
            is_transition=is_transition,
            inv_zeroifier=inv_zeroifier,
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


def natural_domain_for_log_degree(log_degree: int) -> TwoAdicMultiplicativeCoset:
    """Return the canonical domain (subgroup) for the given log degree.

    Convenience wrapper: natural_domain_for_degree(2^log_degree).

    Args:
        log_degree: log2 of the domain size.

    Returns:
        TwoAdicMultiplicativeCoset with shift=1 and log_n = log_degree.

    Reference:
        openvm-native-recursion commit.rs (PcsVariable::natural_domain_for_log_degree)
    """
    return TwoAdicMultiplicativeCoset(log_n=log_degree, shift=1)


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


def split_domains(
    quotient_domain: TwoAdicMultiplicativeCoset,
    num_chunks: int,
) -> list[TwoAdicMultiplicativeCoset]:
    """Split a quotient domain into sub-cosets.

    Convenience wrapper around TwoAdicMultiplicativeCoset.split_domains.

    Args:
        quotient_domain: The domain to split.
        num_chunks: Number of chunks (must be a power of 2).

    Returns:
        List of sub-domains.

    Reference:
        p3-commit domain.rs PolynomialSpace::split_domains (lines 174-186)
    """
    return quotient_domain.split_domains(num_chunks)


def recompute_quotient(
    quotient_chunks: list[list[EF4Coeffs]],
    qc_domains: list[TwoAdicMultiplicativeCoset],
    zeta: EF4Coeffs,
) -> EF4Coeffs:
    """Recompute the full quotient polynomial from chunks at the evaluation point.

    For each chunk domain D_i, compute:
        zp_i = prod_{j != i} Z_{D_j}(zeta) / Z_{D_j}(first_point(D_i))

    Then the full quotient at zeta is:
        Q(zeta) = sum_i zp_i * sum_e ch[i][e] * monomial(e)

    The "monomial" reconstruction converts the 4 base-field components of
    each quotient chunk back into a single extension field element using
    the canonical basis {1, x, x^2, x^3} of GF(p^4).

    Args:
        quotient_chunks: For each chunk, a list of 4 base-field evaluation values.
        qc_domains: The quotient chunk sub-domains from split_domains.
        zeta: The evaluation point (extension field element).

    Returns:
        The reconstructed quotient polynomial value at zeta.

    Reference:
        stark-backend verifier/constraints.rs lines 38-52
        openvm-native-recursion stark/mod.rs StarkVerifier::recompute_quotient
    """
    num_chunks = len(qc_domains)
    assert len(quotient_chunks) == num_chunks

    # Compute zps[i] = prod_{j!=i} Z_{D_j}(zeta) / Z_{D_j}(D_i.first_point())
    zps = []
    for i in range(num_chunks):
        prod = [1, 0, 0, 0]
        for j in range(num_chunks):
            if j != i:
                zp_at_zeta = qc_domains[j].vanishing_poly_at_point(zeta)
                first_pt = ef4_from_base(qc_domains[i].first_point())
                zp_at_first = qc_domains[j].vanishing_poly_at_point(first_pt)
                # prod *= zp_at_zeta / zp_at_first
                factor = ef4_div(zp_at_zeta, zp_at_first)
                prod = ef4_mul(prod, factor)
        zps.append(prod)

    # Reconstruct: Q(zeta) = sum_i zps[i] * (ch[i][0] + ch[i][1]*x + ch[i][2]*x^2 + ch[i][3]*x^3)
    # The quotient_chunks[i] has 4 elements which are the coefficients in the
    # extension field basis. So quotient_chunks[i] IS an EF4Coeffs.
    result = [0, 0, 0, 0]
    for i in range(num_chunks):
        chunk_ef4 = quotient_chunks[i]  # Already [c0, c1, c2, c3]
        term = ef4_mul(zps[i], chunk_ef4)
        result = ef4_add(result, term)

    return result
