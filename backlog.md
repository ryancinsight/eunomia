# Eunomia backlog

<a id="eunomia-stable-qualification"></a>
## EUNOMIA-STABLE-QUALIFICATION — eunomia stable-release qualification [patch] — todo
- priority: verification; milestone: M-special-functions; basis: `31f90b4808c0`.
- Outcome: Close the subnormal integer-power proof across the shipped number formats. Follow the [release scope](docs/adr/0012-release-scope-1-0.md) and [milestone exits](backlog/milestones.md). Planning targets for `eunomia` are 0.13.0, then 1.0.0: `main` carries the breaking `Complex` bound of [ADR 0013](docs/adr/0013-real-complex-components.md), so a 0.12.2 patch needs a branch at the `0.12.1` tag ([EUNOMIA-REAL-COMPONENT-RELEASE](#EUNOMIA-REAL-COMPONENT-RELEASE)). Verification alone can advance a patch version on that branch.
- Acceptance and sequence: [stable-release checklist](checklist.md#stable-release-checklist).

<a id="EUNOMIA-REAL-COMPONENT-RELEASE"></a>
## EUNOMIA-REAL-COMPONENT-RELEASE — the release carrying ADR 0013 is 0.13.0 and ships with the leto `RealElement` bound [major]
- status: blocked; priority: correctness; milestone: M-special-functions; needs: none; basis: `b13d421f24b9`.
- Outcome: no 0.12.x release carries the narrowed `Complex<T>` impl; the release that does is the next breaking 0.x, 0.13.0, and leto changes its `Complex` `Scalar` impl in the same co-evolution unit.
- Evidence (`b13d421f24b9`, read; [ADR 0013](docs/adr/0013-real-complex-components.md) Decision 4 and revision note; eunomia#229): every stack member resolves eunomia from the crates.io registry at 0.12.1 (leto `Cargo.lock:419`, requirement `0.12.1`, caret),
  so a 0.12.x release cut from `main` reaches leto through `cargo update` and fails there (E0277, `T: RealElement`, through `Scalar: NumericElement`); leto cannot name `eunomia::RealElement` until a release exports it.
- Acceptance: leto `crates/leto-ops/src/domain/scalar/impls.rs:313` reads `impl<T> Scalar for Complex<T> where T: Scalar + eunomia::RealElement + core::ops::Neg<Output = T> {}` and requires the released eunomia 0.13.x; leto builds and tests against it;
  `git ls-tree -r <tag> --name-only` and `git grep -n RealElement <tag> -- crates` show `RealElement` in the 0.13.0 tag and in no 0.12.x tag.
- Blocker: release authority; re-open trigger: the user's release decision for eunomia 0.13.0.
- Next step: on that decision, release 0.13.0 under the co-evolution protocol, then edit leto line 313 as quoted and move leto's requirement.
- Scope: leto `crates/leto-ops/src/domain/scalar/impls.rs`, eunomia workspace version.

<a id="EUNOMIA-POWI-SUBNORMAL-SWEEP"></a>
## EUNOMIA-POWI-SUBNORMAL-SWEEP — the powi subnormal bound is pinned on three rows, not swept on every format [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: bf0bce90eabd.
- Outcome: `powi_subnormal.rs` holds the bound of `power.rs` on every shipped format (`F4`, `Bf4`, `F8`, `Bf8`, `F16`, `Bf16`, `F32`, `f32`, `F64`, `f64`),
  not only on the `Bf8`, `f32`, `F16` and `F8` pinned rows, and `integer_power.rs` holds each row its pin drops (`inverts_subnormal_power`) to `check_row`.
- Evidence: the rescue PR #218 (`refs/pull/218/head`) carries both: `every_format!` over `upward_sweep` (log grid, powers from `2^(emin-2)` to `2^(emin+1)`,
  `e` up to 24) and `result_sweep` (results `2^(emax-1)` to `2^(emax+1)`; skipped for `F4`, which has no subnormal), the dropped rows forwarded in `check_previous_powi`, and
  `check_row`, `Format`, `largest_exponent` made `pub(super)`. At that head every format passes; dropping the exponent carry fails the upward sweep on `F8`..`f64`.
- Acceptance: the upward sweep kills the carry-drop mutant (`Carried::multiply` without `.saturating_add(carry)`) on `F8`, `Bf8`, `F16`, `Bf16`, `F32`, `f32`, `F64`, `f64`; it fails at the
  basis on `F16`, `Bf16`, `F32`, `f32`; `F4` and `Bf4` stay held by the pins (every row has `m = 1`, so the carry is zero).
- Next step: restore the sweep and the hook from `refs/pull/218/head`; the counted diff is about 236 lines against `main` once the fix has landed.
- Scope: `crates/eunomia/tests/float_element/powi_subnormal.rs`, `crates/eunomia/tests/float_element/integer_power.rs`.

<a id="EUNOMIA-POWI-SATURATION-GUARD"></a>
## EUNOMIA-POWI-SATURATION-GUARD — the finite-only guard of powi's saturation predicate cannot change a result [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: EUNOMIA-FLOAT-EXTREMES-BITS; basis: ac61854df78a45a2144bf3aa9dc0c21858fa892f.
- Outcome: `saturates` (`crates/eunomia/src/traits/float/power.rs:218-220`) is `T::INFINITY.is_finite() && power.abs() == T::MAX_VALUE`; no test fails without its first conjunct.
  `MAX_VALUE` and `INFINITY` are one constant expression in all ten formats (EUNOMIA-FLOAT-EXTREMES-BITS); `saturates` runs after the non-finite branch (`power.rs:132`), so it is false on an infinity.
- Evidence: the dropped-conjunct mutant passes `--test float_element` (20 of 20) and the package (234 of 234), relayed from the PR #210 judge at `2ef8acd`;
  `power.rs` has changed since (139 lines added since `2ef8acd`; `saturates` has a second use at `:167`, after an infinity test on the same line), so the survivor is relayed, not confirmed at the basis. A rerun here is void, its disabled-branch control also passed.
- Risk: the conjunct is a per-monomorphization constant, so it may let LLVM drop the branch for the seven formats with an infinity.
- Acceptance: `power::powi` and `saturates` are `#[inline(always)]` and `float.rs:382` is `#[inline]`, so take the asm of a non-inlined wrapper per format with an infinity,
  conjunct dropped against the basis. (i) No wrapper gains a compare or branch: delete the conjunct, record the clean asm diffs in the closing PR, keep
  `powi_equals_the_loop_it_replaced_bit_for_bit` and `powi_negative_exponent_rounds_the_exact_power_when_upward_saturates` passing unchanged, and have the `saturates` doc comment say the
  predicate admits every clamped power and an exact power equal to `T::MAX_VALUE` (`F8` `powi(240, -1)`), which the accumulator path rounds correctly. (ii) Any wrapper gains one: keep the
  conjunct and record the mutant as equivalent, with the asm diff, in the `saturates` doc comment.
- Next step: take the per-format asm diffs once EUNOMIA-FLOAT-EXTREMES-BITS has landed, then branch (i) or (ii).
- Scope: `crates/eunomia/src/traits/float/power.rs`, `crates/eunomia/tests/float_element/powi_saturation.rs`.

<a id="EUNOMIA-FLOAT-EXTREMES-BITS"></a>
## EUNOMIA-FLOAT-EXTREMES-BITS — no test asserts `MAX_VALUE == INFINITY` under `T`'s own `==` [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: ac61854df78a45a2144bf3aa9dc0c21858fa892f.
- Outcome: for all ten `FloatElement` implementors `MAX_VALUE == INFINITY` under `T`'s own `==` (the comparison `saturates` makes) is pinned by a test, not by reading.
- Evidence (`ac61854df78a45a2144bf3aa9dc0c21858fa892f`, read): `reduction_identities_are_the_format_infinity_for_every_float` (`crates/eunomia/tests/float_special/finite_extremes.rs:50-86`)
  compares `to_f64` values of `MAX_VALUE`, `INFINITY` and the identity for f32, f64, F32, F64, F16, Bf16, Bf8, Bf4, F8 and F4, never `==` on `T`; the wrappers'
  `PartialEq` is float-semantic (`types/floats.rs:6`). The bits are already pinned for `Bf4`, `F8` and `F4`, the formats whose `INFINITY` is finite, by
  `numeric_constants_match_subbyte_value_contracts` (`crates/eunomia/tests/subbyte_convert.rs:107-125`: 0x05, 0x77, 0x06); the other seven have one infinity encoding.
  The constants are `impls/primitives/numeric.rs:26,31` and `impls/wrappers/numeric.rs` `$inf`/`$max`; `NumericElement` is sealed (`traits/numeric.rs:16-17`), so the ten are all.
- Acceptance: `assert_reduction_identities` (`finite_extremes.rs:54-70`) also asserts `T::MAX_VALUE == T::INFINITY`:
  `git grep -n -e 'MAX_VALUE ==' -e '== .*INFINITY' -e 'assert_eq!(.*MAX_VALUE, .*INFINITY' -e 'MAX_VALUE,$' -- crates/eunomia/tests/float_special/finite_extremes.rs` prints a line
  (`assert!` of `==` or `assert_eq!`; both compare under `T`'s own `PartialEq`, so the form is free: #213 round 2 N-2; the last pattern matches `T::MAX_VALUE,` alone on a line, where rustfmt wraps `assert_eq!`), and nothing at the basis. No bits assertion is added; the subbyte test owns them.
- Next step: add the `==` assertion to `assert_reduction_identities`; EUNOMIA-POWI-SATURATION-GUARD needs it.
- Scope: `crates/eunomia/tests/float_special/finite_extremes.rs`.

<a id="EUNOMIA-POWI-QUOTIENT-BIAS-MUTANT"></a>
## EUNOMIA-POWI-QUOTIENT-BIAS-MUTANT — a downward 2^-20 bias on powi's saturated quotient survives every test [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: ac61854df78a45a2144bf3aa9dc0c21858fa892f.
- Outcome: mutant M2e, the accumulator quotient at `power.rs:142` times `1 - 2^-20` with its sign kept, is killed by a row, or is
  recorded as an equivalent mutant together with the largest exponent `k` whose bias `2^-k` a row kills.
- Evidence: relayed from the PR #210 judge (`/808` F-3, head `2ef8acd`; the saturated branch reads as at `2ef8acd` `power.rs:111-113`, but `power.rs` and the tests have changed since): M2e passes 234 of 234 and no
  mutant-specific failing row shows which binary ran; M2d (`1 - 2^-4`) fails at `F8` `powi(1.5, -17)`, and the upward biases fail.
  The judge's reading: the sweep is exhaustive for e ≤ 128 and every result beyond is zero; a tie occurs only for power-of-two bases,
  and there the correctly rounded neighbour is the lower one or zero.
- Acceptance: on the closing head M2e is rerun. (a) A row fails, or one is added that fails: the closing PR names it, which shows the mutated binary ran.
  (b) None fails: the bias 2^-k is bisected over the integers 4 <= k <= 20 (2^-20 survives, 2^-4 kills; a larger downward bias never raises a rounded result, so the rows it changes only grow) to adjacent exponents,
  the PR names the largest k whose bias a row kills and that row, shows 2^-(k+1) survives, and the module doc of `powi_saturation.rs` states that biases at or below 2^-(k+1) change no result and why (ties round down or to zero):
  `git grep -n -i bias -- crates/eunomia/tests/float_element/powi_saturation.rs` prints a line, and nothing at the basis.
- Next step: apply each bias to a `git archive` export of the head, touch the files, and run `cargo nextest run -p eunomia --test float_element`.
- Scope: `crates/eunomia/tests/float_element/powi_saturation.rs`.

<a id="EUNOMIA-POWI-ROUNDOFF-OWNER"></a>
## EUNOMIA-POWI-ROUNDOFF-OWNER — the `powi` bound names `u` without saying whose roundoff it is [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: ac61854df78a45a2144bf3aa9dc0c21858fa892f.
- Outcome: the `FloatElement::powi` Rustdoc says `u` is the unit roundoff of `Self`, as `power.rs:13,64` do for `T`.
- Evidence (`ac61854df78a45a2144bf3aa9dc0c21858fa892f`, read; the PR #210 judge's `/808` F-2, first half): `float.rs:379` reads "Roundoff `u`" with no owner,
  and `git grep -n -F 'unit roundoff' -- crates/eunomia/src/traits/float.rs` prints nothing; `power.rs:13,64` name it.
- Acceptance: with `DOC` = ``awk '/^ *\/\/\//{b=b" "$0;next} /^ *#\[/{next} /fn powi\(self/{print b} {b=""}' crates/eunomia/src/traits/float.rs | sed -E 's#/// ?##g' | tr -s ' '`` (the doc comment above `fn powi(self` as one line, wherever its summary sits), ``DOC | awk '{gsub(/unit roundoff of \x60Self\x60\x27/,"")} /unit roundoff of \x60Self\x60/{n++} END{print n+0}'`` prints 1 (0 at the basis), so ``unit roundoff of `Self` `` stands and the same words with an apostrophe after `Self` do not; the bare phrase `unit roundoff`, "with unit roundoff `u`" and the possessive "`Self`'s accumulator" name no owner or the wrong one, and a wrap between `of` and `Self` is accepted;
  `float.rs` counts at most 500 lines (newlines plus one; 476 at the basis) and keeps `git grep -n -P '(*UTF)^.{101,}$' -- crates/eunomia/src/traits/float.rs` empty (no line over 100 characters), which the added words in `:379` must not break (98 characters at the basis).
- Next step: reword `float.rs:379`, rewrapping `:379-382` if the added words need it.
- Scope: `crates/eunomia/src/traits/float.rs`.

<a id="EUNOMIA-COMPLEX-POWI-EXACT-CLAIM"></a>
## EUNOMIA-COMPLEX-POWI-EXACT-CLAIM — Complex::powi is called exact but rounds its products [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: 2327b325c5fc.
- Outcome: no source or test text calls `Complex::powi` exact, and its Rustdoc states what it guarantees. `crates/eunomia/src/types/complex/float.rs:132-137`
  says "**exact** exponentiation-by-squaring" and "exact up to the floating-point error of the multiplies"; the loop at `:150-158` rounds each complex
  product and a negative `n` adds one division. Test `powi_is_exact_repeated_multiplication` (`:236`) has four `1e-12` literal tolerances in two assertions, the counted underived class.
- Evidence (`0ae80384255f`, read): the Rustdoc, test name and tolerances above; `git grep -n -i powi` over `backlog.md`, `backlog/` and `gap_audit.md` finds no item for them.
- Acceptance: `git grep -n -i 'exact' crates/eunomia/src/types/complex/float.rs` prints no line about `powi`; the test is renamed for the property it checks
  and each tolerance is a named constant whose doc derives it from a per-product error bound in a cited source, or the assertion is exact equality on inputs whose products and quotient are exact
  (representable is not enough: `w = (2-5i)/4` has exact products, but `w^-3 = (-9088-4160i)/29^3` is not dyadic);
  the `w.powi(7)` and `w.powi(-3) * w.powi(3)` checks (`:250-254`) stay, for example `(1 + i)^7 = 8 - 8i` and `(1 + i)^-2 = -i/2`.
- Next step: read the complex-multiplication error bound in a numerical-analysis reference before citing it; then reword the Rustdoc to state the number of
  rounded complex products (the first, one times the base, is exact) and that bound.
- Scope: `crates/eunomia/src/types/complex/float.rs`.

<a id="EUNOMIA-COUNT-CAST-EXPECT"></a>
## EUNOMIA-COUNT-CAST-EXPECT — `convert/count.rs` expects one cast lint but holds a narrowing cast [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: e5b65f5edae5.
- Outcome: the module-scoped `#![expect]` in `crates/eunomia/src/convert/count.rs` names every cast lint that fires in the module, with a reason for each.
- Evidence (read; verdict F5 of eunomia#182 `0dccc32c0da7`): `count.rs:15-19` expects only `clippy::cast_precision_loss` (integer to float); `F32::from_float` holds `value as f32`,
  an `f64`-to-`f32` narrowing, which is `clippy::cast_possible_truncation` (class recalled, not observed firing: `Cargo.toml:38` allows it workspace-wide).
- Acceptance: `cargo clippy -p eunomia --all-targets -- -W clippy::cast_possible_truncation -W clippy::cast_precision_loss` reports no warning with a `count.rs` span,
  and each lint it showed firing is in the `expect` with its reason; the per-class counts beside `Cargo.toml:38-44` are unchanged or lowered.
- Next step: run that command at the basis and read the spans before editing the `expect`.
- Scope: `crates/eunomia/src/convert/count.rs`.

<a id="EUNOMIA-COUNT-TEST-MODULE"></a>
## EUNOMIA-COUNT-TEST-MODULE — `convert/count.rs` splits its tests across two modules [patch]
- status: todo; priority: tightening; milestone: M-special-functions; needs: none; basis: e5b65f5edae5.
- Outcome: the `count.rs` unit tests live in one place: the inline `mod native_float_tests` joins the `mod tests;` file module (`convert/count/tests.rs`).
- Evidence (read; verdict F8 of eunomia#182): `count.rs:296-297` declares `#[cfg(test)] mod tests;` and `:299-300` an inline `mod native_float_tests`, holding
  `constant_narrowing_preserves_analytic_boundaries` and `constant_narrowing_matches_runtime_across_every_exponent_and_sign`.
- Acceptance: `git grep -n 'mod native_float_tests' -- crates` prints nothing; `cargo nextest list -p eunomia` lists both tests above and the same test names as at the basis otherwise
  (diff of the name lists, not counts); both pass with the 23-row bit table (46 cases at two signs) and the 49,152-encoding grid unchanged.
- Next step: move the module body into `convert/count/tests.rs`, with no assertion edit.
- Scope: `crates/eunomia/src/convert/count.rs`, `crates/eunomia/src/convert/count/tests.rs`.

<a id="EUNOMIA-FROM-F64-RENAME"></a>
## EUNOMIA-FROM-F64-RENAME — `FloatElement::from_f64` names its source type [major]
- status: todo; priority: tightening; milestone: M-special-functions; needs: none; basis: e5b65f5edae5.
- Outcome: the trait method `FloatElement::from_f64` (`crates/eunomia/src/traits/float.rs:23`) carries a name that holds no source type, as the integrity conversion rule schedules for the next change of that method.
- Evidence (read; verdict F6 of eunomia#182): #182 changed its primitive and wrapper implementations without the rename; `git grep -c from_f64 -- crates` sums to 111 lines at the basis.
  A rename of a public trait method is a SemVer break, admitted by the 0.13.0 milestone.
- Acceptance: `git grep -n -w from_f64 -- crates` prints nothing (it also matches the inherent `F16::from_f64`/`Bf16::from_f64`, `types/floats.rs:177,293`, and the macro parameter `$from_f64`, `impls/wrappers/float.rs:12,15,26`: the rename covers them);
  every in-stack caller is updated in the same co-evolution unit; the replacement name and migration are in an ADR drafted first;
  `cargo semver-checks` reports the break under `[major]`.
- Next step: draft the ADR with the recommended name and a caller count per consumer repository.
- Scope: `crates/eunomia/src/traits/float.rs`, `crates/eunomia/src/types/floats.rs`, its implementations, tests and docs, `CHANGELOG.md`, `docs/adr/`.

<a id="EUNOMIA-CHECKLIST-RETIRE"></a>
## EUNOMIA-CHECKLIST-RETIRE — root `checklist.md` is a retired board form [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: bf0bce90eabd.
- Outcome: `checklist.md` (236 lines) is deleted with no open state lost: each of its 12 open boxes is closed with evidence or lives in a board item, as hyperion#107 did.
  The board lint counts the file (`board_checklist_files` in `scripts/atlas-board-lint.py`, atlas) and the conformance scan holds that count non-increasing per member.
- Evidence (`bf0bce90eabd`, read): the 12 `- [ ]` boxes sit in the 0.12.0 cohort section (3: file lines 18, 20, 21), `EUNOMIA-CONST-NATIVE-FLOAT` (4: lines 39-42), `E-037` (1),
  `ATLAS-EUNOMIA-NAN-CONTRACT-2026-08-21` (3), `ATLAS-EUNOMIA-NUMPY-CI-2026-08-20` (1). Line 21 (merge and publication) closes with the #182 merge (`b2477bd8fde1`) and the 0.12.0 publication.
  Lines 18, 20 and 40 hold the independent review of #182's final head: its post-merge review posted `judge` `failure` on `0dccc32c0da7`, and the `Item: EUNOMIA-0-12-COHORT-REVIEW` commits (`0d76dfba5928`, `188d5ca5927b`, in `bf0bce90eabd`) fixed the gating findings. Gates, checks and floors: the closing PR names their receipts; floors belong on consumer boards.
  Receipts of the `done` blocks it cites, deleted by the #216 filing sweep (each commit an ancestor of origin/main, run): `E-037` `a296e12` (PR #85); `ATLAS-EUNOMIA-NAN-CONTRACT-2026-08-21` `834bd3b`, docs `8c4510e`;
  `ATLAS-EUNOMIA-NUMPY-CI-2026-08-20` CI run `33840122785` at `02397fa` (success; NumPy feature contract job success). ADR 0010 delivers CONST-NATIVE-FLOAT in 0.12.0 (`0dccc32`, in tag `crate-eunomia-v0.12.0`). Outside the file, `docs/adr/0009-default-release-version.md:43` reads "the backlog and checklist record the release as pending".
- Acceptance: `git ls-files checklist.md` prints nothing at the closing PR head (it prints the file at the basis); the closing PR body names, for each of the 12 boxes,
  the item ID or the landed commit or receipt that closes it; `git grep -n -i checklist -- docs` is empty; `backlog.md` stays at or under 1000 lines;
  `docs/adr/0009-default-release-version.md` gains a `Revision <date>:` line for the rewording, as ADRs 0010 and 0011 have at line 7 (`git grep -n '^Revision'` over it prints nothing at the basis).
- Next step: check CONST-NATIVE-FLOAT boxes 39, 41 and 42 (42, the archive-identity check, is EUNOMIA-ADR-RELEASE-RESIDUE's acceptance; 40 is closed above) against `0dccc32`, the 0.12.0 tag and the registry, fold any still open into an item, delete the file, reword ADR 0009:43 with a revision note.
- Scope: `checklist.md`, `backlog.md`, `docs/adr/0009-default-release-version.md`.

<a id="EUNOMIA-LADDER-0-12-CLOSE"></a>
## EUNOMIA-LADDER-0-12-CLOSE — the ladder and scope ADR 0012 still treat the published 0.12 cohort as an open bundle [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: bf0bce90eabd.
- Outcome: no ladder, ADR or board text names the deleted bundle item or calls the 0.12 cohort blocked: M-0.12-cohort closes, M-1.0 drops it from `needs:`, and ADR 0011 and ADR 0012 cite what delivered it.
  Closing the block removes its outcome text, so a dated scope-ADR revision note and a judge verdict carry the change; no exit is edited in place.
- Evidence (`bf0bce90eabd`, read): `backlog/milestones.md:8` names the deleted item as the M-0.12-cohort work and `:9` has M-1.0 need the block; ADR 0012 `:13`, `:26` and `:39` say "blocked"; ADR 0011 `:6` reads `Delivery: EUNOMIA-NUMPY-030-UNRELEASED-BUNDLE`.
  Delivered (run): tag `crate-eunomia-v0.12.0` is `b2477bd8fde1`; crates.io lists eunomia 0.12.0 and eunomia-derive 0.12.0, created 2026-10-29, not yanked (`/api/v1/crates/<name>/0.12.0`).
- Acceptance: `git grep -n 'NUMPY-030-UNRELEASED-BUNDLE' -- backlog/milestones.md docs/adr/0011-numpy-030-unreleased-bundle.md` prints nothing (two lines at the basis);
  `git grep -n -i 'blocked' -- docs/adr/0012-release-scope-1-0.md` names no cohort (`:13`, `:26`, `:39` at the basis); ADR 0012 gains a dated `Revision` line; the closing change cites a judge verdict on the ladder edit.
  No release item is filed: the 0.12.0 release exists.
- Next step: draft the ADR 0012 revision note and the `backlog/milestones.md` edit, then obtain the judge verdict.
- Scope: `backlog/milestones.md`, `docs/adr/0012-release-scope-1-0.md`, `docs/adr/0011-numpy-030-unreleased-bundle.md`.

<a id="EUNOMIA-ADR-RELEASE-RESIDUE"></a>
## EUNOMIA-ADR-RELEASE-RESIDUE — ADR 0011 keeps pre-publication steps and ADR 0010 reads as a directive about consumer ranges [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: bf0bce90eabd.
- Outcome: ADR 0011 and ADR 0010 describe the 0.12.0 release as it happened: no pre-publication instruction remains, the archive-identity check is recorded or dropped, and the consumer sentence states what consumers did.
- Evidence (`bf0bce90eabd`, read; judge verdict on eunomia#214 round 1, F3, F6, F11): ADR 0011 `:80-82` reads "Coordinate Aequitas #140/141 ... before the automatic publisher runs. Verify immutable 0.12 archive identity and checksum after authorized publication.";
  no read artifact records that check (crates.io lists a sha256 checksum for eunomia 0.12.0; it was not compared with the tagged tree). ADR 0011 `:56` and `:82` are 126 and 114 characters in a file wrapped near 80.
  ADR 0010 `:53-54` says consumers of `^0.10` or `^0.11` "advance their requirement to 0.12"; the judge read the consumers widening instead: `>=0.10.0, <0.13.0` (aequitas, apollo, athena, hermes, hephaestus), `>=0.11.0, <0.13.0` (leto), `0.12.0` (coeus).
- Acceptance: `git grep -n -i -e 'before the automatic publisher' -e 'after authorized publication' -- docs/adr/0011-numpy-030-unreleased-bundle.md` prints nothing (`:81`, `:82` at the basis); `awk 'length > 90' docs/adr/0011-numpy-030-unreleased-bundle.md` prints nothing (`:56`, `:82` at the basis);
  ADR 0011 records the crates.io checksum and `.cargo_vcs_info.json` commit of eunomia and eunomia-derive 0.12.0 against the tag (the check runs: crates.io lists both checksums; checklist box 42); ADR 0010 states the observed ranges or only that such a consumer widens its range; each edited ADR gains a dated `Revision` line.
- Next step: read the two 0.12.0 archives' `.cargo_vcs_info.json` and compare their checksums with `Cargo.lock` of a consumer, then rewrite `:80-82`, rewrap `:56`, reword ADR 0010 `:53-54`.
- Scope: `docs/adr/0011-numpy-030-unreleased-bundle.md` (not `:48`, EUNOMIA-RELEASE-NOTES-GAPS), `docs/adr/0010-const-native-float-construction.md`.

<a id="EUNOMIA-RELEASE-NOTES-GAPS"></a>
## EUNOMIA-RELEASE-NOTES-GAPS — the 0.12 notes give the CPython Python floor for every interpreter and the 0.11 notes omit the reduced-precision floor [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: bf0bce90eabd.
- Outcome: `README.md`, the CHANGELOG `[0.12.0]` entry and ADR 0011 state the Python floor of the optional NumPy boundary per interpreter, and CHANGELOG `[0.11.0]` lists `FloatElement::floor` for the reduced-precision family (`9d29a66`).
- Evidence (`bf0bce90eabd`, read; judge verdict on eunomia#214 round 1, F6, F10): `README.md:62`, `CHANGELOG.md:61-62` and ADR 0011 `:48` say Python 3.9 with no interpreter; `pyo3-ffi-0.30.1/build.rs:39-51` (local registry source) sets CPython min 3.9, PyPy 3.11, GraalPy 3.10.
  `9d29a66` is an ancestor of tag `crate-eunomia-v0.11.0` and not of `crate-eunomia-v0.10.0` (run); `git grep -n -i floor -- CHANGELOG.md` prints only `:162` (`binary_exponent`), and `git grep -c -i digamma -- CHANGELOG.md` prints 1, so the search finds an entry that exists.
  The `[0.11.0]` section matches the tag's own `CHANGELOG.md`, so the entry is an addition to it.
- Acceptance: `git grep -c '3\.9' -- README.md CHANGELOG.md docs/adr/0011-numpy-030-unreleased-bundle.md` prints a count of at least 1 for each of the three files (the floor sentences still exist; deleting one fails this), and each hit of `git grep -n -B1 -A1 '3\.9'` over the same paths names the interpreter it holds for (no line does at the basis);
  `git grep -n -i floor -- CHANGELOG.md` prints a line between the `## [0.11.0]` and `## [0.10.0]` headings, worded from `git show 9d29a66`.
- Next step: reword the three floor sentences with the per-interpreter minimums, then write the entry from the commit body.
- Scope: `README.md`, `CHANGELOG.md`, `docs/adr/0011-numpy-030-unreleased-bundle.md` (`:48` only; EUNOMIA-ADR-RELEASE-RESIDUE owns the rest).

<a id="EUNOMIA-POWI-BOUND-MARGIN"></a>
## EUNOMIA-POWI-BOUND-MARGIN — `assert_outside_bound` compares in `f64` without stating why `f64` cannot flip it [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: ac61854df78a45a2144bf3aa9dc0c21858fa892f.
- Outcome: a comment above `assert_outside_bound` names the least ratio of error to bound over its callers, 8/3, and says `f64` rounding of the bound and the comparison is far smaller.
- Evidence (`ac61854df78a45a2144bf3aa9dc0c21858fa892f`, read; ratios derived in exact rationals): `crates/eunomia/tests/float_element/powi_saturation.rs:254-271` computes `bound` (`:256-261`) and compares (`:268`) in `f64`, with one doc line and no word on the margin;
  its two callers (`:276-277`) have error over bound `(900−240)/900 ÷ 1/16` = 176/15 (`F8` 30²) and `(9−3)/9 ÷ 1/4` = 8/3 (`Bf4` 3²), against `f64` relative rounding of about 2^-52. The `.powi` in the bound is `f64::powi`, not the trait method.
- Acceptance: with `ABOVE` = ``awk '/fn assert_outside_bound/{print b; exit} /^ *\/\//{b=b" "$0;next} {b=""}' crates/eunomia/tests/float_element/powi_saturation.rs`` (the comment block directly above the function as one line), ``ABOVE | grep -c -E '8/3|2\.67'``, ``ABOVE | grep -c -F 'f64'`` and ``ABOVE | grep -c -E '2\^-52|2\.2e-16|EPSILON'`` each print 1 (0, 0 and 0 at the basis), and the comment says that 8/3 is the least error-over-bound ratio of the callers and that `f64` rounding (about 2^-52 relative) is far below it;
  the test still passes: `cargo nextest run -p eunomia --test float_element`.
- Next step: add the comment above `assert_outside_bound`.
- Scope: `crates/eunomia/tests/float_element/powi_saturation.rs`.

<a id="EUNOMIA-COMPLEX-EXTREMUM-SIGNED-ZERO"></a>
## EUNOMIA-COMPLEX-EXTREMUM-SIGNED-ZERO — `Complex::max_scalar` and `min_scalar` return the first argument when parts differ only in zero sign [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: f4f6da2e038f22f21b7581c27dce127771223f42.
- Outcome: for `Complex<T>` with real `T`, `max_scalar` and `min_scalar` return one bit pattern for either argument order, as the wrapper default does (`traits/numeric.rs:81-82,110-111`: `min(-0, +0)` is `-0`, `max(-0, +0)` is `+0`; the primitive `f32` and `f64` overrides may return either zero, so the rule for `Complex64` is the `Complex` override's own), or the Complex docs state the tie rule.
- Evidence (`f4f6da2e038f22f21b7581c27dce127771223f42`, read; re-validated against judge of eunomia#231, F5; the branch logic was run in a Python model of IEEE `==`, `>` and `>=`, not the Rust code): `impls/primitives/numeric.rs:356-372` returns `self` when `re` compares equal and `im` meets `<=`/`>=`; `-0 == +0`, so
  `max_scalar((-0, 0), (+0, 0))` is `(-0, 0)` and the reversed call `(+0, 0)`; `min_scalar` is the same, and an `im` zero sign behaves alike. The real default (`traits/numeric.rs:97-99`) resolves the zero tie by sign; the Complex override never does. `tests/complex_provider_contract.rs` has no signed-zero cell (`git grep -n -i -E 'to_bits|-0\.0|signed'` over it prints nothing). The override compares the parts with `<`, `==` and `>` and calls no component `min_scalar` (ADR 0013 `:123-125` lists recursion through the component's `min_scalar` as a rejected alternative for the nested case), so the acceptance below holds under either outcome of EUNOMIA-PRIMITIVE-ZERO-TIE.
- Acceptance (either): (a) a test in `crates/eunomia/tests/complex_provider_contract.rs` makes eight calls, {`max_scalar`, `min_scalar`} x {a `Complex64` pair differing only in the sign of a zero `re`, such a pair for a zero `im`} x {both argument orders}, and asserts `to_bits` of both parts equal across the orders and equal to the wrapper rule's pattern (`-0` for `min_scalar`, `+0` for `max_scalar`);
  `cargo nextest run -p eunomia --test complex_provider_contract` fails at the basis and passes at the closing head, and fails on each of four leave-one-out fixes of `numeric.rs:356-372` that resolve the zero tie by sign in three of {`max_scalar`, `min_scalar`} x {`re` tie, `im` tie} and leave the fourth as at the basis (so a test of zero `re` only, of one function only, or of one order only passes one of them and fails this clause);
  or (b) the `Complex` impl's `max_scalar` and `min_scalar` carry a doc stating the tie rule and a doctest asserting it with `to_bits` for both functions and both orders: ``awk '/^ *\/\/\/ *\x60\x60\x60/{f=f?0:($0 ~ /\x60\x60\x60(rust)? *$/?1:2);next} f==1&&/to_bits/{a++} f==1&&/max_scalar/{b++} f==1&&/min_scalar/{c++} f==1&&/assert/{d++} END{print a+0,b+0,c+0,d+0}' crates/eunomia/src/impls/primitives/numeric.rs`` prints four nonzero counts (`0 0 0 0` at the basis; prose without a code fence, and an `ignore`, `no_run` or `text` fence, count nothing) and `cargo test --doc -p eunomia` passes; the counts check that the fences exist, and that both functions and both orders are asserted is review-judged.
- Overturned by: a stack consumer of `Complex::max_scalar`/`min_scalar` that needs the first argument back on a tie (none searched for), or a measured cost of the zero-sign branch in `Complex` reductions (none exists); either moves the choice to option (b) with the first-argument rule documented.
- Next step: decide the tie rule (the wrapper rule applied per component, `min(-0, +0)` is `-0` and `max(-0, +0)` is `+0`, is the recommended one), then add the test or the doc; the priority becomes `correctness` if the rule is the wrapper rule and the override is changed.
- Scope: `crates/eunomia/src/impls/primitives/numeric.rs`, `crates/eunomia/tests/complex_provider_contract.rs`.

<a id="EUNOMIA-ADR3-BYTEMUCK-GATING"></a>
## EUNOMIA-ADR3-BYTEMUCK-GATING — ADR 0003 D2 says the bytemuck interop is unconditional while the ladder and E-026 plan to gate it [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: 0a6bfda3a8a67d548edbcb3f4bb830b5c7f2af9f.
- Outcome: ADR 0003 D2 and the board agree on whether `bytemuck` becomes an optional dependency, and the M-layout-derives exit no longer names the merged branch `feat/eunomia-layout-derives`.
- Evidence (`0a6bfda3a8a67d548edbcb3f4bb830b5c7f2af9f`, read; judge of eunomia#226, filed): ADR 0003 `:60` reads "interop is unconditional rather than feature-gated" and `crates/eunomia/Cargo.toml:13` declares `bytemuck` non-optional, as D2 says; `backlog/milestones.md:7`, E-026 (`backlog.md`) and `gap_audit.md:71,172` plan "bytemuck optional, default-on" gating.
  E-026 names hephaestus as the `default-features = false` consumer that relies on `bytemuck::Pod`; hephaestus (origin/master `fbc786279dd4`) has 0 `bytemuck` manifests and 0 `.rs` files naming it, coeus (origin/main `fbfbaafff8d2`) has 7 and 8. `milestones.md:6` names `feat/eunomia-layout-derives`, the head of PR #86 (merged; #87 delivered from `fix/eunomia-layout-derives-delivery`); neither remote branch exists.
- Acceptance: with A = `git grep -n 'unconditional rather than feature-gated' -- docs/adr/0003-native-byte-layout-and-reduced-precision.md | wc -l` and B = `git grep -n -i -E 'bytemuck.{0,12}(gating|optional)|(gating|optional).{0,12}bytemuck' -- backlog.md backlog/milestones.md gap_audit.md | wc -l`, A prints 1 and B prints 4 at the basis, and at the closing head exactly one of them prints 0 and the other at least 1 (the decision is one-sided);
  `git grep -n 'feat/eunomia-layout-derives' -- backlog/milestones.md` prints nothing (`:6` at the basis); the closing change adds a dated `Revision` line to ADR 0003 (D2 changed) or to ADR 0012 with a judge verdict on the ladder edit (exit retired).
- Next step: record the decision; recommended option: keep D2 and retire the gating exit, since the code matches D2 and the driving consumer no longer depends on `bytemuck` (a veto point). Overturned by: a `default-features = false` consumer that must keep `bytemuck` out of its graph; none is read (hephaestus `fbc786279dd4` has 0 manifests and 0 files naming it, coeus `fbfbaafff8d2` uses it in 7 and 8), or a measurement that the unconditional `bytemuck` edge costs a consumer build time or binary size.
- Scope: `docs/adr/0003-native-byte-layout-and-reduced-precision.md`, `docs/adr/0012-release-scope-1-0.md`, `backlog/milestones.md`, `backlog.md` (E-026), `gap_audit.md` (`:71`, `:172`).

<a id="EUNOMIA-CHANGELOG-NEON-VERIFIED"></a>
## EUNOMIA-CHANGELOG-NEON-VERIFIED — the CHANGELOG says the NEON f8 decode was verified on aarch64 and no aarch64 run exists [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: 0a6bfda3a8a67d548edbcb3f4bb830b5c7f2af9f.
- Outcome: the `CHANGELOG.md` E-030 entry states what ran: the exhaustive differential exists as an aarch64-only test that has not been executed (E-030), or cites its run once E-030 closes.
- Evidence (`0a6bfda3a8a67d548edbcb3f4bb830b5c7f2af9f`, read; judge of eunomia#226, filed): `CHANGELOG.md:248-253` reads "bit-exact against the scalar conversion kernel ... verified by an exhaustive phase-and-length differential test on aarch64";
  the test `neon_f8_unpack_bit_matches_scalar_for_every_byte_and_phase` is `cfg(target_arch = "aarch64")` (`crates/eunomia/tests/subbyte_convert.rs:307`) and every `runs-on` under `.github/workflows` is `ubuntu-latest` (`ci.yml:40,68,150,198`, `msrv.yml:39`, `book-pages.yml:54`); E-030 records that the test "was not executed". `gap_audit.md` G-T2 carried the same claim and is corrected by the filing PR.
- Acceptance: `git grep -n 'verified by an exhaustive phase-and-length differential test on' -- CHANGELOG.md` prints nothing (`:251` at the basis); `git grep -c 'neon::unpack_f8_to_f32' -- CHANGELOG.md` prints 1 (the entry survives; deleting it fails);
  `git grep -n -i 'not executed' -- CHANGELOG.md` prints a line inside that entry (none at the basis), unless E-030 has closed with a recorded run, which the entry then cites.
- Next step: reword `CHANGELOG.md:248-253` from E-030's text.
- Scope: `CHANGELOG.md`.

<a id="EUNOMIA-AVX2-F8-TEST-COMMENT"></a>
## EUNOMIA-AVX2-F8-TEST-COMMENT — the AVX2 f8 differential test's doc comment calls a gather decode branchless and cites E-031 [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: 0a6bfda3a8a67d548edbcb3f4bb830b5c7f2af9f.
- Outcome: the doc comment above `avx2_f8_unpack_bit_matches_scalar_for_every_byte` describes the decode it tests: a 256-entry table gather, retained after E-032 measured the branchless form slower.
- Evidence (`0a6bfda3a8a67d548edbcb3f4bb830b5c7f2af9f`, read; judge of eunomia#226, filed): `crates/eunomia/tests/subbyte_convert.rs:280` reads "the branchless AVX2 `f8 -> f32` decode (E-031)"; `avx2::unpack_f8_to_f32` (`crates/eunomia/src/packed/unpack/intrinsics/avx2.rs:274-302`) indexes `TABLE_BITS: [u32; 256]` through `_mm256_i32gather_ps` (`:294`);
  E-032 records the branchless form implemented, about 27% slower, and the gather kept; E-031 is the F16C bulk conversion (`CHANGELOG.md:327`).
- Acceptance: `git grep -n 'branchless AVX2' -- crates` prints nothing (`subbyte_convert.rs:280` at the basis); `git grep -n 'fn avx2_f8_unpack_bit_matches_scalar_for_every_byte' -- crates/eunomia/tests/subbyte_convert.rs` still prints 1 line (the test survives); `cargo nextest run -p eunomia --test subbyte_convert` passes. The change is comment-only.
- Next step: reword the comment to name the gather and cite E-032.
- Scope: `crates/eunomia/tests/subbyte_convert.rs` (the doc comment only).

<a id="EUNOMIA-NOSTD-AVX512-DETECTION-TEST"></a>
## EUNOMIA-NOSTD-AVX512-DETECTION-TEST — no test pins the no_std AVX-512 detection arms that E-028 corrected [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: 0a6bfda3a8a67d548edbcb3f4bb830b5c7f2af9f.
- Outcome: a test fails when either no_std AVX-512 detection arm drops its `avx512vl` requirement, the defect E-028 fixed without a test.
- Evidence (`0a6bfda3a8a67d548edbcb3f4bb830b5c7f2af9f`, read; judge of eunomia#226, F1): `crates/eunomia/src/packed/unpack/arch.rs:12-17` (`has_avx512bw`) and `:34-38` (`has_avx512f`) are `cfg!(target_feature = ...)` expressions under `#[cfg(not(feature = "std"))]`; `git grep -n 'has_avx512' -- crates/eunomia/tests crates/eunomia/benches` prints nothing and `arch.rs` has no `cfg(test)` module.
  The CI feature matrix runs `cargo check`, not tests, for `--no-default-features` (`.github/workflows/ci.yml:116-130`), at default target features where `cfg!(target_feature = "avx512f")` is false, so no run evaluates an arm true. E-028 recorded "No unit test"; its entry was deleted from the board and the limit is recorded nowhere else.
- Acceptance: `git grep -n 'no_std_avx512_detection' -- crates` prints a test definition (nothing at the basis), `cargo nextest run -p eunomia --no-default-features --lib` passes it, and it fails against each of two mutants of `arch.rs` that delete the `avx512vl` requirement from the no_std arm of `has_avx512bw`, or of `has_avx512f`.
- Next step: express each arm as a `const fn` of the feature booleans so a table-driven test covers every combination without AVX-512 hardware, and have the `cfg!` arms call it.
- Scope: `crates/eunomia/src/packed/unpack/arch.rs`.

<a id="EUNOMIA-ZERO-TIE-TEST-SHAPE"></a>
## EUNOMIA-ZERO-TIE-TEST-SHAPE — eight type-named tests pin the wrapper zero tie that one property test would pin [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: f4f6da2e038f22f21b7581c27dce127771223f42.
- Outcome: `float_order.rs` pins the wrapper zero tie in one test named for the property: for each of `F32`, `F64`, `F16`, `Bf16`, `Bf8`, `Bf4`, `F8` and `F4` it calls `min_scalar` and `max_scalar` on `(+0, -0)` and `(-0, +0)` (the two orders and two functions the eight tests cover today), compares `to_bits` or the raw code, never float `==`, and reports every mismatching type, function and order in one failure.
- Evidence (`f4f6da2e038f`, read; judge of eunomia#231, F1): `crates/eunomia/tests/float_order.rs:307-333` holds `wrapper_zero_test!` and eight invocations named `f32_`, `f64_`, `f16_`, `bf16_`, `bf8_`, `bf4_`, `f8_` and `f4_wrapper_zero_sign`; the loop at `:313-317` covers both orders and both functions per type. Standards (generic instantiation coverage) ask for one body over the shipped type list named for the property.
  Older tests in the file (`bf8_orders_as_ieee754`, `f16_negatives_sort_below_positives`) carry the same naming and stay out of scope.
- Acceptance: `git grep -n -E '(f32|f64|f16|bf16|bf8|bf4|f8|f4)_wrapper_zero_sign' -- crates/eunomia/tests/float_order.rs | wc -l` prints 0 (8 at the basis, run); the unmutated `cargo nextest run -p eunomia --test float_order` passes.
  Leave-one-out: each of four mutants inserts one statement first inside the `self == Self::ZERO && other == Self::ZERO` branch of `crates/eunomia/src/traits/numeric.rs` and makes exactly one (function, order) cell return the wrong zero: M1 in `min_scalar` `if !has_sign_bit(self) && has_sign_bit(other) { return self; }` (cell `min(+0, -0)`), M2 in `min_scalar` `if has_sign_bit(self) && !has_sign_bit(other) { return other; }` (`min(-0, +0)`), M3 in `max_scalar` `if !has_sign_bit(self) && has_sign_bit(other) { return other; }` (`max(+0, -0)`), M4 in `max_scalar` `if has_sign_bit(self) && !has_sign_bit(other) { return self; }` (`max(-0, +0)`).
  Under each Mi, `cargo nextest run -p eunomia --test float_order > "$SCRATCH/out" 2>&1` fails, `grep -E '^ +FAIL ' "$SCRATCH/out" | awk '{print $NF}' | sort -u | wc -l` prints 1, and `grep -o -i -w -E 'f32|f64|f16|bf16|bf8|bf4|f8|f4' "$SCRATCH/out" | tr A-Z a-z | sort -u | wc -l` prints 8 (`$SCRATCH` is the session scratch directory, never the repository). Derived, not run: a test that skips one order, one function or one type passes the mutant of its skipped cell.
- Next step: write one function over the eight wrappers, taking each positive zero, negative zero and encoding, called from one test that collects mismatches; delete the macro and its eight invocations.
- Scope: `crates/eunomia/tests/float_order.rs`.

<a id="EUNOMIA-ZERO-TIE-CHANGELOG"></a>
## EUNOMIA-ZERO-TIE-CHANGELOG — CHANGELOG `[Unreleased]` has no entry for the wrapper `min_scalar`/`max_scalar` zero-sign change [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: EUNOMIA-PRIMITIVE-ZERO-TIE; basis: f4f6da2e038f22f21b7581c27dce127771223f42.
- Outcome: `CHANGELOG.md` `[Unreleased]` has a `### Fixed` entry saying that `min_scalar` and `max_scalar` of `F16`, `Bf16`, `F32`, `F64`, `Bf8`, `Bf4`, `F8` and `F4` return `-0` and `+0` respectively for a mixed-sign zero pair in either argument order, and the primitive `f32`/`f64` sentence ADR 0005 holds once EUNOMIA-PRIMITIVE-ZERO-TIE closes (the need).
- Evidence (`f4f6da2e038f`, read; judge of eunomia#231, F2): `[Unreleased]` holds `### Added`, `### Breaking` and `### Migration` only (`CHANGELOG.md:5-30`). Convention (run): `git log -6 origin/main -- CHANGELOG.md` shows fix commits adding entries by hand (`3320574`, `ac9729b`), and `git ls-tree -r origin/main --name-only` lists no changelog generator configuration (`cliff`, `release-plz`, `release.toml`); this item follows that history, while stack policy generates the changelog from conventional-commit subjects at release, a conflict this item does not resolve.
  At tag `crate-eunomia-v0.12.1` the default selected the zero with `is_negative_zero`, `value == T::ZERO && value.bitand(T::SIGN_MASK) == T::SIGN_MASK` (`traits/numeric.rs:289-291`, read), true for `+0` under float equality: derived, not run, that the released wrappers returned the first argument of `min_scalar` and the second of `max_scalar` for a zero pair.
- Acceptance: with `J` = ``$(awk '/^## \[Unreleased\]/{u=1;next} /^## \[/{u=0} u&&/^### /{f=($0=="### Fixed");next} u&&f' CHANGELOG.md | tr '\n' ' ' | tr -s ' ')`` (the joined `### Fixed` text), ``for w in min_scalar max_scalar F16 Bf16 F32 F64 Bf8 Bf4 F8 F4; do printf '%s' "$J" | grep -c -w -- "$w"; done | sort -u`` prints `1` (`0` at the basis, where `J` is empty), and ``printf '%s' "$J" | grep -c -E 'either( signed)? zero'``, ``printf '%s' "$J" | grep -c -F -- '-0'`` and ``printf '%s' "$J" | grep -c -F -- '+0'`` each print 1 (0 at the basis). The clauses check that the entry names every type, both functions, both zeros and the primitive caveat; that its sentences are true is review-judged.
- Next step: run `min_scalar` and `max_scalar` on a mixed-sign pair of each wrapper at the 0.12.1 tag, word the entry from what ran, and add it.
- Scope: `CHANGELOG.md`.

<a id="EUNOMIA-ADR5-REVISION-NOTE"></a>
## EUNOMIA-ADR5-REVISION-NOTE — ADR 0005 narrowed its Decision to the wrappers without a dated revision note [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: f4f6da2e038f22f21b7581c27dce127771223f42.
- Outcome: ADR 0005 carries one dated `Revision` note saying its Decision moved from "all real floating-point implementations" to the eight wrappers, that primitive `f32` and `f64` return either zero on a mixed-sign tie, and the evidence for the change (eunomia#231).
- Evidence (`f4f6da2e038f`, read; judge of eunomia#231, F3): the Decision opened "one value contract for all real floating-point implementations" and said the primitive overrides "implement the same table" (`git show 9ccfb01^:docs/adr/0005-real-scalar-min-max-special-values.md`, lines 23 and 36); at the basis it reads "for the real floating-point wrappers" and lets the primitives return either zero (`:23`, `:34-36`).
  ADR governance rewrites a changed decision in place with a dated revision note; `git grep -n '^Revision' -- docs/adr/0005-real-scalar-min-max-special-values.md` prints nothing, while ADRs 0004, 0010, 0011 and 0013 carry them (`0013:8`).
- Acceptance: with `R` = ``$(awk '/^## Context/{exit} /^(- )?Revision [0-9][0-9][0-9][0-9]-/{r=1} r' docs/adr/0005-real-scalar-min-max-special-values.md | tr '\n' ' ' | tr -s ' ')`` (the revision text, joined so a wrapped phrase counts), ``for p in wrapper primitive f32 'all real floating-point' '#231'; do printf '%s' "$R" | grep -c -F -- "$p"; done | sort -u`` prints `1` (`0` at the basis, where `R` is empty). The clauses check that the note exists, is dated and names the narrowing and its evidence; that it is true is review-judged.
- Next step: write the note from eunomia#231's change and the sign-bit defect it fixed (`is_negative_zero` compared with float equality).
- Scope: `docs/adr/0005-real-scalar-min-max-special-values.md`.

<a id="EUNOMIA-PRIMITIVE-ZERO-TIE"></a>
## EUNOMIA-PRIMITIVE-ZERO-TIE — ADR 0005 decides either-zero for primitive `f32`/`f64` on a premise std contradicts; confirm or revise [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: f4f6da2e038f22f21b7581c27dce127771223f42.
- Outcome: ADR 0005 confirms or revises its Accepted decision that `f32` and `f64` may return either zero on a mixed-sign tie, with std's documented behaviour as the rationale and the strongest rejected alternative recorded; the trait docs, the primitive override comments and the tests agree.
- Evidence (`f4f6da2e038f`, read; judge of eunomia#231, F4 and N3): ADR 0005 `:34-36` states either-zero and `:58-59` keeps "the existing zero-cost path"; both sit beside the pre-#231 premise that the primitives "implement the same table" (`git show 9ccfb01^:docs/adr/0005-real-scalar-min-max-special-values.md`, line 36), which std does not give: core 1.97.0 `library/core/src/num/f32.rs:1027-1033` (`max`) and `:1054-1060` (`min`) say that for inputs that compare equal, `+0.0` and `-0.0` included, either "may be returned non-deterministically" (read at the pinned toolchain). `impls/primitives/numeric.rs:94-103` returns `self.min(other)` and `self.max(other)`.
  So `F32` and `f32` may return different zeros, and a reduction over `f32` may change its zero sign with target or optimization (derived from the std text and the two code paths, not run). The only other signed-zero tie item is EUNOMIA-COMPLEX-EXTREMUM-SIGNED-ZERO (`Complex`, not primitives).
- Acceptance: `awk '/^## Alternatives rejected/{a=1;next} /^## /{a=0} a' docs/adr/0005-real-scalar-min-max-special-values.md | tr '\n' ' ' | grep -c -i -E 'primitive.*f32|f32.*primitive'` prints 1 (0 at the basis, run); `git grep -n 'may return either' -- crates/eunomia/src/traits/numeric.rs | wc -l` prints 2 if the decision keeps either-zero and 0 if it adopts the wrapper rule (2 at the basis); that the rationale cites std's text and the rejected alternative is the strongest one is review-judged.
  Under the wrapper rule, `float_order.rs` asserts the wrapper rule's `to_bits` for `f32` and `f64`, and M1-M4 of EUNOMIA-ZERO-TIE-TEST-SHAPE (`has_sign_bit(x)` read as `x.is_sign_negative()`, each guarded by `core::mem::size_of::<Self>() == S` for one of `f32`, `f64`) inserted first in the matching primitive override each fail `cargo nextest run -p eunomia --test float_order`, and the unmutated run passes; hypothesis, unrun: at the basis the unmutated run fails, since std leaves the tie to the target.
- Overturned by: a stack consumer's contract needing one zero sign from `f32`/`f64` reductions (none searched for), or a criterion measurement of the both-zero branch on `f32`/`f64` `min_scalar`/`max_scalar` that shows its cost within noise (none exists); either moves the decision to the wrapper rule, and the priority becomes `correctness` if a consumer contract is found violated.
- Next step: draft the ADR 0005 revision; recommended option, a veto point: keep either-zero, which keeps the native instruction path and adds no branch to `f32`/`f64`; the pin for that option is EUNOMIA-PRIMITIVE-ZERO-TIE-TEST.
- Scope: `docs/adr/0005-real-scalar-min-max-special-values.md`, `crates/eunomia/src/impls/primitives/numeric.rs`, `crates/eunomia/src/traits/numeric.rs`, `crates/eunomia/tests/float_order.rs`.

<a id="EUNOMIA-PRIMITIVE-ZERO-TIE-TEST"></a>
## EUNOMIA-PRIMITIVE-ZERO-TIE-TEST — no test asserts that a primitive mixed-sign zero pair gives a zero [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: f4f6da2e038f22f21b7581c27dce127771223f42.
- Outcome: one test calls `min_scalar` and `max_scalar` on `f32` and `f64` with `(+0, -0)` and `(-0, +0)`, eight calls, and asserts each result is a zero of either sign (`abs().to_bits() == 0`), so the either-zero contract of the primitives has a failing case in every call.
- Evidence (`f4f6da2e038f`, read; judge of eunomia#231, F7 and N4): before #231 `assert_real_min_max_contract::<f32>` and `::<f64>` asserted `min_zero == negative_zero` under float equality (`git show 9ccfb01^:crates/eunomia/tests/float_order.rs`, lines 285 and 303), which fails when the override returns a nonzero value; the change removed it for the primitives and added nothing.
  Searched at the basis, not claimed complete: direct `min_scalar`/`max_scalar` calls in `crates/eunomia/tests` (`float_element.rs:102-103`, `integer_element.rs:42-43`, `complex_provider_contract.rs`, `float_order.rs`) and the `RealField::clamp` calls of `float_order.rs:337-339` pass no mixed-sign zero pair to a primitive.
- Acceptance: for each function F in `min_scalar` and `max_scalar`, size S in 4 and 8 (`f32`, `f64`) and order (P, Q) in (true, false) and (false, true), the statement `if core::mem::size_of::<Self>() == S && self == 0.0 && other == 0.0 && self.is_sign_positive() == P && other.is_sign_positive() == Q { return 1.0; }` first in override F of `impls/primitives/numeric.rs:96-103` (eight mutants, each changing one call's result to a nonzero value) makes `cargo nextest run -p eunomia --test float_order` fail the new test, and the unmutated run passes. A test that omits one call passes the mutant of that call (derived, not run).
  Hypothesis, unrun: no existing test fails under any of the eight mutants.
- Next step: add the eight-call check to `float_order.rs`.
- Scope: `crates/eunomia/tests/float_order.rs`.

<a id="EUNOMIA-MINMAX-DOC-ORDER"></a>
## EUNOMIA-MINMAX-DOC-ORDER — the `min_scalar`/`max_scalar` docs no longer say the zero tie is independent of operand order [patch]
- status: todo; priority: verification; milestone: M-special-functions; needs: none; basis: f4f6da2e038f22f21b7581c27dce127771223f42.
- Outcome: the Rustdoc of `NumericElement::min_scalar` and `max_scalar` states that the wrapper default returns `-0` for `min` and `+0` for `max` in either argument order, as ADR 0005 (`:56`) and the book (`docs/book/numeric_element.md`) do, and carries a doctest asserting it.
- Evidence (`f4f6da2e038f`, read; judge of eunomia#231, F8): `crates/eunomia/src/traits/numeric.rs:81-82` and `:110-111` read "`min(-0, +0)` is `-0` in the wrapper default"; before #231 the same lines ended "`-0`, independent of operand order" (`git show 9ccfb01^:crates/eunomia/src/traits/numeric.rs`, lines 82 and 111), so `min(-0, +0)` now reads as one order. Neither doc block holds a code fence at the basis.
- Acceptance: `cargo test --doc -p eunomia -- --list 2>/dev/null | grep -c -E 'NumericElement::(min|max)_scalar'` prints 2 (0 at the basis; derived from rustdoc's listing of doctests by item path, unrun), and `cargo test --doc -p eunomia` passes. Each doctest calls its function on `F32` pairs in both orders and compares `to_bits`; that it does, and that the prose says "either order" and keeps "in the wrapper default", is review-judged.
- Next step: restore "in either operand order" to the two doc paragraphs and add the two doctests.
- Scope: `crates/eunomia/src/traits/numeric.rs` (the two doc comments only).

<a id="EUNOMIA-REALFIELD-LN10-MIN-POSITIVE"></a>
## EUNOMIA-REALFIELD-LN10-MIN-POSITIVE — `RealField` has no `LN_10` and no smallest-positive-normal constant [minor]
- status: todo; priority: feature; milestone: M-special-functions; needs: none; basis: f4f6da2e038f22f21b7581c27dce127771223f42.
- Outcome: generic code over `T: RealField` names `<T as RealField>::LN_10` and `<T as RealField>::MIN_POSITIVE`, the natural logarithm of 10 and the smallest positive normal value of `T`, without converting a literal.
- Evidence (`f4f6da2e038f`, read; relayed by the lead, not read: the reviews of horae#147 and hyperion#124 needed both): `crates/eunomia/src/traits/field.rs:19-31` declares `PI`, `TAU`, `FRAC_PI_2`, `E`, `LN_2`, `SQRT_2` and `EPSILON`; `git grep -n -E 'LN_10|MIN_POSITIVE' -- crates/eunomia/src/traits crates/eunomia/src/impls` prints one line, the comment at `traits/field.rs:50` (run). `RealField` is implemented for `f32`, `f64`, `F32` and `F64` only (`impls/field.rs:12,39,75,104`); `F16` and `Bf16` have an inherent `MIN_POSITIVE` (`types/floats.rs:108,226`), not a trait constant.
  `FloatElement` is sealed (`traits/float.rs:9`) and `RealField` extends it, so a required constant breaks no downstream implementor (derived): [minor], additive.
- Acceptance: `git grep -n -E 'const (LN_10|MIN_POSITIVE):' -- crates/eunomia/src/traits/field.rs | wc -l` prints 2 (0 at the basis); a test `crates/eunomia/tests/real_field_constants.rs` with one body instantiated over `f32`, `f64`, `F32` and `F64` compares each constant by `to_bits` with `core::fN::consts::LN_10` and `fN::MIN_POSITIVE` (`cargo nextest run -p eunomia --test real_field_constants` passes; at the basis it does not compile). It fails under each of eight mutants (derived, not run): `LN_10` set to `LN_2`, or `MIN_POSITIVE` set to `EPSILON`, in any one of the four impls.
  The ladder has no exit check naming this feature (`backlog/milestones.md`); `M-special-functions` is the nearest block, and admitting it with an exit check or recording a non-goal is a ladder decision under a judge verdict.
- Next step: add the two constants to `RealField` and the four impls from `core`'s constants, with the test.
- Scope: `crates/eunomia/src/traits/field.rs`, `crates/eunomia/src/impls/field.rs`, `crates/eunomia/tests/real_field_constants.rs`, `CHANGELOG.md`.

<a id="eunomia-byte-layout-reduced-precision"></a>
## Byte-layout & reduced-precision (open residue of E-022…E-033)

Delivered slices are recorded in [CHANGELOG.md](CHANGELOG.md), [ADR 0003](docs/adr/0003-native-byte-layout-and-reduced-precision.md)
and [gap_audit.md](gap_audit.md) §Byte-layout / reduced-precision. The entries below are the
open residue: E-024 (on demand), E-026 gating and E-027 (consumer co-evolution),
E-030 (aarch64 execution evidence), E-032 (recorded finding).

- **E-024 [arch] — milestone: M-ecosystem.** OCP-MXFP `FP8`(E4M3)/`FP4`(E2M1) formats (no infinity) via a
  kernel special-value **policy parameter**, added when coeus/hephaestus
  quantization needs them. Dep: a driving consumer (E-023 is delivered, CHANGELOG.md). Acceptance: OCP
  reference vectors match; existing IEEE-style types unchanged. ADR 0003 D3.
- **E-026 [arch] — gating deferred** The native `layout` vocabulary is delivered
  ([CHANGELOG.md](CHANGELOG.md), ADR 0003 D2).
  **Deferred (co-evolution, pairs with E-027):** making `bytemuck` an optional
  (default-on) feature and gating its impls is NOT purely additive —
  `default-features = false` consumers (e.g. hephaestus GPU buffers) rely on
  `eunomia::Complex: bytemuck::Pod`, so it requires consumer verification, not a
  solo eunomia commit. Milestone: M-layout-derives (E-026 gating exit).
- **E-027 [arch] — todo; milestone: M-layout-derives.** Migrate the consumer
  GPU-ABI structs onto the E-026 vocabulary while preserving the
  `bytemuck::Pod` wgpu contract via the bridge. Dep: E-026. <a id="e-027"></a>
  The provider derives are delivered (`eunomia-derive`, PRs #86 and #87;
  [CHANGELOG.md](CHANGELOG.md), ADR 0003 D2); their branch is merged and deleted.
  Consumer state by `git grep` at the fetched default branches: hephaestus (origin/master
  `fbc786279dd4`) has `eunomia` as a direct dependency, 0 `bytemuck` manifests, 0 `.rs` files
  naming `bytemuck`, and 134 lines matching `derive\(.*(Pod|Zeroable)`; coeus (origin/main
  `fbfbaafff8d2`) has `bytemuck` in 7 manifests and 8 `.rs` files and 0 such lines.
  The remaining migration is coeus.
- **E-030 [patch] — NEON f8 decode delivered; aarch64 execution evidence open**
  `neon::unpack_f8_to_f32` is a branchless 16-element arithmetic decode ([CHANGELOG.md](CHANGELOG.md);
  `crates/eunomia/src/packed/unpack/intrinsics/neon.rs`). Open: the exhaustive NEON-versus-scalar
  differential `neon_f8_unpack_bit_matches_scalar_for_every_byte_and_phase`
  (`crates/eunomia/tests/subbyte_convert.rs`; 256 values x 16 phases x 257 lengths) is
  `cfg(target_arch = "aarch64")` and was not executed. The aarch64 lib and test-target checks passed
  under the 1.95.0 toolchain pinned then (`rust-toolchain.toml` now pins 1.97.0),
  `.github/workflows/ci.yml` has no aarch64 job, the
  ~700-byte rodata/text reduction is an analytical estimate (no two-revision release build), and the
  `[patch]` SemVer class is by analysis (no mechanical run). An aarch64 hardware or CI run remains
  the open execution evidence.
- **E-032 [patch] — investigated, no change (finding recorded)** AVX2
  `unpack_f8_to_f32` optimization: the E-031 benchmark flagged it at ~7 Gelem/s
  (gather-bound hypothesis). A branchless SIMD E4M3 decode (`significand × 2^scale`,
  flush-to-zero-safe, bit-exact incl. NaN payloads) was implemented and measured
  **~27% slower** (684 ns vs 537 ns, p < 0.05) than the `_mm256_i32gather_ps` LUT
  on this µarch — the hot 1 KB LUT gather out-throughputs the ~20-op ALU dependency
  chain. **Hypothesis refuted; the gather is retained** (no regression shipped).
  Kept the deliverable: an exhaustive direct `avx2::unpack_f8_to_f32` differential
  test (all 256 bytes vs the scalar kernel, valid on AVX-512 hosts too). **Do not
  re-attempt the branchless-compute path for f8→f32.**

<a id="eunomia-roadmap-num-complex-migration"></a>
## Roadmap — consumer num-complex → eunomia migration (tracked [arch])

**DEPENDENCY ORDERING (corrected 2026-06-29 — these are NOT independent passes).**
The consumers are chained through apollo: `num_complex::Complex` crosses crate
boundaries in public APIs, so a downstream repo cannot drop num-complex until its
upstream does. Verified order:

  apollo (FFT/spectral core, num_complex in its public `FftPlan` API)
     ↑ consumed by
  ritk-filter (calls `apollo_fft::FftPlan1D::*_complex_slice_inplace(&mut [Complex<f32>])`)
  CFDrs spectral paths
     ↑ consumed by
  (end consumers)

Concretely: **ritk is BLOCKED on apollo.** A trial swap of ritk-filter's 10
num_complex files to `eunomia::Complex` failed to build — `apollo_fft`'s
`forward_complex_slice_inplace(&mut [num_complex::Complex<f32>])` mandates the
num_complex type at the boundary. eunomia::Complex IS layout-identical
(`#[repr(C)]` `{re,im}`, Pod), so a `bytemuck::cast_slice_mut` boundary hack
*could* unblock ritk while keeping num-complex as a thin dep — but that does NOT
remove the dependency, so it is rejected. apollo must migrate its public API first.

- **E-011 [arch]** **apollo FIRST (critical-path upstream).** ~250 num_complex
  files across ~20 crates (apollo-{fft,czt,dctdst,dht,frft,fwht,gft,hilbert,mellin,
  ntt,nufft,qft,radon,sdft,sft,sht,stft,…}). apollo-fft *alone* is 121 num_complex
  files (larger than the whole gaia migration), and is consumed by ~19 sibling
  crates, so its public `Complex` API change ripples across the whole workspace —
  migrate **bottom-up, apollo-fft first** (it is what ritk needs). Each crate: add
  eunomia dep+patch; swap `num_complex::Complex` → `eunomia::Complex` (layout-
  identical, mechanical bulk — ref-ops + `powi` now exist in `types/complex/`);
  `num-complex` `bytemuck` feature → native Pod, `serde` → eunomia's `serde` feature; `mnemosyne`
  `features=["num-complex"]` → `["eunomia"]`; consolidate the GPU `ComplexPod
  {re,im}` structs onto `eunomia::Complex<f32>`. **FFT-numerics correctness is the
  gate** (differential vs reference transforms) — not rushable.
- **E-012 [arch]** **ritk** — UNBLOCKED ONLY AFTER apollo-fft (E-011). Then a clean
  ~10-file swap of ritk-filter (`num_complex::Complex` → `eunomia::Complex`,
  drop the `num-complex` bytemuck dep). Trial run done + reverted; resume post-apollo.
- **E-012b [arch]** **CFDrs.** Sparse-linalg blocker **largely RESOLVED** (earlier
  "needs a whole new sparse crate" was wrong). **Sparse linear algebra lives in
  leto-ops (CPU) + hephaestus (GPU), hermes-SIMD-backed** — the same layering as
  the dense linalg, NOT a separate crate. `leto-ops::application::sparse` already
  had `CsrMatrix` + `spmv` + `spmm`; **`CooMatrix` (assembly/triplet format with
  duplicate-accumulating `to_csr`) added** (leto `34e4019`), covering CFDrs's
  `CsrMatrix`/`CooMatrix` usage. Crucially, CFDrs implements its **own** iterative
  solvers (`cfd-math/linear_solver/{conjugate_gradient,bicgstab,gmres,
  preconditioners}`) over SpMV — no external sparse-solver dependency — and direct
  `nalgebra_sparse` usage is just 2 sites (`pattern::SparsityPattern`,
  `ops::serial`). Dense linalg is covered (leto-ops cholesky/LU/QR/SVD/eigen/schur).
  So CFDrs's remaining migration is the mechanical swap `nalgebra_sparse::
  {CsrMatrix,CooMatrix}` → `leto_ops::…::sparse::{CsrMatrix,CooMatrix}` + the 2
  minor nalgebra_sparse sites, then the dense `nalgebra`→leto + `num_complex`→
  eunomia passes (spectral via apollo, E-011). No remaining hard infra blocker.
  Possible leto follow-ups if CFDrs needs them: GPU SpMV in hephaestus; a
  `SparsityPattern` analogue; CSC format. (gaia-style per-site migration.)
- **E-013 [minor]** Replace direct `num-traits` deps with `eunomia::FloatElement`/
  `NumericElement`.
  - **CFDrs, gaia: NOT a clean swap** — their central `Scalar` traits bound
    `num_traits::Float` *alongside* `nalgebra::RealField` (gaia
    `src/domain/core/scalar.rs`: `Scalar: RealField + Float + ToPrimitive`),
    so the domain math leans on RealField. Dropping num_traits there is
    entangled with the larger **nalgebra → eunomia** migration, which first
    needs a eunomia `RealField`-equivalent (ordered/field/real surface beyond
    `FloatElement`). Promote as its own [arch] item — see E-017.

- **E-017 [arch]** nalgebra → eunomia (+ leto/hephaestus for the actual linear
  algebra).
  - **Geometry library: in leto, NOT eunomia.** eunomia is datatypes (scalars/
    complex + the RealField/ComplexField scalar field traits). The nalgebra
    *geometry* (Vector/Point/Unit/Quaternion/Isometry + cross/norm/transforms)
    is linear algebra → **leto** (CPU; `leto::geometry`) / **hephaestus** (GPU).
    (An earlier mis-placement in eunomia-gpu was reverted, PR #9.)
    (leto/hephaestus keep nalgebra as a *dev*-oracle — out of scope.)
- **E-007 [minor]** GPU layout vector types grow as a backend consumer appears;
  std140 UBO alignment stays a buffer-packing concern, not a type property.
