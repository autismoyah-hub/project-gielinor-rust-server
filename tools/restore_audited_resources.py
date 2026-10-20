"""Restore a restricted audited resource backup into a NEW directory only.

The manifest and payload store remain private. This tool ships no asset bytes,
credentials or reference material, and makes no redistribution-rights decision.
"""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import shutil


def digest(path):
    with path.open("rb") as src:
        return hashlib.file_digest(src, "sha256").hexdigest()


def restore(manifest, payloads, output, prefixes=()):
    manifest, payloads, output = Path(manifest), Path(payloads), Path(output)
    if output.exists():
        raise FileExistsError("Restore output already exists")
    data = json.loads(manifest.read_text(encoding="utf-8-sig"))
    if data.get("failures"):
        raise ValueError("Backup has unresolved failures")
    entries = []
    paths = set()
    verified = set()
    for entry in data["verified_entries"]:
        relative = entry["path"]
        if prefixes and not any(relative.startswith(prefix) for prefix in prefixes):
            continue
        path = PurePosixPath(relative)
        if (path.is_absolute() or ".." in path.parts or not path.parts
                or any(":" in part or "\\" in part for part in path.parts)):
            raise ValueError("Unsafe resource path")
        if relative in paths:
            raise ValueError("Duplicate resource path")
        paths.add(relative)
        sha = entry["sha256"]
        if not re.fullmatch("[0-9a-f]{64}", sha):
            raise ValueError("Invalid payload identity")
        source = payloads / sha[:2] / sha
        if sha not in verified:
            if source.stat().st_size != entry["bytes"] or digest(source) != sha:
                raise ValueError("Resource payload mismatch")
            verified.add(sha)
        entries.append((entry, source))
    if not entries:
        raise ValueError("No selected resource entries")
    # Never merge into an existing scene/source tree or replace an old master.
    output.mkdir(exist_ok=False)
    root = output.resolve()
    receipt = []
    for entry, source in entries:
        target = output / entry["path"]
        target.parent.mkdir(parents=True, exist_ok=True)
        if not target.resolve().is_relative_to(root):
            raise ValueError("Restore escaped output directory")
        with source.open("rb") as src, target.open("xb") as dst:
            shutil.copyfileobj(src, dst, 1024 * 1024)
        if digest(target) != entry["sha256"]:
            raise ValueError("Restored resource mismatch")
        receipt.append({"path": entry["path"], "sha256": entry["sha256"]})
    result = {"manifest_sha256": digest(manifest), "restored": receipt,
              "distinct_payloads": len(verified), "status": "PASS"}
    with (output / "restore-receipt.json").open("x", encoding="utf-8") as dst:
        json.dump(result, dst, indent=2)
    return {"status": "PASS", "files": len(receipt),
            "distinct_payloads": len(verified), "manifest_sha256": result["manifest_sha256"]}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path)
    parser.add_argument("payloads", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--prefix", action="append", default=[])
    args = parser.parse_args()
    print(json.dumps(restore(args.manifest, args.payloads, args.output, args.prefix)))
