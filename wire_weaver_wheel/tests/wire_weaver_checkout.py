"""The wire_weaver checkout that this workspace's `Cargo.lock` pins, for the tests that use its fixtures.

The API crates (`tests/*_api`), the conformance corpus and the simulated devices (`hub_pytest_device`,
`thermo_devices`) live in the wire_weaver repository. Cargo's checkout of the locked commit holds them, so the
tests read the fixtures of the exact framework the wheel is built with. The devices are built from that checkout
with its own `Cargo.lock`, into this workspace's `target/`.
"""

import functools
import json
import subprocess
import sys
from pathlib import Path

WW_TOOLS = Path(__file__).resolve().parents[2]
TARGET_DIR = WW_TOOLS / "target" / "wire_weaver_fixtures"


@functools.cache
def root() -> Path:
    """The top of the locked wire_weaver checkout (fetches it when Cargo has not yet)."""
    out = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version", "1"],
        cwd=WW_TOOLS,
        check=True,
        capture_output=True,
        text=True,
    )
    for package in json.loads(out.stdout)["packages"]:
        if package["name"] == "wire_weaver" and (package.get("source") or "").startswith("git+"):
            return Path(package["manifest_path"]).parents[1]
    raise LookupError("Cargo.lock pins no wire_weaver from git")


def binary(package: str, name: str) -> Path:
    """Builds the binary `name` of the wire_weaver test crate `package` and returns its path."""
    subprocess.run(
        [
            "cargo",
            "build",
            "-q",
            "--locked",
            "--manifest-path",
            str(root() / "Cargo.toml"),
            "--target-dir",
            str(TARGET_DIR),
            "-p",
            package,
            "--bin",
            name,
        ],
        cwd=WW_TOOLS,
        check=True,
    )
    exe = TARGET_DIR / "debug" / (name + ".exe" if sys.platform == "win32" else name)
    assert exe.exists(), f"{exe} missing after cargo build"
    return exe
