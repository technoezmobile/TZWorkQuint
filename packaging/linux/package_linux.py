"""package_linux.py
Assembles Linux release archives (.tar.gz) for the TZWorkQuint suite.
Supports x86_64 and aarch64 architectures.
"""
import os
import sys
import shutil
import hashlib
import tarfile
import argparse
from pathlib import Path

def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()

def package_linux(bin_dir: Path, assets_dir: Path, docs_dir: Path, out_dir: Path, version: str = "3.0.0", arch: str = "x86_64"):
    out_dir.mkdir(parents=True, exist_ok=True)
    pkg_name = f"TZWorkQuint-{version}-linux-{arch}"
    stage = out_dir / pkg_name
    if stage.exists():
        shutil.rmtree(stage)
    stage.mkdir(parents=True)

    print(f"=== Packaging TZWorkQuint {version} for Linux ({arch}) ===")

    # 1. Copy binaries
    apps = ["tz-workquint", "tz-cellforge", "tz-wordloom", "tz-slidespark", "tz-pdflex", "tz-draftline"]
    stage_bin = stage / "bin"
    stage_bin.mkdir(parents=True)
    missing = []
    for app in apps:
        src = bin_dir / app
        if src.exists():
            dest = stage_bin / app
            shutil.copy2(src, dest)
            dest.chmod(0o755)
            print(f"  Added binary: {app} ({dest.stat().st_size:,} bytes)")
        else:
            missing.append(app)
    if missing:
        raise SystemExit(f"Error: Missing required binaries at {bin_dir}: {', '.join(missing)}")

    # 2. Copy shared assets and docs
    if assets_dir.exists():
        shutil.copytree(assets_dir, stage / "assets")
    if docs_dir.exists():
        shutil.copytree(docs_dir, stage / "docs")

    # 3. Add desktop entries
    desktop_dir = stage / "share" / "applications"
    desktop_dir.mkdir(parents=True)
    linux_pkg_dir = Path(__file__).parent
    for desktop_file in linux_pkg_dir.glob("*.desktop"):
        shutil.copy2(desktop_file, desktop_dir / desktop_file.name)

    # 4. Copy icons
    icons_dir = stage / "share" / "icons" / "hicolor" / "128x128" / "apps"
    icons_dir.mkdir(parents=True)
    if (stage / "assets" / "brand").exists():
        for png in (stage / "assets" / "brand").glob("*-128.png"):
            app_name = png.stem.replace("-128", "")
            shutil.copy2(png, icons_dir / f"{app_name}.png")

    # 5. Create tar.gz archive
    archive_path = out_dir / f"{pkg_name}.tar.gz"
    print(f"\nCompressing {archive_path.name}...")
    with tarfile.open(archive_path, "w:gz") as tar:
        tar.add(stage, arcname=pkg_name)
    shutil.rmtree(stage)

    print(f"Linux portable package created: {archive_path} ({archive_path.stat().st_size:,} bytes, SHA-256: {sha256_file(archive_path)})")
    return archive_path

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Package TZWorkQuint for Linux")
    parser.add_argument("--bin-dir", type=Path, required=True, help="Directory containing compiled Linux binaries")
    parser.add_argument("--assets-dir", type=Path, required=True, help="Directory containing brand assets")
    parser.add_argument("--docs-dir", type=Path, required=True, help="Directory containing docs")
    parser.add_argument("--out-dir", type=Path, default=Path("dist"), help="Output directory")
    parser.add_argument("--version", type=str, default="3.0.0")
    parser.add_argument("--arch", type=str, default="x86_64", choices=["x86_64", "aarch64"])
    args = parser.parse_args()
    package_linux(args.bin_dir, args.assets_dir, args.docs_dir, args.out_dir, args.version, args.arch)

