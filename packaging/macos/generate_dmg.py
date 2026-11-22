"""generate_dmg.py
Packages a staged macOS application bundle into a distributable .dmg disk image.
On macOS systems, leverages native `hdiutil`. On non-macOS cross-environments, creates
a compressed .tar.gz bundle with .app structure and documents DMG packaging commands.
"""
import os
import sys
import shutil
import hashlib
import argparse
import subprocess
from pathlib import Path

def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()

def create_dmg(app_bundle: Path, output_dmg: Path, volume_name: str = "TZWorkQuint"):
    output_dmg.parent.mkdir(parents=True, exist_ok=True)
    temp_dmg = output_dmg.with_suffix(".temp.dmg")
    
    if sys.platform == "darwin":
        print(f"Creating DMG on macOS using hdiutil: {output_dmg.name}...")
        # Create staging folder with symlink to /Applications
        stage_dmg = output_dmg.parent / "dmg_stage"
        if stage_dmg.exists():
            shutil.rmtree(stage_dmg)
        stage_dmg.mkdir(parents=True)
        
        # Copy .app bundle into staging
        dest_app = stage_dmg / app_bundle.name
        shutil.copytree(app_bundle, dest_app, symlinks=True)
        
        # Symlink /Applications
        os.symlink("/Applications", stage_dmg / "Applications")
        
        # Build UDZO compressed DMG
        cmd = [
            "hdiutil", "create",
            "-volname", volume_name,
            "-srcfolder", str(stage_dmg),
            "-ov",
            "-format", "UDZO",
            str(output_dmg)
        ]
        subprocess.run(cmd, check=True)
        shutil.rmtree(stage_dmg)
        print(f"DMG created successfully: {output_dmg} ({output_dmg.stat().st_size:,} bytes, SHA-256: {sha256_file(output_dmg)})")
    else:
        print(f"Non-macOS platform ({sys.platform}) detected.")
        print(f"Creating fallback compressed bundle for {app_bundle.name}...")
        tar_gz = output_dmg.with_suffix(".tar.gz")
        shutil.make_archive(str(tar_gz.with_suffix("").with_suffix("")), "gztar", root_dir=app_bundle.parent, base_dir=app_bundle.name)
        print(f"Created macOS bundle archive: {tar_gz} ({tar_gz.stat().st_size:,} bytes)")
        print(f"Note: Run `hdiutil create -volname '{volume_name}' -srcfolder <bundle_dir> -format UDZO '{output_dmg.name}'` on a macOS host.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Create macOS DMG from app bundle")
    parser.add_argument("app_bundle", type=Path, help="Path to .app bundle")
    parser.add_argument("output_dmg", type=Path, help="Destination .dmg path")
    parser.add_argument("--volume-name", type=str, default="TZWorkQuint", help="DMG volume name")
    args = parser.parse_args()
    create_dmg(args.app_bundle, args.output_dmg, args.volume_name)

