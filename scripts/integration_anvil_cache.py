"""Read-only smithing cache preflight; runtime decoding/native behavior remain separate."""
import hashlib
import importlib.util
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("widget_cache", ROOT / "scripts/integration-widget-cache.py")
cache = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cache)

class Reader:
    def __init__(self, data): self.data, self.pos = data, 0
    def take(self, n):
        result = self.data[self.pos:self.pos+n]
        if len(result) != n: raise ValueError("Truncated definition")
        self.pos += n
        return result
    def integer(self, n, signed=False): return int.from_bytes(self.take(n), "big", signed=signed)
    def string(self):
        end = self.data.index(0, self.pos)
        value = self.data[self.pos:end]
        self.pos = end+1
        return value

def enum(data):
    r = Reader(data)
    result = {"entries": {}}
    while True:
        op = r.integer(1)
        if op == 0: break
        if op in (1, 2): result["key" if op == 1 else "value"] = chr(r.integer(1))
        elif op == 3: result["default_string_hex"] = r.string().hex()
        elif op == 4: result["default"] = r.integer(4, True)
        elif op == 6:
            result["entries"] = {r.integer(4, True): r.integer(4, True) for _ in range(r.integer(2))}
        elif op == 200: result["transmit"] = False
        else: raise ValueError(f"Unexpected smithing enum opcode {op}")
    if r.pos != len(data): raise ValueError("Trailing enum bytes")
    if (result["key"], result["value"]) != ("o", "i"): raise ValueError("Wrong enum types")
    result["sha256"] = hashlib.sha256(data).hexdigest()
    return result

def events(data):
    r = Reader(data)
    if r.integer(1) != 255: raise ValueError("Expected IF3 smithing component")
    kind = r.integer(1)
    r.take(10)
    width, height = r.integer(1), r.integer(1)
    r.take(5)
    if kind == 0: r.take(5)
    elif kind == 5: r.take(15)
    elif kind == 6:
        r.take(21)
        if width: r.take(2)
        if height: r.take(2)
    elif kind == 4:
        r.take(2); r.string(); r.take(8)
    elif kind in (3, 9): r.take(6)
    return r.integer(3)

def inspect():
    root = ROOT / "work/integration/b-rust/cache/game"
    source = ROOT / "crates/world/src/anvil_products.rs"
    text = source.read_text(encoding="utf8")
    rows = [tuple(map(int, row)) for row in re.findall(r"\((\d+), (\d+), (\d+)\)", text)]
    if len(rows) != 188: raise ValueError("Unexpected product grid")
    products = {row[2] for row in rows}
    tables = {id: enum(cache.file(root, 2, 8, id)) for id in (844, 845, 846)}
    for id, table in tables.items():
        missing = products - table["entries"].keys()
        if missing: raise ValueError(f"Missing enum {id} products: {sorted(missing)}")
    components = {}
    for slot in sorted({3, 4, 5, 6, 7} | {row[1] for row in rows}):
        data = cache.file(root, 3, 312, slot)
        mask = events(data)
        if not mask & 2: raise ValueError(f"Missing cached Op1 on 312:{slot}")
        components[slot] = {"events": mask, "sha256": hashlib.sha256(data).hexdigest()}
    return {"status": "PASS", "scope": "Owned game-cache enum payloads and smithing grid/button Op1 masks; not native acceptance or Rust decoder validation", "cache_root": str(root), "product_source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(), "product_slots": len(rows), "tables": tables, "components": components}

if __name__ == "__main__":
    import sys
    result = inspect()
    with Path(sys.argv[1]).open("x", encoding="utf8") as out: json.dump(result, out, indent=2)
    print(f"PASS: {result['product_slots']} product slots, 3 enum tables, {len(result['components'])} cached Op1 components")
