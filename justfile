# `just jobs=N <recipe>` caps build and test parallelism for that run; unset leaves CARGO_BUILD_JOBS and
# NEXTEST_TEST_THREADS as inherited, or at cargo's default (the CPU count, a number maturin can read too)
jobs := ""
export CARGO_BUILD_JOBS := if jobs != "" { jobs } else { env_var_or_default('CARGO_BUILD_JOBS', num_cpus()) }
export NEXTEST_TEST_THREADS := if jobs != "" { jobs } else { env_var_or_default('NEXTEST_TEST_THREADS', 'num-cpus') }

# List all available targets
default:
    @just --list

# The tests. `just test`: all of them (test-all). `just test --changed`: the Rust tests of the crates changed against
# main and the crates that depend on them (through tpm when installed, otherwise all), then the Python tests
test *flags:
    {{ if flags == "--changed" { "changedtests run" } else { just_executable() + " test-all " + flags } }}

# Every test: nextest, the doctests, the Python tests, the builds of `just check`
test-all: check
    cargo nextest run --workspace --no-fail-fast
    # nextest does not run doctests
    cargo test --workspace --doc -q
    just test-py

# Build the Python module (wire_weaver_wheel) into its own venv and run its tests, the conformance corpus too (TEST-23)
[working-directory('wire_weaver_wheel')]
test-py:
    uv run --group dev pytest -q

# The conformance corpus through the Python module only (TEST-23)
[working-directory('wire_weaver_wheel')]
test-conformance-py:
    uv run --group dev pytest -q tests/test_conformance.py

# What the tests don't build: the release wheel, imported from a fresh venv
check: wheel
    uv run --no-project --isolated --with "$(ls target/wheels/wire_weaver-*.whl | head -1)" \
        python -c "import wire_weaver, wire_weaver.logring"

# Build the release wheel into target/wheels with maturin
wheel:
    rm -rf target/wheels
    cd wire_weaver_wheel && uv run --no-project --with "maturin>=1.11,<2.0" maturin build --release --locked --out ../target/wheels

# Format, manifests, spelling, clippy and rustdoc as CI runs them, warnings are errors
lint:
    cargo fmt --all --check
    cargo sort -w -g --check
    taplo fmt --check
    typos
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
    cargo machete

# Python console with `ww` imported and the device connected as `dev` (rebuilds the module if needed), `just py --help`
[working-directory('wire_weaver_wheel')]
py *args:
    WW_CWD="{{ invocation_directory() }}" uv run python -i scripts/console.py {{ args }}

# Build ww and the release wheel. ww goes on PATH through tpm's kept-releases stack when tpm is installed, else
# cargo install; the wheel stays in target/wheels for `uv pip install` or `uv add` in a project
install: wheel
    cargo build --profile deploy --locked -p ww
    if command -v tpm >/dev/null; then tpm install target/deploy/ww; else cargo install --locked --path ww; fi

# Bring this PC up to main: refuse on local changes, pull --ff-only, install, check ww --version against main
deploy:
    #!/usr/bin/env bash
    set -euo pipefail
    dirty=$(git status --porcelain)
    if [ -n "$dirty" ]; then echo "local changes, not deploying:"; echo "$dirty"; exit 1; fi
    git pull --ff-only
    just install
    ww --version | grep -q "($(git rev-parse --short HEAD)" || { echo "ww --version does not show $(git rev-parse --short HEAD):"; ww --version; exit 1; }
    ww --version

# Build ww from a fresh clone of HEAD with nothing but this repository around (what `cargo install --git` sees)
check-clean-clone:
    rm -rf /tmp/ww-clean-clone && git clone -q "$(git rev-parse --show-toplevel)" /tmp/ww-clean-clone
    cd /tmp/ww-clean-clone && cargo build --locked -p ww
    rm -rf /tmp/ww-clean-clone
