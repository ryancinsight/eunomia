# Digamma fixture error budget

`scripts/digamma_error_budget.py` computes an a priori enclosure for each
finite fixture, before executing the implementation. It uses 100 decimal-digit
arithmetic and serializes bounds outward. `digamma_error_bounds.rs` contains
absolute A and relative r such that the acceptance is |computed-reference| <=
A + r|reference|. The earlier operation-count argument for 128u(1+|psi|) is
withdrawn: cancellation does not permit that inference. The retained 128u
checks are additional tight regressions, not a proved global error contract.

Let u=2^-p (p=24 or 53), eta be the least subnormal, and each (a,e) enclose a
real center a with absolute input error e. Rounding any arithmetic result t
with propagated error E adds u(|t|+E)+eta/2. Thus addition propagates ea+eb;
multiplication propagates |a|eb+|b|ea+ea*eb; reciprocal propagates
 e/(|a|(|a|-e)), provided |a|>e. FMA propagates product and addend errors but
rounds only once. Coefficient error includes the f64 rational-division rounding
and target-format conversion, including their product term. These rules include underflow and
are applied to every recurrence increment, reciprocal, sum, inverse-square,
Bernoulli Horner stage and subtraction. The actual representable loop count is
used; it is not inferred from an unrounded threshold.

Reflection starts from fl(1-x); the initial subtraction error is included.
The nearest-integer remainder r=x-round(x) is exact: subtraction from zero is
exact for |x|<1/2, and Sterbenz applies for the remaining finite nonintegers.
For theta=pi*r, the angle uncertainty is at most (2u+u^2)|theta|+eta/2, accounting
for the pi constant and multiplication. The mean-value theorem bounds the
cotangent contribution by pi*Etheta / min(sin(theta_lo)^2,sin(theta_hi)^2).
The interval does not contain zero for the finite tested outputs. Exact
half-integers use zero cotangent and do not evaluate tan. The numerator and division use relative u bounds. If t_min bounds the tangent
magnitude below, the two-ulp premise gives a relative denominator error at most
d=(4u+2eta/t_min)/(1-u). The factor 1/(1-u) converts the ULP bound around the rounded reference
to the exact tangent magnitude. The direct quotient enclosure is
 Q=(1+u)^2/(1-d)-1, retaining all cross terms. Its absolute contribution is
Q times the maximum endpoint magnitude of pi*cot(theta), plus eta/2 for final
division rounding. No cancellation is replaced by a relative bound on psi:
the cotangent magnitude remains in A.

Log input uncertainty contributes e_y/(y-e_y). `libm` 0.2.16's
`src/math/log.rs` explicitly documents error <1 ulp, bounded by
2u*(|log(y)|+e_y/(y-e_y))+eta/2. Its tan documentation says "nearly rounded", without a
quantified global theorem. The budget therefore uses an explicit **two-ulp
premise for tan at fixture arguments**, separately verified against 100-digit
hi+lo references in `digamma_primitive_fixtures.rs`. Single-precision primitive
calls are verified separately. This is a checked finite-domain premise, not a
proof of a universal libm tan bound. Source: libm 0.2.16 `math/tan.rs`,
`math/k_tan.rs`, `math/tanf.rs` and `math/k_tanf.rs`.

DLMF 5.11(ii) bounds the asymptotic truncation by the first omitted term,
|B18|/(18*(y-e_y)^18). The final reflected subtraction contributes u times
its result plus the propagated errors. Independent reference rounding adds
another u; the two final eta/2 terms are retained in A; hence r=2u/(1-u), and A is the accumulated absolute enclosure divided
by (1-u). Serialization uses nextafter toward positive infinity. The reference
hi+lo primitive checks require measured error plus measurement uncertainty to
fit the primitive ULP premise; they account separately for rounding of lo and of their
error-measurement subtraction. For the ordinary anchor/zero fixtures this
propagated A is tighter than the old fixed envelope; no observed error sets a
bound and no existing check is weakened.

Limit: this is a reproducible finite-fixture acceptance analysis conditional on
the checked primitive premises. It does not assert correctly rounded digamma,
universal relative accuracy at zeros, or hardware backend accuracy. Overflowing
pole asymptotes and NaNs use separate exact classification tests.
