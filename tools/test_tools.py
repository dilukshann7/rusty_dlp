"""Packaging fixtures only: no Cargo, binary execution, downloads or release publication."""
import hashlib
import pathlib
import subprocess
import sys
import tarfile
import tempfile
import unittest
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]


class PackageTests(unittest.TestCase):
    def fixture(self, root, target, platform):
        for name in ["Cargo.toml", "README.md", "THIRD_PARTY.md", "LICENSE"]:
            (root / name).write_bytes((ROOT / name).read_bytes())
        (root / "assets").mkdir()
        (root / "assets" / "dependencies.json").write_bytes((ROOT / "assets" / "dependencies.json").read_bytes())
        binary = root / "target" / target / "release" / ("rusty-dlp.exe" if platform.startswith("windows") else "rusty-dlp")
        binary.parent.mkdir(parents=True)
        binary.write_bytes(b"offline packaging fixture, not executable")

    def package(self, root, target, platform):
        return subprocess.run([sys.executable, str(ROOT / "tools" / "package.py"), "--root", str(root), "--target", target, "--platform", platform], capture_output=True, text=True)

    def test_archives_include_binary_docs_and_checksum_for_all_targets(self):
        for target, platform in [("x86_64-pc-windows-msvc", "windows-x86_64"), ("x86_64-unknown-linux-gnu", "linux-x86_64"), ("x86_64-apple-darwin", "macos-x86_64"), ("aarch64-apple-darwin", "macos-aarch64")]:
            with self.subTest(platform=platform), tempfile.TemporaryDirectory() as folder:
                root = pathlib.Path(folder)
                self.fixture(root, target, platform)
                result = self.package(root, target, platform)
                self.assertEqual(result.returncode, 0, result.stderr)
                archives = list((root / "dist").glob("*.zip")) + list((root / "dist").glob("*.tar.gz"))
                self.assertEqual(len(archives), 1)
                archive = archives[0]
                digest = hashlib.sha256(archive.read_bytes()).hexdigest()
                self.assertEqual(archive.with_suffix(archive.suffix + ".sha256").read_text().split()[0], digest)
                if archive.suffix == ".zip":
                    with zipfile.ZipFile(archive) as package:
                        names = package.namelist()
                        if platform.startswith("macos"):
                            self.assertTrue(any(name.endswith(".app/Contents/MacOS/rusty-dlp") for name in names))
                            self.assertTrue(any(name.endswith(".app/Contents/Info.plist") for name in names))
                            binary_info = next(info for info in package.infolist() if info.filename.endswith(".app/Contents/MacOS/rusty-dlp"))
                            self.assertTrue((binary_info.external_attr >> 16) & 0o111)
                else:
                    with tarfile.open(archive) as package:
                        names = package.getnames()
                        binary = next(item for item in package.getmembers() if item.name.endswith("/rusty-dlp"))
                        self.assertTrue(binary.mode & 0o111)
                self.assertTrue(any(name.endswith("/README.md") for name in names))
                self.assertTrue(any(name.endswith("/THIRD_PARTY.md") for name in names))

    def test_missing_binary_does_not_start_a_build(self):
        with tempfile.TemporaryDirectory() as folder:
            root = pathlib.Path(folder)
            result = self.package(root, "x86_64-pc-windows-msvc", "windows-x86_64")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Already-built binary not found", result.stderr)
            self.assertFalse((root / "target").exists())

    def test_existing_package_directory_is_not_replaced(self):
        with tempfile.TemporaryDirectory() as folder:
            root = pathlib.Path(folder)
            self.fixture(root, "x86_64-pc-windows-msvc", "windows-x86_64")
            self.assertEqual(self.package(root, "x86_64-pc-windows-msvc", "windows-x86_64").returncode, 0)
            sentinel = next((root / "dist").glob("rusty-dlp-*")) / "keep.txt"
            sentinel.write_text("keep")
            self.assertNotEqual(self.package(root, "x86_64-pc-windows-msvc", "windows-x86_64").returncode, 0)
            self.assertEqual(sentinel.read_text(), "keep")


if __name__ == "__main__":
    unittest.main()
