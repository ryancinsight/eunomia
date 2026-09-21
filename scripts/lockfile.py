#!/usr/bin/env python3
"""Regenerate or check `Cargo.lock` against the source set CI actually resolves.

# The trap this exists for

This repository is normally worked on inside the Atlas stack, whose
`.cargo/config.toml` carries a `[patch]` overlay redirecting every first-party
dependency to a local working tree. Cargo discovers that config by walking up
from the *current directory*, so any `cargo` command run from inside the stack
picks it up -- including anything that rewrites the lock.

A lock written with the overlay active has every `source = "git+..."` line
**stripped**, because those dependencies resolved to local paths rather than to
git. Committing it replaces all 87 git sources with nothing. CI has no overlay,
so it re-resolves, and every `--locked` job fails with

    error: cannot update the lock file ... because --locked was passed

which names neither the cause nor the fix. That message is also what a merely
*stale* lock produces -- one pinning first-party revisions whose versions no
longer satisfy the manifests -- so the two failures are indistinguishable from
the log alone (KW-CI-087).

This is not limited to deliberate regeneration. *Any* cargo invocation that
updates the lock while the overlay is active flattens it -- an ordinary
`cargo check` inside the stack is enough, which is how it happens in practice:
nobody sets out to rewrite the lock. Treat a modified `Cargo.lock` after routine
work as suspect and run `--check` before staging it.

Both are fixed the same way: regenerate from outside the overlay. This script
does that by running cargo from a temporary directory that is not underneath the
stack root, which is the whole mechanism -- there is no flag that disables config
discovery.

# Usage

    scripts/lockfile.py --check        # verify the committed lock, offline
    scripts/lockfile.py --regenerate   # rewrite it correctly (needs network)
"""

from __future__ import annotations

import argparse
import os
import re
import json
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

REPOSITORY = Path(__file__).resolve().parent.parent
LOCKFILE = REPOSITORY / "Cargo.lock"
MANIFEST = REPOSITORY / "Cargo.toml"

# Any first-party dependency resolves through one of these. A lock with none of
# them has been flattened by the overlay.
FIRST_PARTY_SOURCE = re.compile(r'^source = "git\+https://github\.com/ryancinsight/', re.M)
FIRST_PARTY_GIT = "git+https://github.com/ryancinsight/"

# One `[[package]]` source line for a first-party repository, captured whole so
# the revision and the URL spelling both take part in the identity comparison.
FIRST_PARTY_PACKAGE_SOURCE = re.compile(
    r'^source = "(git\+https://github\.com/ryancinsight/[^"]+)"', re.M
)

# A member records how many first-party providers its graph currently resolves
# through more than one source, as a single integer beside its lock. The file is
# a ratchet: the check fails when the measured excess exceeds it, and the number
# is lowered as the dependency-ordered unpin sweep closes each fork. Absent, the
# check reports and does not fail -- a member that has never measured its graph
# is not failed by a guard it has not adopted.
PROVIDER_IDENTITY_BASELINE_NAME = ".provider-identity-baseline"


def shared_target_dir(manifest: Path) -> Path | None:
    """The `target-dir` a stack config above `manifest` declares, if any.

    The neutral working directory below disables cargo's config discovery on
    purpose — that is what excludes the `[patch]` overlay. It excludes the rest
    of the config with it, and `target-dir` is in the rest: cargo then falls
    back to `<manifest dir>/target` and writes a repo-local cache inside the
    member. That is the `target_forks` debt class the conformance ratchet
    counts, produced by the tooling that measures it.

    Walking up from the manifest rather than from the cwd is the correction:
    the overlay is excluded because of *where cargo runs*, and the target
    directory is restored because of *what is being built*.
    """
    for directory in [manifest.parent, *manifest.parent.parents]:
        config = directory / ".cargo" / "config.toml"
        if not config.is_file():
            continue
        for line in config.read_text(encoding="utf-8").splitlines():
            stripped = line.split("#", 1)[0].strip()
            if not stripped.startswith("target-dir"):
                continue
            _, _, value = stripped.partition("=")
            value = value.strip().strip('"').strip("'")
            if value:
                return (directory / value).resolve()
    return None



