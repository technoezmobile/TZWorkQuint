"""package_windows.py
Automates creation of Windows packages for TZWorkQuint:
- Inno Setup Executable Installer (.exe)
- WiX 3 Windows Installer Package (.msi)
- Portable Suite Archive (.zip)
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

def package_windows(stage_dir: Path, out_dir: Path, version: str = "3.0.0", arch: str = "x64"):
    out_dir.mkdir(parents=True, exist_ok=True)
    bin_dir = stage_dir / "bin"
    apps = ["tz-workquint", "tz-cellforge", "tz-wordloom", "tz-slidespark", "tz-pdflex", "tz-draftline"]
    
    print(f"=== Packaging TZWorkQuint {version} for Windows ({arch}) ===")
    print(f"Staging payload: {stage_dir}")
    print(f"Output directory: {out_dir}")

    # 1. Verify required executables exist
    missing = []
    for app in apps:
        exe = bin_dir / f"{app}.exe"
        if not exe.exists():
            missing.append(exe.name)
        else:
            print(f"  Found binary: {exe.name} ({exe.stat().st_size:,} bytes)")
    if missing:
        raise SystemExit(f"Error: Missing required suite executables: {', '.join(missing)}")

    # 2. Build Portable ZIP
    zip_name = f"TZWorkQuint-{version}-win64.zip" if arch == "x64" else f"TZWorkQuint-{version}-win-arm64.zip"
    zip_path = out_dir / zip_name
    print(f"\nCreating Portable ZIP: {zip_path.name}...")
    shutil.make_archive(str(out_dir / zip_path.stem), "zip", root_dir=stage_dir)
    print(f"  Created {zip_path.name} ({zip_path.stat().st_size:,} bytes, SHA-256: {sha256_file(zip_path)})")

    # 3. Build WiX MSI if candle/light are available
    candle = None
    light = None
    wix_env = os.environ.get("WIX_PATH")
    if wix_env and (Path(wix_env) / "candle.exe").exists():
        candle = Path(wix_env) / "candle.exe"
        light = Path(wix_env) / "light.exe"
    elif shutil.which("candle") and shutil.which("light"):
        candle = Path(shutil.which("candle"))
        light = Path(shutil.which("light"))
    elif Path(r"C:\Program Files (x86)\WiX Toolset v3.11\bin\candle.exe").exists():
        candle = Path(r"C:\Program Files (x86)\WiX Toolset v3.11\bin\candle.exe")
        light = Path(r"C:\Program Files (x86)\WiX Toolset v3.11\bin\light.exe")
    elif Path(r"C:\Program Files (x86)\WiX Toolset v3.14\bin\candle.exe").exists():
        candle = Path(r"C:\Program Files (x86)\WiX Toolset v3.14\bin\candle.exe")
        light = Path(r"C:\Program Files (x86)\WiX Toolset v3.14\bin\light.exe")
    elif Path(r"E:\CRAFTING SOFTWARES\artifacts\github-release-prep\tooling\wix\candle.exe").exists():
        candle = Path(r"E:\CRAFTING SOFTWARES\artifacts\github-release-prep\tooling\wix\candle.exe")
        light = Path(r"E:\CRAFTING SOFTWARES\artifacts\github-release-prep\tooling\wix\light.exe")

    if candle and light:
        msi_name = f"TZWorkQuint-{version}-win64.msi" if arch == "x64" else f"TZWorkQuint-{version}-win-arm64.msi"
        msi_path = out_dir / msi_name
        wxs_path = out_dir / f"suite-{arch}.wxs"
        wixobj_path = out_dir / f"suite-{arch}.wixobj"
        print(f"\nCompiling WiX MSI: {msi_path.name}...")
        gen_script = Path(__file__).parent / "generate_msi.py"
        subprocess.run([sys.executable, str(gen_script), str(stage_dir), str(wxs_path), version, arch], check=True)
        candle_arch = "x64"
        subprocess.run([str(candle), "-arch", candle_arch, "-out", str(wixobj_path), str(wxs_path)], check=True)
        subprocess.run([str(light), "-sval", "-out", str(msi_path), str(wixobj_path)], check=True)
        print(f"  Created {msi_path.name} ({msi_path.stat().st_size:,} bytes, SHA-256: {sha256_file(msi_path)})")
    else:
        print(f"\nNotice: WiX binaries not found; skipping local MSI compilation.")

    # 4. Build Inno Setup Installer if ISCC is available
    iscc = None
    iscc_env = os.environ.get("ISCC_PATH")
    if iscc_env and Path(iscc_env).exists():
        iscc = Path(iscc_env)
    elif shutil.which("ISCC") or shutil.which("iscc"):
        iscc = Path(shutil.which("ISCC") or shutil.which("iscc"))
    elif Path(r"C:\Program Files (x86)\Inno Setup 6\ISCC.exe").exists():
        iscc = Path(r"C:\Program Files (x86)\Inno Setup 6\ISCC.exe")
    elif Path(r"C:\Users\singh\AppData\Local\Programs\Inno Setup 6\ISCC.exe").exists():
        iscc = Path(r"C:\Users\singh\AppData\Local\Programs\Inno Setup 6\ISCC.exe")

    iss_file = Path(__file__).parent / ("tzworkquint.iss" if arch == "x64" else "tzworkquint-arm64.iss")
    if iscc and iss_file.exists():
        print(f"\nCompiling Inno Setup EXE Installer with {iscc}...")
        subprocess.run([str(iscc), str(iss_file)], check=True)
        exe_name = f"TZWorkQuint-{version}-Setup.exe" if arch == "x64" else f"TZWorkQuint-{version}-Setup-arm64.exe"
        exe_path = out_dir / exe_name
        if exe_path.exists():
            print(f"  Created {exe_path.name} ({exe_path.stat().st_size:,} bytes, SHA-256: {sha256_file(exe_path)})")
    else:
        print(f"\nNotice: Inno Setup compiler not found; skipping local EXE compilation.")

    print("\nWindows packaging complete.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Package TZWorkQuint for Windows")
    parser.add_argument("--stage", type=Path, default=Path(r"E:\CRAFTING SOFTWARES\dist\TZWorkQuint-3.0.0-win64"))
    parser.add_argument("--out", type=Path, default=Path(r"E:\CRAFTING SOFTWARES\dist"))
    parser.add_argument("--version", type=str, default="3.0.0")
    parser.add_argument("--arch", type=str, default="x64", choices=["x64", "arm64"])
    args = parser.parse_args()
    package_windows(args.stage, args.out, args.version, args.arch)

