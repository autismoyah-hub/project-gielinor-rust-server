"""Native Floor One acceptance via the existing ordinary-action test adapter.

All gameplay is earned through native menus/packets. Account databases are read
only for final verification. Never use a live account or a shared state name.
The server, unchanged client, approved content, private session configuration and
existing Gateway module are supplied explicitly; none are downloaded or edited.
"""
import argparse
import hashlib
import json
import secrets
import socket
import sqlite3
import subprocess
import sys
import time
from collections import defaultdict, deque
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(__doc__)
    for name in ["server", "client", "server_root", "content_root", "content_descriptor",
                 "session_config", "adapter_directory", "walk_edges", "output"]:
        parser.add_argument("--" + name.replace("_", "-"), type=Path, required=True)
    parser.add_argument("--server-source", required=True)
    parser.add_argument("--client-source", required=True)
    parser.add_argument("--resume", type=Path, help="Continue this driver's own preserved earned account")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    sys.path.insert(0, str(args.adapter_directory.resolve()))
    from tower_gateway import Gateway
    config = json.loads(args.session_config.read_text(encoding="utf-8"))
    # Navigation observations exported by the maintained read-only floor checker.
    # These guide visible native walking; they cannot modify server collision.
    edges = defaultdict(list)
    for line in args.walk_edges.read_text(encoding="utf-8").splitlines()[1:]:
        x, z, dx, dz = map(int, line.split())
        edges[(10368 + x, 64 + z)].append((10368 + x + dx, 64 + z + dz))
    previous = json.loads((args.resume / "results.private.json").read_text(encoding="utf-8")) if args.resume else None
    state_name = previous["state_name"] if previous else "agent7-native-" + secrets.token_hex(6)
    password = args.output / "password.private.txt"
    if previous:
        assert previous["server_sha256"] == hashlib.sha256(args.server.read_bytes()).hexdigest()
        assert previous["client_sha256"] == hashlib.sha256(args.client.read_bytes()).hexdigest()
        password.write_bytes((args.resume / "password.private.txt").read_bytes())
    else:
        password.write_text(secrets.token_hex(12), encoding="utf-8")
    database = args.server_root / "work/state" / state_name / "accounts.sqlite"
    report = {"state_name": state_name, "server_source": args.server_source, "client_source": args.client_source,
              "server_sha256": hashlib.sha256(args.server.read_bytes()).hexdigest(),
              "client_sha256": hashlib.sha256(args.client.read_bytes()).hexdigest(),
              "navigation_export_sha256": hashlib.sha256(args.walk_edges.read_bytes()).hexdigest(),
              "scenario": "Fresh native tools, earned kitchen milestone, portal, logout/server restart and return",
              "independent_verification": False, "accepted": False, "sessions": []}
    if previous:
        report["continued_earned_attempt"] = args.resume.name

    def symbol(kind, name):
        rows = (Path(config["symbols"]) / (kind + ".sym")).read_text(encoding="utf-8").splitlines()
        return next(int(row.split()[0]) for row in rows if len(row.split()) == 2 and row.split()[1] == name)

    tool_ids = [symbol("obj", name) for name in ["bronze_axe", "bronze_pickaxe", "net", "hammer", "tinderbox", "knife"]]
    # Pinned maintained floor anchors, also asserted by cinderworks_floor_check.
    # Historical experimental coordinates are intentionally not used.
    plaque, chest, portal = [symbol("loc", name) for name in ["plaque", "threshold_chest", "threshold_portal"]]
    fishing = symbol("npc", "0_48_48_newbiefishing")
    population = []
    population_file = args.walk_edges.parent / "npc-footprints.tsv"
    for line in population_file.read_text(encoding="utf-8").splitlines()[1:]:
        values = line.split()
        if int(values[1]) == fishing:
            population.append({"x": 10368 + int(values[2]), "z": 64 + int(values[3])})
    if not population:
        raise RuntimeError("Maintained navigation export has no authored fishing population")
    raw, cooked, burnt = [symbol("obj", name) for name in ["raw_shrimp", "shrimp", "burnt_shrimp"]]
    count = lambda state, item: sum(i["count"] for i in state["items"] if i["id"] == item)
    def conserved(state):
        # Native widget/operation metadata may change when switching tabs; the
        # conserved gameplay state is slot identity, amount, item vars and XP.
        inventory = lambda key: sorted((i["slot"], i["id"], i["count"], i.get("vars", 0))
                                       for i in state[key] if i["count"] > 0)
        return {"items": inventory("items"), "equipment": inventory("equipment"),
                "xp": state["xp"], "journal": state["journal"]}
    before_restart = None
    for session in range(2):
        directory = args.output / ("fresh" if session == 0 else "restart")
        directory.mkdir()
        stop = directory / "server.stop"
        with socket.socket() as probe:
            probe.bind(("127.0.0.1", 0))
            port = probe.getsockname()[1]
        address = f"127.0.0.1:{port}"
        server_args = [args.server, "--game-cache", config["game_cache"], "--js5-cache", config["js5_cache"],
                       "--symbols", config["symbols"], "--rsa-exponent", config["exponent"], "--rsa-modulus", config["modulus"],
                       "--argon2", config["argon2"], "--state-name", state_name, "--bind", address, "--spawn", "3222,3218,0",
                       "--cinderworks-content", args.content_root, "--stop-file", stop]
        with (directory / "server.log").open("wb") as server_log, (directory / "client.log").open("wb") as client_log:
            server = subprocess.Popen(list(map(str, server_args)), cwd=args.server_root, stdout=server_log,
                                      stderr=subprocess.STDOUT, creationflags=subprocess.CREATE_NO_WINDOW)
            client = None
            gateway = None
            try:
                deadline = time.monotonic() + 90
                while time.monotonic() < deadline:
                    if server.poll() is not None:
                        raise RuntimeError("Server exited; preserve server.log")
                    with socket.socket() as probe:
                        probe.settimeout(.1)
                        if probe.connect_ex(("127.0.0.1", port)) == 0:
                            break
                    time.sleep(.25)
                else:
                    raise TimeoutError("Server startup")
                client_args = [args.client, args.content_descriptor, config["client_cache"], address, "towerprobe",
                               password, config["public_key"], directory, "classic", "1", "--exposure=0.8"]
                client = subprocess.Popen(list(map(str, client_args)), cwd=args.output, stdout=client_log,
                                          stderr=subprocess.STDOUT, creationflags=subprocess.CREATE_NO_WINDOW)
                # The native adapter emits state only in response to a command.
                # Publish one read-only bootstrap observation, then let Gateway
                # advance from its acknowledged sequence without redispatching it.
                (directory / "command.txt").write_text("1 observe\n", encoding="utf-8")
                deadline = time.monotonic() + 100
                while not (directory / "state.json").exists():
                    if client.poll() is not None or time.monotonic() >= deadline:
                        raise RuntimeError("Native client did not publish state; preserve client.log")
                    time.sleep(.2)
                gateway = Gateway(directory, directory / "actions.ndjson")
                observed = gateway.wait(lambda s: s["ready"], 90)
                if session == 0 and not previous:
                    # Fresh registration dialogs are delivered on separate
                    # server ticks. Wait for each expected ordinary dialog;
                    # "ready" alone does not mean registration has finished.
                    observed = gateway.wait(lambda s: 679 in s.get("modals", []) or 929 in s.get("modals", []), 45)
                if 679 in observed.get("modals", []):
                    gateway.act("interface 44499018 1")
                    observed = gateway.wait(lambda s: 929 in s.get("modals", []), 45)
                if 929 in observed.get("modals", []):
                    gateway.act("interface 60882951 1")
                    observed = gateway.wait(lambda s: s["ready"] and 929 not in s.get("modals", []), 30)

                def walk(x, z):
                    until, retry = time.monotonic() + 55, 0
                    while time.monotonic() < until:
                        s = gateway.observe()
                        if s["ready"] and s["position"][1:] == [x, z]:
                            return s
                        if s["ready"] and time.monotonic() >= retry:
                            start, goal = tuple(s["position"][1:]), (x, z)
                            # Follow exported parent edges in short visible
                            # segments. All movement goes through the client.
                            parents, queue = {start: None}, deque([start])
                            while queue and goal not in parents:
                                at = queue.popleft()
                                for neighbor in edges[at]:
                                    if neighbor not in parents:
                                        parents[neighbor] = at
                                        queue.append(neighbor)
                            if goal not in parents:
                                raise RuntimeError("No maintained navigation route to native target")
                            route, at = [], goal
                            while at != start:
                                route.append(at)
                                at = parents[at]
                            route.reverse()
                            target = route[min(3, len(route) - 1)]
                            try:
                                gateway.act(f"walk {target[0]} {target[1]}")
                            except RuntimeError as error:
                                if "action is not in the visible native menu" not in str(error):
                                    raise
                                # The rejected menu sent no action; try the
                                # nearest visible step, never replay a timed-out action.
                                target = route[0]
                                gateway.act(f"walk {target[0]} {target[1]}")
                            retry = time.monotonic() + 4
                        time.sleep(.2)
                    raise TimeoutError("Ordinary walking did not reach the target")

                if session == 0:
                    if observed["position"][1] < 10000:
                        gateway.act(f"object {portal} 3225 3218 1")
                    gateway.wait(lambda s: s["ready"] and s["position"][1] > 10000, 35)
                    walk(10418, 106)
                    initial = gateway.observe()
                    initial_uid = max((e["uid"] for e in initial["message_events"]), default=-1)
                    gateway.act(f"object {plaque} 10418 105 1")
                    admitted = gateway.wait(lambda s: "run 1" in s["journal"] and all(count(s, tool) == 1 for tool in tool_ids)
                        and any(e["uid"] > initial_uid and (e["text"].startswith("Cinderworks run ")
                            or e["text"].startswith("Restore services in any order")) for e in s["message_events"]), 35)
                    assert "Cinderworks milestones: 0/5" in admitted["journal"]
                    gateway.act(f"object {plaque} 10418 105 1")
                    duplicate = gateway.wait(lambda s: s["ready"] and any(
                        event["uid"] > max((e["uid"] for e in admitted["message_events"]), default=-1)
                        and event["text"].startswith("Restore services in any order") for event in s["message_events"]), 25)
                    assert conserved(duplicate) == conserved(admitted), "Repeated recovery changed possessions or XP"
                    gateway.act(f"object {chest} 10419 105 1")
                    refused = gateway.wait(lambda s: s["ready"] and any(
                        event["uid"] > max((e["uid"] for e in duplicate["message_events"]), default=-1)
                        and event["text"].startswith("Restore services in any order") for event in s["message_events"]), 25)
                    assert conserved(refused) == conserved(admitted), "Unearned claim changed state"
                    gateway.act("interface 35913795 1")
                    for attempt in range(3):
                        state = gateway.observe()
                        target = min(population,
                                     key=lambda n: abs(n["x"] - state["position"][1]) + abs(n["z"] - state["position"][2]))
                        # Waterborne NPCs are deliberately off the walkable mesh.
                        # Choose their actual nearest exported shore, never an
                        # old experimental NPC position or a blocked water tile.
                        shore = min(edges, key=lambda at: (max(abs(at[0] - target["x"]), abs(at[1] - target["z"])),
                                                          abs(at[0] - target["x"]) + abs(at[1] - target["z"])))
                        walk(*shore)
                        visible = gateway.wait(lambda s: any(n["id"] == fishing for n in s["npcs"]), 15)
                        target = min((n for n in visible["npcs"] if n["id"] == fishing),
                                     key=lambda n: abs(n["x"] - shore[0]) + abs(n["z"] - shore[1]))
                        required_raw = max(8, count(visible, raw) + 1)
                        gateway.act(f"npc {target['index']} 1")
                        gateway.wait(lambda s: count(s, raw) >= required_raw, 130)
                        walk(*edges[shore][0])  # Interrupt gathering through ordinary movement.
                        caught = gateway.observe()
                        previous_total = count(caught, raw) + count(caught, cooked) + count(caught, burnt)
                        walk(10432, 109)
                        # Maintained cooking consumes one selected input per
                        # ordinary use, rather than the old experiment's batch.
                        # Explicitly select and cook each raw item; never change
                        # server rates, recipe queues or the character's state.
                        cooked_state = gateway.observe()
                        for _ in range(count(cooked_state, raw)):
                            remaining = count(cooked_state, raw)
                            raw_item = next(i for i in cooked_state["items"] if i["id"] == raw and i["count"] > 0)
                            gateway.act(f"select-item {raw_item['slot']}")
                            gateway.act(f"use-object {symbol('loc', 'newbierange')} 10432 107")
                            cooked_state = gateway.wait(lambda s: s["ready"] and count(s, raw) < remaining, 25)
                        assert count(cooked_state, raw) == 0
                        assert count(cooked_state, cooked) + count(cooked_state, burnt) == previous_total, "Food conservation"
                        if "Cinderworks milestones: 1/5" in cooked_state["journal"]:
                            break
                    else:
                        raise RuntimeError("Three bounded cooking batches produced no successful kitchen output")
                    assert "Done: Kitchen" in cooked_state["journal"]
                    assert "Ash Imp: not defeated" in cooked_state["journal"]
                    assert "Ash Warden: not defeated" in cooked_state["journal"]
                    assert all(count(cooked_state, tool) == 1 for tool in tool_ids)
                    report["earned_fishing_xp_delta"] = cooked_state["xp"][10] - admitted["xp"][10]
                    report["earned_cooking_xp_delta"] = cooked_state["xp"][7] - admitted["xp"][7]
                    assert report["earned_fishing_xp_delta"] > 0 and report["earned_cooking_xp_delta"] > 0
                    walk(10427, 114)
                    gateway.act(f"object {portal} 10428 114 1")
                    outside = gateway.wait(lambda s: s["ready"] and s["position"][1] < 10000, 35)
                    assert "Cinderworks milestones: 1/5" in outside["journal"]
                    before_restart = conserved(outside)
                else:
                    assert conserved(observed) == before_restart, "Logout/server restart changed earned progression"
                    gateway.act(f"object {portal} 3225 3218 1")
                    returned = gateway.wait(lambda s: s["ready"] and s["position"][1] > 10000, 35)
                    assert conserved(returned) == before_restart, "Return changed permanent state"
                final = gateway.observe()
                report["sessions"].append({"position": final["position"], "journal": final["journal"], "snapshot": conserved(final)})
                gateway.act("capture")
                gateway.act("stop")
                client.wait(timeout=30)
                assert client.returncode == 0
                deadline = time.monotonic() + 50
                while time.monotonic() < deadline:
                    with sqlite3.connect(database.resolve().as_uri() + "?mode=ro", uri=True) as db:
                        row = db.execute("SELECT last_logout,world_id FROM characters LIMIT 1").fetchone()
                    if row and row[0] and row[1] is None:
                        break
                    time.sleep(.4)
                else:
                    raise RuntimeError("Normal logout was not durably acknowledged")
            finally:
                try:
                    if client is not None and client.poll() is None:
                        try:
                            if gateway is None:
                                raise RuntimeError("No native adapter available for graceful stop")
                            gateway.act("stop")
                            client.wait(timeout=30)
                        except Exception:
                            # Only this owned disposable client; preserve a failed
                            # acceptance, never terminate another native session.
                            report["forced_disposable_client_cleanup"] = True
                            client.terminate()
                            client.wait(timeout=15)
                finally:
                    stop.write_text("Drain normal native acceptance saves", encoding="utf-8")
                    try:
                        server.wait(timeout=45)
                    finally:
                        (args.output / "results.private.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
        assert server.returncode == 0
    with sqlite3.connect(database.resolve().as_uri() + "?mode=ro", uri=True) as db:
        report["final_run"] = db.execute("SELECT run_id,services,flags,clears FROM tower_run").fetchall()
        report["final_milestones"] = [(floor, int.from_bytes(mask, "little")) for floor, mask in db.execute("SELECT floor_id,objectives FROM tower_objective_history")]
        report["final_floor_conquests"] = db.execute("SELECT floor_id FROM tower_floor_completion").fetchall()
    assert report["final_run"] == [(1, 2, 0, 0)]
    assert report["final_milestones"] == [(1, 2)]
    assert report["final_floor_conquests"] == []
    report["accepted"] = True
    (args.output / "results.private.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps({key: report[key] for key in ["accepted", "server_source", "client_source", "server_sha256", "client_sha256", "final_run", "final_milestones", "final_floor_conquests"]}))


if __name__ == "__main__":
    main()
