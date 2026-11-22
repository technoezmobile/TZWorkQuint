"""generate_deb.py
Builds Debian (.deb) package for the TZWorkQuint suite.
Can construct the package using native dpkg-deb or fallback to pure Python
ar / tar.gz construction.
"""
import os
import sys
import shutil
import hashlib
import tarfile
import argparse
import subprocess
from pathlib import Path

def write_ar_header(f, name, size, mode=0o100644, mtime=0, uid=0, gid=0):
    header = f"{name:<16}{mtime:<12}{uid:<6}{gid:<6}{mode:<8o}{size:<10}`\n".encode("ascii")
    f.write(header)

def write_ar_file(out_file: Path, files: list):
    """Write standard ar archive (Debian .deb package container)."""
    with open(out_file, "wb") as f:
        f.write(b"!<arch>\n")
        for member_name, data in files:
            write_ar_header(f, member_name, len(data))
            f.write(data)
            if len(data) % 2 != 0:
                f.write(b"\n") # 2-byte alignment padding

def generate_deb(stage_payload: Path, out_dir: Path, version: str = "3.0.0", arch: str = "amd64"):
    out_dir.mkdir(parents=True, exist_ok=True)
    deb_arch = "amd64" if arch in ("x86_64", "amd64") else "arm64"
    deb_name = f"tzworkquint_{version}_{deb_arch}.deb"
    deb_path = out_dir / deb_name

    print(f"=== Generating Debian Package: {deb_name} ===")

    # Temporary tree for dpkg-deb / ar assembly
    build_root = out_dir / f"deb_build_{deb_arch}"
    if build_root.exists():
        shutil.rmtree(build_root)

    debian_dir = build_root / "DEBIAN"
    debian_dir.mkdir(parents=True)

    # 1. Control file
    control_content = f"""Package: tzworkquint
Version: {version}
Section: editors
Priority: optional
Architecture: {deb_arch}
Maintainer: TZLabs Packaging Team <contact@tzlabs.in>
Depends: libc6 (>= 2.31), libfontconfig1, libx11-6
Description: TZWorkQuint Productivity & Engineering Suite
 TZWorkQuint from TZLabs bundles five native desktop editors (Wordloom,
 CellForge, SlideSpark, PDFlex, Draftline) with a lightweight desktop
 launcher and an optional AI assistant.
Homepage: https://tzlabs.in
"""
    (debian_dir / "control").write_text(control_content, encoding="utf-8")

    # 2. postinst and prerm scripts
    postinst_content = """#!/bin/sh
set -e
# Create convenient /usr/local/bin symlinks
mkdir -p /usr/local/bin
for app in tz-workquint tz-cellforge tz-wordloom tz-slidespark tz-pdflex tz-draftline; do
    if [ -f "/opt/tzworkquint/bin/$app" ]; then
        ln -sf "/opt/tzworkquint/bin/$app" "/usr/local/bin/$app"
    fi
done

# Update desktop database and icon caches if available
if which update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications || true
fi
if which gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q /usr/share/icons/hicolor || true
fi
exit 0
"""
    (debian_dir / "postinst").write_text(postinst_content, encoding="utf-8")
    (debian_dir / "postinst").chmod(0o755)

    prerm_content = """#!/bin/sh
set -e
# Remove /usr/local/bin symlinks
for app in tz-workquint tz-cellforge tz-wordloom tz-slidespark tz-pdflex tz-draftline; do
    rm -f "/usr/local/bin/$app"
done
exit 0
"""
    (debian_dir / "prerm").write_text(prerm_content, encoding="utf-8")
    (debian_dir / "prerm").chmod(0o755)

    # 3. Payload directories
    opt_dir = build_root / "opt" / "tzworkquint"
    shutil.copytree(stage_payload, opt_dir)

    # Desktop entries
    apps_dir = build_root / "usr" / "share" / "applications"
    apps_dir.mkdir(parents=True)
    linux_pkg_dir = Path(__file__).parent
    for desktop_file in linux_pkg_dir.glob("*.desktop"):
        shutil.copy2(desktop_file, apps_dir / desktop_file.name)

    # Icons
    icons_dir = build_root / "usr" / "share" / "icons" / "hicolor" / "128x128" / "apps"
    icons_dir.mkdir(parents=True)
    if (opt_dir / "assets" / "brand").exists():
        for png in (opt_dir / "assets" / "brand").glob("*-128.png"):
            app_name = png.stem.replace("-128", "")
            shutil.copy2(png, icons_dir / f"{app_name}.png")

    # Try native dpkg-deb if installed
    if shutil.which("dpkg-deb"):
        print("Using system dpkg-deb...")
        subprocess.run(["dpkg-deb", "--build", "--root-owner-group", str(build_root), str(deb_path)], check=True)
    else:
        print("dpkg-deb not available; constructing standard Debian package via Python...")
        # 1. debian-binary
        deb_binary = b"2.0\n"

        # 2. control.tar.gz
        control_tar = build_root / "control.tar.gz"
        with tarfile.open(control_tar, "w:gz") as tar:
            for item in sorted(debian_dir.glob("*")):
                tar.add(item, arcname=f"./{item.name}")
        control_data = control_tar.read_bytes()

        # 3. data.tar.gz
        data_tar = build_root / "data.tar.gz"
        with tarfile.open(data_tar, "w:gz") as tar:
            for top_dir in ("opt", "usr"):
                src_top = build_root / top_dir
                if src_top.exists():
                    tar.add(src_top, arcname=f"./{top_dir}")
        data_bytes = data_tar.read_bytes()

        # Assemble ar container
        write_ar_file(deb_path, [
            ("debian-binary", deb_binary),
            ("control.tar.gz", control_data),
            ("data.tar.gz", data_bytes)
        ])

    shutil.rmtree(build_root)
    h = hashlib.sha256(deb_path.read_bytes()).hexdigest()
    print(f"DEB package created: {deb_path.name} ({deb_path.stat().st_size:,} bytes, SHA-256: {h})")
    return deb_path

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate DEB package for TZWorkQuint")
    parser.add_argument("stage_payload", type=Path, help="Path to TZWorkQuint staged folder")
    parser.add_argument("out_dir", type=Path, help="Output directory")
    parser.add_argument("--version", type=str, default="3.0.0")
    parser.add_argument("--arch", type=str, default="amd64", choices=["amd64", "arm64", "x86_64", "aarch64"])
    args = parser.parse_args()
    generate_deb(args.stage_payload, args.out_dir, args.version, args.arch)

