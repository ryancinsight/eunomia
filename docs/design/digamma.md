# Real digamma provider contract

This additive `FloatElement::digamma` supplies d/dx ln|Gamma(x)| for real
non-poles. It keeps a single native generic algorithm in the scalar provider;
Leto owns traversal, Coeus owns differentiation, and Hephaestus owns devices.
A default method avoids a new required method or a canonical-trait replacement.
No breaking design sign-off is needed for this additive surface.

Use recurrence psi(x)=psi(x+1)-1/x, reflection
psi(x)=psi(1-x)-pi*cot(pi*x), and the Bernoulli asymptotic expansion at x>=16.
Reduce reflection arguments modulo the nearest integer before multiplication by
pi to avoid loss at large negative arguments; integer and half-integer cases
are tested explicitly. Eight Bernoulli terms leave a remainder bounded by
|B18|/(18*16^18) < 3e-21 on the positive ray (DLMF 5.11(ii)). Arithmetic,
logarithms and tangents use the native generic scalar; f64 and F64 override the
default boundary and never narrow. Reduced storage formats explicitly follow
the existing special-function software contract: exact widening to f32,
evaluation there, one result conversion. That is a storage-boundary policy,
not a claim of native reduced-precision transcendental instructions.

These classifications describe the native IEEE evaluation. Reduced results
then follow their storage format conversion, including finite-only saturation
and format-specific NaN encoding. NaN propagates. Positive infinity returns
positive infinity. Negative infinity
and negative integers return NaN. Positive zero returns negative infinity,
negative zero positive infinity (the one-sided Laurent term -1/x). Positive and
negative finite tiny arguments obey the same pole asymptote, including overflow.

Acceptance uses psi(1)=-EulerGamma, psi(1/2)=-EulerGamma-2ln2,
recurrence/reflection, and mpmath fixtures evaluated at exact binary inputs with
100 decimal digits. Near zeros, use absolute plus relative error: the absolute
term accounts for cancellation in recurrence/reflection; a relative-only test
is ill-conditioned. The acceptance bounds come from explicit absolute-error propagation through
recurrence magnitudes, argument increments, Bernoulli/FMA arithmetic, log
and reflection conditioning, with gradual underflow terms. See
[the complete derivation](digamma-error-budget.md) and its fixture generator.
The quantified tangent premise is verified at the actual fixture arguments;
no universal libm error theorem or operation-count cancellation bound is claimed.
Reduced-format tests use exact provider-boundary
comparison and representable neighbors at poles and the positive zero.

Sources: https://dlmf.nist.gov/5.5 (E2, E4),
https://dlmf.nist.gov/5.11 (E2 and error bounds),
https://docs.scipy.org/doc/scipy/reference/generated/scipy.special.digamma.html,
https://docs.pytorch.org/docs/2.14/special.html#torch.special.digamma,
https://docs.jax.dev/en/latest/_autosummary/jax.scipy.special.digamma.html.
SciPy separately improves the first two zeros; the explicit absolute error
bound here remains the acceptance contract. Special-value differences between
frameworks are recorded rather than used to erase domain tests.
