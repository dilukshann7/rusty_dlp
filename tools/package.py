"""Package an ALREADY BUILT executable. This script does not invoke Cargo or publish anything."""
import argparse
import hashlib
import pathlib
import plistlib
import shutil
import stat
import tarfile
import zipfile


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", required=True)
    parser.add_argument("--platform", required=True, choices=["windows-x86_64", "linux-x86_64", "macos-x86_64", "macos-aarch64"])
    parser.add_argument("--root", type=pathlib.Path, default=pathlib.Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    root = args.root.resolve()
    windows = args.platform.startswith("windows")
    suffix = ".exe" if windows else ""
    executable = root / "target" / args.target / "release" / ("rusty-dlp" + suffix)
    if not executable.is_file():
        raise SystemExit(f"Already-built binary not found: {executable}")
    # Output is confined to dist; this utility never deletes or resets an existing directory.
    destination = root / "dist"
    destination.mkdir(exist_ok=True)
    version = next(line.split('"')[1] for line in (root / "Cargo.toml").read_text().splitlines() if line.startswith("version ="))
    name = f"rusty-dlp-{version}-{args.platform}"
    stage = destination / name
    stage.mkdir(exist_ok=False)
    if args.platform.startswith("macos"):
        contents = stage / "rusty_dlp.app" / "Contents"
        macos = contents / "MacOS"
        resources = contents / "Resources"
        macos.mkdir(parents=True)
        resources.mkdir()
        binary = macos / "rusty-dlp"
        shutil.copy2(executable, binary)
        plist = {"CFBundleName": "rusty_dlp", "CFBundleDisplayName": "rusty_dlp", "CFBundleExecutable": "rusty-dlp", "CFBundleIdentifier": "dev.rustydlp.desktop", "CFBundleVersion": version, "CFBundleShortVersionString": version, "CFBundlePackageType": "APPL", "NSHighResolutionCapable": True}
        (contents / "Info.plist").write_bytes(plistlib.dumps(plist))
    else:
        binary = stage / executable.name
        shutil.copy2(executable, binary)
    binary.chmod(binary.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    for document in ["README.md", "THIRD_PARTY.md", "LICENSE"]:
        shutil.copy2(root / document, stage / document)
    shutil.copy2(root / "assets" / "dependencies.json", stage / "dependencies.json")
    if args.platform.startswith("linux"):
        archive = destination / (name + ".tar.gz")
        def permissions(info):
            if info.isfile():
                info.mode = 0o755 if info.name.endswith("/rusty-dlp") else 0o644
            return info
        with tarfile.open(archive, "w:gz") as output:
            output.add(stage, arcname=name, filter=permissions)
    else:
        archive = destination / (name + ".zip")
        with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as output:
            for path in sorted(stage.rglob("*")):
                if path.is_file():
                    relative = path.relative_to(destination).as_posix()
                    output.write(path, relative)
                    info = output.getinfo(relative)
                    info.create_system = 3
                    info.external_attr = (0o100755 if path == binary else 0o100644) << 16
    with archive.open("rb") as source:
        digest = hashlib.file_digest(source, "sha256").hexdigest()
    archive.with_suffix(archive.suffix + ".sha256").write_text(f"{digest}  {archive.name}\n")
    print(f"Prepared {archive.name} (unsigned; not a platform smoke-test result)")


if __name__ == "__main__":
    main()
