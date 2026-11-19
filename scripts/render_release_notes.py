#!/usr/bin/env python3
"""Render the current version's bilingual GitHub Release notes."""

import argparse
import os
from pathlib import Path
import re
import tomllib


ROOT = Path(__file__).resolve().parent.parent


def highlights(changelog, version):
    heading = re.compile(r"^## \[([^\]]+)\](?:[ \t].*)?$", re.MULTILINE)
    versions = list(heading.finditer(changelog))
    for index, match in enumerate(versions):
        if match[1] == version:
            end = versions[index + 1].start() if index + 1 < len(versions) else len(changelog)
            section = changelog[match.end():end].strip()
            break
    else:
        raise ValueError(f"CHANGELOG.md is missing [{version}]")

    parts = re.split(r"^### (简体中文|English)[ \t]*$", section, flags=re.MULTILINE)
    if len(parts) != 5 or parts[0].strip() or set(parts[1::2]) != {"简体中文", "English"}:
        raise ValueError(f"CHANGELOG.md [{version}] must contain ### 简体中文 and ### English")
    notes = {language: text.strip() for language, text in zip(parts[1::2], parts[2::2])}
    if not all(notes.values()):
        raise ValueError(f"CHANGELOG.md [{version}] needs changes in both languages")
    return notes


def render_section(notes, version, repository, chinese):
    tag = f"v{version}"
    download = f"{repository}/releases/download/{tag}"
    language, install, changes, downloads, verify = (
        ("简体中文", "安装", "版本变更", "下载", "校验 SHA-256") if chinese
        else ("English", "Install", "Changes", "Downloads", "Verify SHA-256")
    )
    asset_names = (f"thymeleaf-{version}.crate", f"thymeleaf-{version}-api-docs.tar.gz", "SHA256SUMS")
    descriptions = (
        ("与 crates.io 一致的 Cargo 源码包",
         "启用全部特性构建的离线 API 文档。解压后打开 \x60doc/thymeleaf/index.html\x60。",
         "两个构建产物的 SHA-256 校验清单") if chinese
        else ("Cargo source package identical to the crates.io package",
              "Offline API documentation built with all features. Extract and open \x60doc/thymeleaf/index.html\x60.",
              "SHA-256 checksums for both artifacts")
    )
    asset_rows = "\n".join(
        f"| [\x60{name}\x60]({download}/{name}) | {description} |"
        for name, description in zip(asset_names, descriptions)
    )
    # Changelog links are checkout-relative. Releases link to the same tagged files.
    notes = re.sub(r"\]\((?:\./)?(docs/[^)\s]+)\)", rf"]({repository}/blob/{tag}/\1)", notes)
    verify_help = (
        "将三个文件下载到同一目录，在该目录执行对应系统的命令。两个构建产物都应显示 \x60OK\x60。"
        if chinese else
        "Download all three files to the same directory and run the command for your system there. "
        "Both artifacts should report \x60OK\x60."
    )
    return f"""## {language}

### {install}

\x60\x60\x60sh
cargo add 'thymeleaf@={version}'
\x60\x60\x60

[crates.io](https://crates.io/crates/thymeleaf/{version}) · [docs.rs](https://docs.rs/thymeleaf/{version}/thymeleaf/)

### {changes}

{notes}

### {downloads}

{('| 文件 | 用途 |' if chinese else '| File | Purpose |')}
| --- | --- |
{asset_rows}

### {verify}

{verify_help}

**Linux**

\x60\x60\x60sh
sha256sum -c SHA256SUMS
\x60\x60\x60

**macOS**

\x60\x60\x60sh
shasum -a 256 -c SHA256SUMS
\x60\x60\x60
"""


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("version", help="Cargo version without the v prefix")
    args = parser.parse_args()
    version = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["workspace"]["package"]["version"]
    if version != args.version:
        parser.error(f"requested {args.version} but the workspace version is {version}")

    slug = os.environ.get("GITHUB_REPOSITORY", "easy-4-rust/thymeleaf-rust")
    repository = f"https://github.com/{slug}"
    try:
        notes = highlights((ROOT / "CHANGELOG.md").read_text(encoding="utf-8"), version)
    except ValueError as error:
        parser.error(str(error))
    sections = (render_section(notes["简体中文"], version, repository, True),
                render_section(notes["English"], version, repository, False))
    print("\n---\n\n".join(sections), end="")


if __name__ == "__main__":
    main()
