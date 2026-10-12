"""Compare completed, versioned native burial runs and their saved state."""
import json
from pathlib import Path
import sys
root=Path(__file__).resolve().parent.parent/"work/integration"
left,right,output=sys.argv[1:]
records=[]
for run, fixture in ((left,"burial-kotlin-fixture-001.json"),(right,"burial-floor-fixture-001.json")):
    directory=(root/"runs"/run).resolve()
    if directory.parent!=(root/"runs").resolve(): raise ValueError("invalid run")
    result=json.loads((directory/"result.json").read_text(encoding="utf-8-sig"))
    shutdown=json.loads((directory/"shutdown.json").read_text(encoding="utf-8-sig"))
    state=json.loads((directory/"state.json").read_text(encoding="utf-8-sig"))
    before=json.loads((root/fixture).read_text())
    assert result["mode"]=="inventory-bury" and result["clientExit"]==0
    assert shutdown["exited"] and not shutdown["forced"] and shutdown["exitCode"]==0
    assert state["session_metadata"]["world_id"] is None
    prayer=next(s for s in state["stats"] if s["stat_id"]==5)
    assert prayer["fine_xp"]==490, prayer
    inventory=[ [item["slot"],item["obj"],item["count"],item["vars"]] for item in state["inventory"] if item["inv_type"]==93]
    assert inventory==before["inventory_before"], "burial changed unrelated inventory"
    assert "Both native burials confirmed; Prayer XP=49" in (directory/"client.log").read_text()
    records.append({"run":run,"client_hash":result["clientHash"],"prayer":prayer,"inventory_conserved":True,"unforced_shutdown":True})
assert records[0]["client_hash"]==records[1]["client_hash"], "use the same native driver"
assert records[0]["prayer"]==records[1]["prayer"], "saved Prayer mismatch"
destination=(root/output).resolve()
if destination.parent!=root.resolve() or destination.exists(): raise ValueError("use a new output basename")
destination.write_text(json.dumps({"scenario":"native regular/bat bone burial","matched":True,"runs":records},indent=2)+"\n")
print("Matched native burial and saved Prayer fine XP490; original inventories preserved")
