"""Build the x64 MSI from the Windows portable payload using wixl."""

import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import uuid
import xml.etree.ElementTree as ET


UPGRADE_CODE = uuid.UUID("8d48e4ac-c17e-4557-82ab-8429edaa32da")
node = ET.SubElement


def identifier(prefix, path):
    return prefix + uuid.uuid5(UPGRADE_CODE, path).hex


def product_version(version):
    # MSI compares three numeric fields. Reserve the last slot for stable.
    match = re.fullmatch(r"(\d+)\.(\d+)\.(\d+)(?:-beta\.(\d+))?", version)
    if not match:
        raise ValueError(f"Unsupported MSI version: {version}")
    major, minor, patch = map(int, match.groups()[:3])
    beta = match[4]
    if max(major, minor, patch) > 255 or (beta and int(beta) >= 255):
        raise ValueError(f"Version exceeds MSI field range: {version}")
    return f"{major}.{minor}.{patch * 256 + (int(beta) if beta else 255)}"


def build(version, payload, output):
    if not (payload / "hibi.exe").is_file():
        raise ValueError(f"Missing hibi.exe in {payload}")
    root = ET.Element("Wix", xmlns="http://schemas.microsoft.com/wix/2006/wi")
    product = node(root, "Product", Id=str(uuid.uuid5(UPGRADE_CODE, "product/" + version)),
                   Name="Hibi", Language="1033", Manufacturer="hibi-ai",
                   Version=product_version(version), UpgradeCode=str(UPGRADE_CODE))
    node(product, "Package", InstallerVersion="500", Compressed="yes", InstallScope="perMachine")
    node(product, "MajorUpgrade", DowngradeErrorMessage="A newer version of Hibi is installed.")
    # Windows Installer can restore the old product only within the upgrade transaction.
    sequence = node(product, "InstallExecuteSequence")
    node(sequence, "RemoveExistingProducts", After="InstallInitialize")
    node(product, "Media", Id="1", Cabinet="hibi.cab", EmbedCab="yes")
    icon = Path(__file__).resolve().parent / "assets" / "hibi.ico"
    node(product, "Icon", Id="hibi.ico", SourceFile=os.path.relpath(icon, payload))
    node(product, "Property", Id="ARPPRODUCTICON", Value="hibi.ico")
    target = node(product, "Directory", Id="TARGETDIR", Name="SourceDir")
    programs = node(target, "Directory", Id="ProgramFiles64Folder")
    node(target, "Directory", Id="ProgramMenuFolder")
    directories = {Path("."): node(programs, "Directory", Id="INSTALLDIR", Name="Hibi")}
    feature = node(product, "Feature", Id="Hibi", Title="Hibi", Level="1")
    for path in sorted(payload.rglob("*")):
        rel = path.relative_to(payload)
        name = rel.as_posix()
        if name == ".hibi-bundle":
            continue
        parent = directories[rel.parent]
        if path.is_dir():
            directories[rel] = node(parent, "Directory", Id=identifier("D", name), Name=rel.name)
            continue
        component_id = identifier("C", name)
        component = node(parent, "Component", Id=component_id,
                         Guid=str(uuid.uuid5(UPGRADE_CODE, name)), Win64="yes")
        file = node(component, "File", Id=identifier("F", name), Name=rel.name, Source=name, KeyPath="yes")
        if name == "hibi.exe":
            node(file, "Shortcut", Id="HibiShortcut", Directory="ProgramMenuFolder",
                 Name="Hibi", Icon="hibi.ico", Advertise="yes", WorkingDirectory="INSTALLDIR")
        node(feature, "ComponentRef", Id=component_id)

    with tempfile.NamedTemporaryFile(dir=payload, suffix=".wxs") as source:
        ET.ElementTree(root).write(source, encoding="utf-8", xml_declaration=True)
        source.flush()
        subprocess.run(["wixl", "-a", "x64", "-o", str(output.resolve()), source.name],
                       cwd=payload, check=True)


if __name__ == "__main__":
    build(sys.argv[1], Path(sys.argv[2]).resolve(), Path(sys.argv[3]))
