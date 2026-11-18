"""Check publishable crates against source-verified crates.io release tags."""

import hashlib
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tomllib
import urllib.error
import urllib.request


REQUIRED_BASELINES = {"thymeleaf"}
USER_AGENT = "thymeleaf-rust-semver-checks (https://github.com/easy-4-rust/thymeleaf-rust)"


def published_release(name: str) -> dict | None:
    request = urllib.request.Request(
        f"https://crates.io/api/v1/crates/{name}",
        headers={"User-Agent": USER_AGENT},
    )
    try:
        with urllib.request.urlopen(request, timeout=20) as response:
            data = json.load(response)
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return None
        raise

    version = data["crate"]["max_version"]
    release = next(
        (
            item
            for item in data["versions"]
            if item["num"] == version and not item["yanked"]
        ),
        None,
    )
    if not version or release is None:
        raise ValueError(f"{name}: crates.io returned no usable latest release")
    return release


def source_files(archive: tarfile.TarFile, prefix: str) -> dict[str, bytes]:
    files = {}
    for member in archive.getmembers():
        if not member.isfile() or not member.name.startswith(prefix):
            continue
        relative = member.name.removeprefix(prefix)
        if (relative.startswith("src/") and relative.endswith(".rs")) or relative == "build.rs":
            files[relative] = archive.extractfile(member).read()
    return files


def verify_release_tag(package: dict, release: dict) -> str:
    name, version = package["name"], release["num"]
    tag = f"v{version}"
    subprocess.run(
        ["git", "rev-parse", "--verify", f"refs/tags/{tag}^{{commit}}"],
        check=True,
        stdout=subprocess.DEVNULL,
    )
    archive_request = urllib.request.Request(
        f"https://crates.io{release['dl_path']}",
        headers={"User-Agent": USER_AGENT},
    )
    with urllib.request.urlopen(archive_request, timeout=30) as response:
        archive_bytes = response.read()
    if hashlib.sha256(archive_bytes).hexdigest() != release["checksum"]:
        raise ValueError(f"{name} {version}: published archive checksum mismatch")

    crate_path = Path(package["manifest_path"]).parent.relative_to(Path.cwd())
    tag_archive = subprocess.run(
        ["git", "archive", "--format=tar", tag, crate_path.as_posix()],
        check=True,
        stdout=subprocess.PIPE,
    ).stdout
    with tarfile.open(fileobj=io.BytesIO(archive_bytes), mode="r:gz") as published, tarfile.open(
        fileobj=io.BytesIO(tag_archive), mode="r:"
    ) as tagged:
        published_prefix = f"{name}-{version}/"
        tag_prefix = f"{crate_path.as_posix()}/"
        published_manifest = published.extractfile(published_prefix + "Cargo.toml.orig")
        tagged_manifest = tagged.extractfile(tag_prefix + "Cargo.toml")
        original_manifest = tagged_manifest.read()
        if published_manifest.read() != original_manifest:
            raise ValueError(f"{name} {version}: release tag manifest differs from crates.io")
        tag_version = tomllib.loads(original_manifest.decode())["package"]["version"]
        if isinstance(tag_version, dict) and tag_version.get("workspace"):
            root_manifest = subprocess.run(
                ["git", "show", f"{tag}:Cargo.toml"],
                check=True,
                stdout=subprocess.PIPE,
            ).stdout
            tag_version = tomllib.loads(root_manifest.decode())["workspace"]["package"]["version"]
        if tag_version != version:
            raise ValueError(f"{name} {version}: release tag resolves to version {tag_version}")
        published_sources = source_files(published, published_prefix)
        tagged_sources = source_files(tagged, tag_prefix)
        if not published_sources or published_sources != tagged_sources:
            raise ValueError(f"{name} {version}: release tag Rust source differs from crates.io")
    print(
        f"::notice::{name} {version}: release tag matches {len(published_sources)} published Rust source files",
        flush=True,
    )
    return tag


def main() -> None:
    metadata = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    )
    packages = json.loads(metadata.stdout)["packages"]
    publishable = sorted(
        package["name"]
        for package in packages
        if package["publish"] is None or "crates-io" in package["publish"]
    )
    if not REQUIRED_BASELINES.issubset(publishable):
        raise ValueError("the published thymeleaf crate is missing from workspace metadata")

    for name in publishable:
        release = published_release(name)
        if release is None:
            if name in REQUIRED_BASELINES:
                raise ValueError(f"{name}: published crates.io baseline unexpectedly missing")
            print(f"::notice::{name} has no crates.io release yet", flush=True)
            continue
        tag = verify_release_tag(
            next(package for package in packages if package["name"] == name), release
        )
        # thymeleaf 0.1.1 omitted DTD assets from its archive; the verified tag
        # has identical API source and can build rustdoc for the real release.
        print(f"::notice::Checking {name} against crates.io {release['num']}", flush=True)
        subprocess.run(
            [
                "cargo",
                "semver-checks",
                "check-release",
                "--package",
                name,
                "--baseline-rev",
                tag,
            ],
            check=True,
        )


if __name__ == "__main__":
    main()
