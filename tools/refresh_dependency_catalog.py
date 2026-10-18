"""Maintainer utility: freeze upstream release metadata and hashes; never download binaries.

Run deliberately, review the diff, then ship the catalog with the application.
"""
import datetime
import json
import pathlib
import re
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[1]


def fetch(url):
    request = urllib.request.Request(url, headers={"User-Agent": "RustyDLP-catalog/0.1"})
    with urllib.request.urlopen(request, timeout=30) as response:
        return response.read()


def release(repo, tag="latest"):
    route = "latest" if tag == "latest" else "tags/" + tag
    return json.loads(fetch(f"https://api.github.com/repos/{repo}/releases/{route}"))


def asset(data, name, archive, outputs):
    item = next(a for a in data["assets"] if a["name"] == name)
    digest = item.get("digest", "")
    if not digest or not digest.startswith("sha256:"):
        raise ValueError(f"No SHA256 published for {name}")
    return {"url": item["browser_download_url"], "sha256": digest[7:], "archive": archive, "outputs": outputs}


def main():
    ytdlp = release("yt-dlp/yt-dlp")
    deno = release("denoland/deno")
    releases = json.loads(fetch("https://api.github.com/repos/yt-dlp/FFmpeg-Builds/releases?per_page=10"))
    pinned = next(r for r in releases if r["tag_name"].startswith("autobuild-"))
    ffmpeg = release("yt-dlp/FFmpeg-Builds", pinned["tag_name"])
    mac_page = fetch("https://ffmpeg.martin-riedl.de/").decode()
    targets = {
        "windows-x86_64": ("yt-dlp.exe", "x86_64-pc-windows-msvc", "win64", ".exe"),
        "linux-x86_64": ("yt-dlp_linux", "x86_64-unknown-linux-gnu", "linux64", ""),
        "macos-x86_64": ("yt-dlp_macos", "x86_64-apple-darwin", "amd64", ""),
        "macos-aarch64": ("yt-dlp_macos", "aarch64-apple-darwin", "arm64", ""),
    }
    entries = []
    for target, (yt_name, deno_target, ff_target, suffix) in targets.items():
        entries.append({"kind": "YtDlp", "target": target, "version": ytdlp["tag_name"], "source": "yt-dlp/yt-dlp", "assets": [asset(ytdlp, yt_name, "raw", ["yt-dlp" + suffix])]})
        entries.append({"kind": "Deno", "target": target, "version": deno["tag_name"], "source": "denoland/deno", "assets": [asset(deno, f"deno-{deno_target}.zip", "zip", ["deno" + suffix])]})
        if target.startswith("macos"):
            assets = []
            versions = []
            for name in ("ffmpeg", "ffprobe"):
                matches = re.findall(rf'href=[\"\']([^\"\']*/download/macos/{ff_target}/([^/]+)/{name}\.zip)[\"\']', mac_page)
                # Site uses relative links; accept either relative or absolute provider URLs.
                if not matches:
                    matches = re.findall(rf'href=[\"\'](/download/macos/{ff_target}/([^/]+)/{name}\.zip)[\"\']', mac_page)
                path, version = matches[0]
                url = path if path.startswith("https://") else "https://ffmpeg.martin-riedl.de" + path
                checksum = fetch(url + ".sha256").decode().split()[0]
                if not re.fullmatch(r"[0-9a-fA-F]{64}", checksum):
                    raise ValueError("Invalid provider checksum")
                assets.append({"url": url, "sha256": checksum.lower(), "archive": "zip", "outputs": [name]})
                versions.append(version)
            if versions[0] != versions[1]:
                raise ValueError("FFmpeg and ffprobe versions differ")
            entries.append({"kind": "Ffmpeg", "target": target, "version": versions[0], "source": "Martin Riedl FFmpeg builds", "assets": assets})
        else:
            extension = "zip" if suffix else "tar.xz"
            name = next(a["name"] for a in ffmpeg["assets"] if a["name"].endswith(f"-{ff_target}-gpl.{extension}"))
            entries.append({"kind": "Ffmpeg", "target": target, "version": ffmpeg["tag_name"], "source": "yt-dlp/FFmpeg-Builds", "assets": [asset(ffmpeg, name, extension, ["ffmpeg" + suffix, "ffprobe" + suffix])]})
    result = {"schema_version": 1, "generated_at": datetime.date.today().isoformat(), "entries": entries}
    destination = ROOT / "assets" / "dependencies.json"
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(f"Frozen {len(entries)} dependency entries across {len(targets)} targets.")


if __name__ == "__main__":
    main()
