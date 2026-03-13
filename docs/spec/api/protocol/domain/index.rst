protocol.domain
===============

.. py:module:: protocol.domain

.. autoapi-nested-parse::

   Two-adic multiplicative coset domains and Lagrange selectors.

   Provides domain computation for the STARK verifier: vanishing polynomials,
   Lagrange selectors, domain splitting, and disjoint domain creation.

   Reference:
       p3-field-0.4.1/src/coset.rs (TwoAdicMultiplicativeCoset struct)
       p3-commit-0.4.1/src/domain.rs (PolynomialSpace impl for TwoAdicMultiplicativeCoset)
       p3-fri-0.4.1/src/two_adic_pcs.rs (natural_domain_for_degree)



Attributes
----------

.. autoapisummary::

   protocol.domain.p


Classes
-------

.. autoapisummary::

   protocol.domain.DomainSelectors
   protocol.domain.TwoAdicMultiplicativeCoset


Functions
---------

.. autoapisummary::

   protocol.domain.natural_domain_for_degree
   protocol.domain.create_disjoint_domain


Module Contents
---------------

.. py:data:: p
   :value: 2013265921


.. py:class:: DomainSelectors

   Lagrange selectors evaluated at a point outside the domain.

   These are NOT normalized Lagrange basis polynomials; they are
   unnormalized selectors as defined in Plonky3.

   Reference:
       p3-commit-0.4.1/src/domain.rs (LagrangeSelectors struct, lines 20-30)


   .. py:attribute:: is_first_row
      :type:  primitives.field.FF4


   .. py:attribute:: is_last_row
      :type:  primitives.field.FF4


   .. py:attribute:: is_transition
      :type:  primitives.field.FF4


   .. py:attribute:: inv_zeroifier
      :type:  primitives.field.FF4


.. py:class:: TwoAdicMultiplicativeCoset

   Coset of a two-adic multiplicative subgroup of BabyBear.

   Represents the set {shift * omega^i : i = 0, ..., 2^log_n - 1}
   where omega = get_omega(log_n) is the canonical 2^log_n-th root of unity.

   Reference:
       p3-field-0.4.1/src/coset.rs (TwoAdicMultiplicativeCoset struct)


   .. py:attribute:: log_n
      :type:  int


   .. py:attribute:: shift
      :type:  primitives.field.Fe


   .. py:method:: size() -> int

      Return the number of elements in the domain.

      Reference:
          p3-field coset.rs TwoAdicMultiplicativeCoset::size



   .. py:method:: gen() -> primitives.field.Fe

      Return the subgroup generator (primitive 2^log_n-th root of unity).

      Reference:
          p3-field coset.rs TwoAdicMultiplicativeCoset::subgroup_generator



   .. py:method:: shift_inverse() -> primitives.field.Fe

      Return the multiplicative inverse of the coset shift.

      Reference:
          p3-field coset.rs TwoAdicMultiplicativeCoset::shift_inverse



   .. py:method:: first_point() -> primitives.field.Fe

      Return the first element of the coset (the shift itself).

      Reference:
          p3-commit domain.rs PolynomialSpace::first_point (line 139-141)



   .. py:method:: next_point(point) -> primitives.field.FF4

      Map the i-th element to the (i+1)-th: multiply by the generator.

      For a coset gH with generator h, next_point(x) = x * h.

      Args:
          point: An extension field element (evaluation point).

      Returns:
          point * gen (extension field multiplication by base element).

      Reference:
          p3-commit domain.rs PolynomialSpace::next_point (line 144-146)



   .. py:method:: vanishing_poly_at_point(point) -> primitives.field.FF4

      Evaluate the vanishing polynomial Z_{gH}(X) at the given point.

      Z_{gH}(X) = (g^{-1} * X)^|H| - 1

      where g = shift, |H| = 2^log_n.

      Args:
          point: Extension field element to evaluate at.

      Returns:
          Z_{gH}(point) as an extension field element.

      Reference:
          p3-commit domain.rs PolynomialSpace::vanishing_poly_at_point (lines 226-228)



   .. py:method:: selectors_at_point(point) -> DomainSelectors

      Compute Lagrange selectors at an evaluation point.

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



   .. py:method:: split_domains(num_chunks: int) -> list[TwoAdicMultiplicativeCoset]

      Split this coset into num_chunks smaller cosets of equal size.

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



.. py:function:: natural_domain_for_degree(degree: int) -> TwoAdicMultiplicativeCoset

   Return the canonical domain (subgroup) for the given degree.

   The natural domain for degree n is the unique two-adic subgroup of order n,
   i.e., the coset with shift = 1.

   Args:
       degree: Must be a power of 2.

   Returns:
       TwoAdicMultiplicativeCoset with shift=1 and log_n = log2(degree).

   Reference:
       p3-fri-0.4.1/src/two_adic_pcs.rs (Pcs::natural_domain_for_degree, line 188-190)


.. py:function:: create_disjoint_domain(domain: TwoAdicMultiplicativeCoset, min_size: int) -> TwoAdicMultiplicativeCoset

   Create a domain disjoint from the given one with at least min_size elements.

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


