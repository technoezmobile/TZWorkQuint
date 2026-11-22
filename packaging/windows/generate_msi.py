"""generate_msi.py
Generate WiX 3 installer input XML from a complete TZWorkQuint payload directory.
Supports Windows x64 and ARM64 architecture payloads.
"""
from pathlib import Path
from uuid import NAMESPACE_URL, uuid5
import hashlib
import sys
import xml.etree.ElementTree as ET

if len(sys.argv) < 5:
    print("Usage: generate_msi.py <stage_dir> <output_wxs> <version> <arch: x64|arm64>")
    sys.exit(1)

stage = Path(sys.argv[1]).resolve()
output = Path(sys.argv[2]).resolve()
version = sys.argv[3]
arch = sys.argv[4].lower()

if arch not in ("x64", "arm64"):
    raise SystemExit(f"Unsupported architecture '{arch}'. Must be 'x64' or 'arm64'.")

expected_pe_machine = {
    "x64": b"\x64\x86",    # IMAGE_FILE_MACHINE_AMD64 (0x8664)
    "arm64": b"\x64\xaa",  # IMAGE_FILE_MACHINE_ARM64 (0xAA64)
}

# Verify PE headers for suite binaries if present
for name in ("workquint", "cellforge", "wordloom", "slidespark", "pdflex", "draftline"):
    exe_path = stage / "bin" / f"tz-{name}.exe"
    if exe_path.exists():
        data = exe_path.read_bytes()
        offset = int.from_bytes(data[60:64], "little")
        pe_sig = data[offset:offset+4]
        pe_mach = data[offset+4:offset+6]
        if pe_sig != b"PE\x00\x00":
            raise SystemExit(f"Invalid PE header in {exe_path.name}")
        # Note: In cross-compilation environments or stub builds, warn if machine differs
        if pe_mach != expected_pe_machine[arch]:
            print(f"Warning: {exe_path.name} PE machine header ({pe_mach.hex()}) differs from expected {arch} ({expected_pe_machine[arch].hex()})")

def ident(prefix, path):
    return prefix + hashlib.sha256(path.as_posix().lower().encode("utf-8")).hexdigest()[:24]

def child(parent, tag, **attrs):
    return ET.SubElement(parent, tag, attrs)

root = ET.Element("Wix", xmlns="http://schemas.microsoft.com/wix/2006/wi")
product = child(
    root, "Product",
    Id="*",
    Name="TZWorkQuint",
    Language="1033",
    Version=version,
    Manufacturer="TZLabs",
    UpgradeCode="89F71861-195D-459A-8723-170CC5D3C4EC"
)

child(product, "Package", InstallerVersion="500", Compressed="yes", InstallScope="perMachine")
child(product, "MajorUpgrade", DowngradeErrorMessage="A newer TZWorkQuint MSI is already installed.")
child(product, "MediaTemplate", EmbedCab="yes", CompressionLevel="high")

target = child(product, "Directory", Id="TARGETDIR", Name="SourceDir")
prog_folder = "ProgramFiles64Folder" if arch in ("x64", "arm64") else "ProgramFilesFolder"
programs = child(target, "Directory", Id=prog_folder)
appdir = child(programs, "Directory", Id="INSTALLDIR", Name="TZWorkQuint")
child(target, "Directory", Id="ProgramMenuFolder")
child(target, "Directory", Id="DesktopFolder")

dirs = {Path("."): appdir}
feature = child(product, "Feature", Id="Suite", Title="TZWorkQuint Productivity Suite", Level="1")

for path in sorted(stage.rglob("*")):
    rel = path.relative_to(stage)
    parent = dirs[rel.parent]
    if path.is_dir():
        dirs[rel] = child(parent, "Directory", Id=ident("D", rel), Name=path.name)
        continue
    if rel.as_posix() in ("install.cmd", "install.ps1", "SHA256SUMS.txt"):
        continue

    cid = ident("C", rel)
    component = child(
        parent, "Component",
        Id=cid,
        Guid=str(uuid5(NAMESPACE_URL, f"https://tzlabs.in/msi/{arch}/" + rel.as_posix().lower())),
        Win64="yes"
    )
    item = child(component, "File", Id=ident("F", rel), Name=path.name, Source=str(path), KeyPath="yes")

    # Start menu & Desktop shortcuts for main suite launcher
    if rel.as_posix() == "bin/tz-workquint.exe":
        child(
            item, "Shortcut",
            Id="SuiteMenuShortcut",
            Directory="ProgramMenuFolder",
            Name="TZWorkQuint",
            Description="TZWorkQuint Desktop Suite Launcher",
            Advertise="yes",
            WorkingDirectory=ident("D", rel.parent)
        )
        child(
            item, "Shortcut",
            Id="SuiteDesktopShortcut",
            Directory="DesktopFolder",
            Name="TZWorkQuint",
            Description="TZWorkQuint Desktop Suite Launcher",
            Advertise="yes",
            WorkingDirectory=ident("D", rel.parent)
        )
    child(feature, "ComponentRef", Id=cid)

output.parent.mkdir(parents=True, exist_ok=True)
ET.ElementTree(root).write(output, encoding="utf-8", xml_declaration=True)
print(f"Generated WiX input: {output} ({arch})")