def run_outside_the_overlay(
    arguments: list[str], manifest: Path | None = None
) -> subprocess.CompletedProcess[str]:
    """Run cargo with a working directory outside the stack root.

    Cargo resolves `.cargo/config.toml` by walking up from the working
    directory, never from `--manifest-path`, so this is what excludes the
    overlay. Running from the repository itself would silently include it.

    `manifest` defaults to `MANIFEST` as it stands at call time -- `main`
    reassigns it under `--manifest-path`, which a definition-time default would
    never see -- and the consumer lock sweep passes a member's, so one
    overlay-free runner serves the stack.

    Excluding the overlay excludes the whole config, `target-dir` included, so
    the shared cache is restored explicitly from the stack config above the
    manifest ([`shared_target_dir`]). Without it every check writes a
    repo-local `target/` into the member it inspects, which is the
    `target_forks` class the conformance ratchet counts. An inherited
    `CARGO_TARGET_DIR` wins, so CI -- where there is no stack config and each
    job is isolated anyway -- is unaffected.
    """
    if manifest is None:
        manifest = MANIFEST
    environment = dict(os.environ)
    if "CARGO_TARGET_DIR" not in environment:
        shared = shared_target_dir(manifest)
        if shared is not None:
            environment["CARGO_TARGET_DIR"] = str(shared)
    with tempfile.TemporaryDirectory() as neutral_directory:
        return subprocess.run(
            ["cargo", *arguments, "--manifest-path", str(manifest)],
            cwd=neutral_directory,
            env=environment,
            capture_output=True,
            # `text=True` alone decodes with the locale codepage. Cargo emits
            # UTF-8, so on a Windows console (cp1252) subprocess's reader thread
            # dies on the first byte it cannot map and the captured stream is
            # lost. The verdict survives -- it comes from `returncode` -- but the
            # message explaining a failure does not, which is the one moment it
            # is needed.
            encoding="utf-8",
            errors="replace",
            check=False,
        )


def declared_first_party_dependencies() -> int | None:
    """Count dependencies the workspace declares on first-party git sources.

    `cargo metadata --no-deps` reads the manifests alone, so a flattened or
    missing lock cannot influence it. A workspace declaring none (the atlas
    tool workspaces) legitimately has a lock with no first-party sources, and
    the flattening diagnosis must not fire on it. `None` when cargo cannot
    read the workspace; the --locked resolution below reports that.
    """
    completed = run_outside_the_overlay(["metadata", "--no-deps", "--format-version", "1"])
    if completed.returncode != 0:
        return None
    packages = json.loads(completed.stdout)["packages"]
    return sum(
        1
        for package in packages
        for dependency in package["dependencies"]
        if (dependency.get("source") or "").startswith(FIRST_PARTY_GIT)
    )


def check() -> int:
    if not LOCKFILE.is_file():
        print(f"error: {LOCKFILE} does not exist", file=sys.stderr)
        return 1

    sources = len(FIRST_PARTY_SOURCE.findall(LOCKFILE.read_text(encoding="utf-8")))
    declared = declared_first_party_dependencies()
    if sources == 0 and declared != 0:
        print(
            "error: Cargo.lock contains no first-party git sources.\n"
            "\n"
            "It was regenerated with the Atlas stack overlay active, which\n"
            "resolves those dependencies to local paths and drops their git\n"
            "sources. CI has no overlay and will fail every --locked job.\n"
            "\n"
            "Fix: scripts/lockfile.py --regenerate",
            file=sys.stderr,
        )
        return 1

    completed = run_outside_the_overlay(
        ["metadata", "--locked", "--format-version", "1", "--all-features"]
    )
    if completed.returncode != 0:
        print(
            f"error: the committed Cargo.lock does not resolve under --locked "
            f"({sources} first-party git sources present, so it is stale rather "
            f"than flattened).\n"
            f"\n"
            f"The pinned first-party revisions no longer satisfy the manifests'\n"
            f"version requirements, so cargo must re-resolve and --locked\n"
            f"refuses. This is what blocks the benchmark baseline alignment.\n"
            f"\n"
            f"Fix: scripts/lockfile.py --regenerate\n"
            f"\n"
            f"cargo said:\n{completed.stderr.strip()}",
            file=sys.stderr,
        )
        return 1

    if declared == 0:
        print("Cargo.lock resolves under --locked; no first-party dependencies declared.")
    else:
        print(f"Cargo.lock resolves under --locked; {sources} first-party git sources.")
    return check_provider_identity()


def provider_identities(lock_text: str) -> dict[str, set[str]]:
    """Map each first-party repository to the distinct sources it resolves through.

    The key drops the `.git` suffix and the URL case, so the two spellings of one
    repository count as the fork they are rather than as two providers.
    """
    identities: dict[str, set[str]] = {}
    for source in FIRST_PARTY_PACKAGE_SOURCE.findall(lock_text):
        base = source.split("#", 1)[0]
        repository = base.split("?", 1)[0].removesuffix(".git").lower()
        identities.setdefault(repository, set()).add(base)
    return identities


def provider_identity_baseline() -> int | None:
    """Read the member's committed excess-source bound, or `None` when unset."""
    path = LOCKFILE.parent / PROVIDER_IDENTITY_BASELINE_NAME
    if not path.is_file():
        return None
    try:
        return int(path.read_text(encoding="utf-8").split("#", 1)[0].strip())
    except ValueError:
        print(
            f"error: {path} must contain a single integer bound.",
            file=sys.stderr,
        )
        return -1


