"""Thin CLI wrapper that delegates to the downloaded rninja binary."""

import hashlib
import os
import platform
import subprocess
import sys
import tarfile
import urllib.request
from pathlib import Path

try:
    from importlib.metadata import version as get_version
except ImportError:
    from importlib_metadata import version as get_version  # type: ignore[no-redef]

GITHUB_OWNER = "neul-labs"
REPO = "rninja"
VERSION = get_version("rninja")

ARCH_MAP = {
    "x86_64": "x86_64",
    "AMD64": "x86_64",
    "arm64": "aarch64",
    "aarch64": "aarch64",
}


def _get_binary_dir() -> Path:
    """Return the directory where binaries are installed."""
    return Path(__file__).parent / "bin"


def _get_download_url() -> str:
    """Determine the correct tarball URL for this platform."""
    system = platform.system().lower()
    machine = platform.machine()
    arch = ARCH_MAP.get(machine, machine)

    if system == "darwin":
        platform_name = "apple-darwin"
    elif system == "linux":
        platform_name = "unknown-linux-gnu"
    else:
        raise RuntimeError(f"Unsupported platform: {system}")

    return (
        f"https://github.com/{GITHUB_OWNER}/{REPO}/releases/download/"
        f"v{VERSION}/rninja-{VERSION}-{arch}-{platform_name}.tar.gz"
    )


def _download_and_extract(dest_dir: Path) -> None:
    """Download the prebuilt binary tarball and extract to dest_dir."""
    url = _get_download_url()
    print(f"Downloading rninja {VERSION} from {url}")

    dest_dir.mkdir(parents=True, exist_ok=True)
    tarball_path = dest_dir / "rninja.tar.gz"
    try:
        urllib.request.urlretrieve(url, tarball_path)
    except Exception as e:
        raise RuntimeError(f"Failed to download rninja: {e}") from e

    print("Extracting binaries...")
    with tarfile.open(tarball_path, "r:gz") as tf:
        tf.extractall(path=dest_dir)

    tarball_path.unlink()

    # Make binaries executable on Unix
    if platform.system() != "Windows":
        for binary in ["rninja", "rninja-cached", "rninja-daemon"]:
            binary_path = dest_dir / binary
            if binary_path.exists():
                binary_path.chmod(0o755)


def _ensure_binary(name: str) -> str:
    """Return the path to a binary, downloading if necessary."""
    bin_dir = _get_binary_dir()

    if platform.system() == "Windows":
        path = bin_dir / f"{name}.exe"
        if path.exists():
            return str(path)
    else:
        path = bin_dir / name
        if path.exists():
            return str(path)

    # Binary not found — download it
    if os.environ.get("RNINJA_SKIP_DOWNLOAD"):
        raise RuntimeError(
            f"Binary '{name}' not found in {bin_dir} and RNINJA_SKIP_DOWNLOAD is set."
        )

    _download_and_extract(bin_dir)

    # Check again after download
    if platform.system() == "Windows":
        path = bin_dir / f"{name}.exe"
    else:
        path = bin_dir / name

    if path.exists():
        return str(path)

    raise RuntimeError(
        f"Binary '{name}' not found in {bin_dir} after download. "
        "Please reinstall the package."
    )


def _run(name: str) -> None:
    """Run the named binary with forwarded arguments."""
    binary = _ensure_binary(name)
    result = subprocess.run([binary] + sys.argv[1:])
    sys.exit(result.returncode)


def main() -> None:
    """Run rninja."""
    _run("rninja")


def cached_main() -> None:
    """Run rninja-cached."""
    _run("rninja-cached")


def daemon_main() -> None:
    """Run rninja-daemon."""
    _run("rninja-daemon")


if __name__ == "__main__":
    main()
