"""Build a per-user Windows installer from an already-built release executable."""
import argparse
import hashlib
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--compiler", type=Path)
    parser.add_argument("--binary", type=Path)
    args = parser.parse_args()
    root = args.root.resolve()
    compiler = args.compiler or shutil.which("ISCC.exe")
    if not compiler:
        candidate = Path("C:/Program Files (x86)/Inno Setup 6/ISCC.exe")
        compiler = candidate if candidate.is_file() else None
    if not compiler or not Path(compiler).is_file():
        raise SystemExit("Inno Setup compiler missing. Supply --compiler path/to/ISCC.exe.")
    binary = (args.binary or root / "target/x86_64-pc-windows-msvc/release/rusty-dlp.exe").resolve()
    if not binary.is_file():
        raise SystemExit(f"Already-built binary not found: {binary}")
    version = next(line.split('"')[1] for line in (root / "Cargo.toml").read_text().splitlines() if line.startswith("version ="))
    output = root / "dist" / f"rusty_dlp-{version}-windows-x64-setup.exe"
    if output.exists():
        raise SystemExit(f"Installer already exists; preserve or move it first: {output}")
    subprocess.run([str(compiler), "/Qp", f"/DAppVersion={version}", f"/DSourceRoot={root}", f"/DBinaryPath={binary}", str(root / "installer/windows.iss")], check=True)
    digest = hashlib.sha256(output.read_bytes()).hexdigest()
    output.with_suffix(".exe.sha256").write_text(f"{digest}  {output.name}\n")
    print(f"Prepared {output.name} (unsigned)")


if __name__ == "__main__":
    main()