def check_provider_identity() -> int:
    """Bound the first-party providers that resolve through more than one source.

    A provider reached by two sources is compiled twice, so its public types stop
    matching across the boundary between the consumers that took different
    routes. Nothing reports this: the build is green, the lock resolves under
    `--locked`, and the mismatch surfaces only where the two halves meet.

    It is also the reason a merged co-evolution pin cannot be dropped on its own.
    Removing one while a transitive first-party consumer still pins the older
    revision adds a source rather than removing one -- measured on apollo, where
    dropping four merged pins raised the excess from four to eight because
    `leto-ops` still pinned hermes at `5a399ee`.

    The bound is a ratchet, not a zero: these forks exist today and a check that
    fails on arrival gets disabled rather than fixed.
    """
    identities = provider_identities(LOCKFILE.read_text(encoding="utf-8"))
    forked = {name: sources for name, sources in identities.items() if len(sources) > 1}
    excess = sum(len(sources) - 1 for sources in forked.values())

    for name, sources in sorted(forked.items()):
        print(f"  {name.rsplit('/', 1)[-1]} resolves through {len(sources)} sources:")
        for source in sorted(sources):
            print(f"    {source.split('ryancinsight/', 1)[-1]}")

    baseline = provider_identity_baseline()
    if baseline is None:
        print(
            f"Provider identity: {excess} source(s) in excess of one per "
            f"repository; unbounded (no {PROVIDER_IDENTITY_BASELINE_NAME})."
        )
        return 0
    if baseline < 0:
        return 1

    if excess > baseline:
        print(
            f"error: {excess} first-party provider sources in excess of one per\n"
            f"repository; the committed bound is {baseline}.\n"
            f"\n"
            f"Each extra source is a second copy of that provider in the graph, so\n"
            f"its public types no longer match across the boundary between the\n"
            f"consumers that reached it by different routes.\n"
            f"\n"
            f"A merged co-evolution pin cannot be removed until every transitive\n"
            f"first-party consumer of that provider has advanced too; unpinning\n"
            f"ahead of them adds a source rather than removing one.\n"
            f"\n"
            f"Fix: advance the consumers first, or restore the pin. Lower\n"
            f"{PROVIDER_IDENTITY_BASELINE_NAME} only when a fork actually closes.",
            file=sys.stderr,
        )
        return 1

    if excess < baseline:
        print(
            f"Provider identity: {excess} source(s) in excess of one per repository, "
            f"below the committed bound of {baseline} -- lower it to {excess}."
        )
        return 0

    print(
        f"Provider identity: {excess} source(s) in excess of one per repository "
        f"(bound {baseline})."
    )
    return 0


def indexed_first_party_dependencies() -> int:
    """Count git providers in indexed Cargo dependency tables, without Cargo."""
    indexed = subprocess.run(
        ["git", "ls-files", "--cached", "-z", "--", "*Cargo.toml"],
        cwd=REPOSITORY, capture_output=True, text=True, check=True, timeout=30,
    )
    count = 0
    for path in indexed.stdout.split("\0"):
        if not path or Path(path).name != "Cargo.toml":
            continue
        blob = subprocess.run(
            ["git", "show", f":{path}"], cwd=REPOSITORY,
            capture_output=True, text=True, check=True, timeout=30,
        )
        pending = [tomllib.loads(blob.stdout)]
        while pending:
            table = pending.pop()
            for name, value in table.items():
                if not isinstance(value, dict):
                    continue
                if name in {"dependencies", "dev-dependencies", "build-dependencies"}:
                    count += sum(
                        isinstance(dependency, dict)
                        and str(dependency.get("git", "")).startswith(FIRST_PARTY_GIT.removeprefix("git+"))
                        for dependency in value.values()
                    )
                else:
                    pending.append(value)
    return count


def check_staged() -> int:
    """Reject a staged lock missing declared first-party git sources."""
    staged = subprocess.run(
        ["git", "diff", "--cached", "--name-only", "--", "Cargo.lock"],
        cwd=REPOSITORY, capture_output=True, text=True, check=True, timeout=30,
    )
    if not staged.stdout.strip():
        return 0
    blob = subprocess.run(
        ["git", "show", ":Cargo.lock"], cwd=REPOSITORY,
        capture_output=True, text=True, check=True, timeout=30,
    )
    if FIRST_PARTY_SOURCE.search(blob.stdout):
        return 0
    declared = indexed_first_party_dependencies()
    if declared == 0:
        return 0
    print("error: staged Cargo.lock omits declared first-party git sources", file=sys.stderr)
    return 1


def regenerate() -> int:
    completed = run_outside_the_overlay(["generate-lockfile"])
    if completed.returncode != 0:
        print(f"error: regeneration failed:\n{completed.stderr.strip()}", file=sys.stderr)
        return 1
    print("Cargo.lock regenerated outside the overlay.")
    return check()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="verify the committed lock")
    mode.add_argument("--regenerate", action="store_true", help="rewrite the lock correctly")
    mode.add_argument("--check-staged", action="store_true", help="check the staged lock for pre-commit")
    parser.add_argument("--manifest-path", type=Path, default=None,
                        help="path to Cargo.toml (overrides auto-detection from __file__)")
    arguments = parser.parse_args()
    if arguments.manifest_path is not None:
        global REPOSITORY, LOCKFILE, MANIFEST
        MANIFEST = arguments.manifest_path.resolve()
        REPOSITORY = MANIFEST.parent
        LOCKFILE = REPOSITORY / "Cargo.lock"
    if arguments.check_staged:
        return check_staged()
    return regenerate() if arguments.regenerate else check()


if __name__ == "__main__":
    raise SystemExit(main())
