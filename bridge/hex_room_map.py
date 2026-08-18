#!/usr/bin/env python3
"""The elephant reads the hex map.

The bridge between the Rust `HexRoomMap` (the MUD as an Eisenstein lattice)
and the elephant (the Python package that reads any room's field). The Rust
map exports itself as JSON (`cargo run --example hex_mud -- --json`); this
bridge turns every room back into something the elephant can read, runs the
REAL dial bank over it, and computes the same two map-level quantities the
Rust mirror computes:

    map_temperature(fields)   — the grid's aggregate field (mean warmth)
    deadband_ring(fields, …)  — the terrain's deadband: when the map's field
                                crosses a threshold (a panic spreading), the
                                ring names the region that crossed; on a
                                stable map, nothing rings.

If the elephant is importable (via `ELEPHANT_ROOT` env or the `../elephant`
sibling directory), its real `RoomField` and dials are used — each room with
`events` becomes a real elephant `Room` read by `DialBank(DEFAULT_DIALS)`.
Without the elephant, the bridge falls back to the mirrored readings the
Rust map carried (same warmth formula, same ring logic) — the documented
minimal fallback.

Usage:
    python3 bridge/hex_room_map.py --map bridge/example_map.json
    python3 bridge/hex_room_map.py --map bridge/example_map.json --threshold 0.5
"""
from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Dict, List, Optional, Tuple

# The six D₆ unit directions — the same set the Rust map uses (E12::directions).
HEX_DIRECTIONS = [(1, 0), (0, 1), (1, 1), (-1, 0), (0, -1), (-1, -1)]


def _load_elephant():
    """Import the elephant package; None when it is not importable."""
    root = os.environ.get("ELEPHANT_ROOT")
    candidates = []
    if root:
        candidates.append(Path(root))
    candidates.append(Path(__file__).resolve().parent.parent.parent / "elephant")
    for p in candidates:
        if (p / "elephant" / "__init__.py").is_file() and str(p) not in sys.path:
            sys.path.insert(0, str(p))
    try:
        from elephant.field import RoomField  # noqa: F401
        from elephant.dial import DialBank
        from elephant.dials import DEFAULT_DIALS
        from elephant.room import Message, Room
        from elephant.field import read_field

        return {"RoomField": RoomField, "DialBank": DialBank,
                "DEFAULT_DIALS": DEFAULT_DIALS, "Room": Room,
                "Message": Message, "read_field": read_field}
    except Exception:
        return None


ELEPHANT = _load_elephant()


# ---------------------------------------------------------------------- #
# Hex lattice math (mirror of the Rust side)                             #
# ---------------------------------------------------------------------- #
def hex_neighbors(coord: Tuple[int, int]) -> List[Tuple[int, int]]:
    """The six D₆ neighbors of a hex."""
    return [(coord[0] + da, coord[1] + db) for da, db in HEX_DIRECTIONS]


def hex_distance(a: Tuple[int, int], b: Tuple[int, int]) -> int:
    """True hex distance: the minimal number of D₆ unit steps."""
    da, db = a[0] - b[0], a[1] - b[1]
    return max(abs(da), abs(db), abs(da - db))


# ---------------------------------------------------------------------- #
# The field — the elephant's reading of a room                           #
# ---------------------------------------------------------------------- #
def mirror_warmth(readings: Dict[str, float]) -> float:
    """The Rust mirror's warmth — byte-for-byte the elephant's formula."""
    r = readings
    return (
        0.30 * r.get("mood", 0.0)
        + 0.15 * r.get("joke_landing", 0.0)
        + 0.10 * (r.get("earnestness", 0.5) - 0.5) * 2
        + 0.10 * (r.get("presence", 0.5) - 0.5) * 2
        + 0.10 * (r.get("volume", 0.5) - 0.5) * 2
        - 0.15 * r.get("cynicism", 0.5)
        - 0.10 * r.get("panic", 0.0)
    )


def room_fields(map_data: Dict) -> Dict[Tuple[int, int], object]:
    """Every room -> a field the elephant can read.

    Rooms with `events` become real elephant Rooms read by the real dial
    bank. Rooms without events fall back to the mirrored readings the Rust
    map exported (documented minimal fallback).
    """
    if ELEPHANT is None:
        # Documented fallback: mirror semantics, no elephant needed.
        return {
            tuple(r["coord"]): _MirrorField(r.get("field") or {})
            for r in map_data.get("rooms", [])
        }

    out = {}
    for r in map_data.get("rooms", []):
        coord = tuple(r["coord"])
        events = r.get("events")
        if events:
            room = ELEPHANT["Room"](r.get("name", f"room-{coord}"))
            for i, text in enumerate(events):
                room.messages.append(
                    ELEPHANT["Message"](author="[room]", text=str(text), ts=float(i) * 60.0)
                )
            field = ELEPHANT["read_field"](room, ELEPHANT["DialBank"](ELEPHANT["DEFAULT_DIALS"]))
        else:
            field = ELEPHANT["RoomField"](r.get("field") or {})
        out[coord] = field
    return out


class _MirrorField:
    """The documented fallback: the Rust mirror's semantics, no elephant."""

    def __init__(self, readings: Dict[str, float]):
        self.readings = dict(readings)
        self.panic = float(self.readings.get("panic", 0.0))

    def warmth(self) -> float:
        return mirror_warmth(self.readings)


def _warmth(field) -> float:
    return field.warmth()


def _panic(field) -> float:
    if isinstance(field, _MirrorField):
        return field.panic
    return float(field.readings.get("panic", 0.0))


# ---------------------------------------------------------------------- #
# The map's field and the deadband                                       #
# ---------------------------------------------------------------------- #
def map_temperature(fields: Dict[Tuple[int, int], object]) -> Optional[float]:
    """The grid's aggregate field: mean warmth over every read room."""
    if not fields:
        return None
    return sum(_warmth(f) for f in fields.values()) / len(fields)


def map_panic(fields: Dict[Tuple[int, int], object]) -> Optional[float]:
    """The grid's aggregate panic — the stampede sense over the whole map."""
    if not fields:
        return None
    return sum(_panic(f) for f in fields.values()) / len(fields)


def deadband_ring(fields: Dict[Tuple[int, int], object], map_field: float,
                  threshold: float) -> Optional[Dict]:
    """The terrain's deadband: ring when the map's field crosses the band.

    Mirrors the Rust `HexRoomMap::deadband_ring` exactly: quiet below the
    threshold; when crossed, the ring names the largest connected region of
    read rooms whose own panic also crossed; if no single room crossed but
    the aggregate did, the ring names every read room.
    """
    if threshold < 0 or abs(map_field) < threshold:
        return None
    if not fields:
        return None

    best: List[Tuple[int, int]] = []
    seen = set()
    for coord, field in fields.items():
        if _panic(field) < threshold or coord in seen:
            continue
        comp = []
        stack = [coord]
        seen.add(coord)
        while stack:
            c = stack.pop()
            comp.append(c)
            for n in hex_neighbors(c):
                if n in seen:
                    continue
                f = fields.get(n)
                if f is not None and _panic(f) >= threshold:
                    seen.add(n)
                    stack.append(n)
        if len(comp) > len(best):
            best = comp

    coords = best or list(fields.keys())
    return {
        "coords": coords,
        "center": coords[0],
        "map_field": map_field,
        "threshold": threshold,
    }


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--map", required=True, help="HexRoomMap JSON export")
    ap.add_argument("--threshold", type=float, default=0.5, help="deadband threshold")
    args = ap.parse_args()

    map_data = json.loads(Path(args.map).read_text())
    fields = room_fields(map_data)
    temp = map_temperature(fields)
    mean_panic = map_panic(fields)
    # The ring trigger: the map's field. Mean panic gets diluted by calm
    # rooms, so the CLI rings on the PEAK reading — the elephant feeling the
    # loudest room, the war's hottest hex. (The Rust API takes any aggregate;
    # the caller chooses.)
    peak_panic = max((_panic(f) for f in fields.values()), default=0.0)
    print(f"map: {map_data.get('map', '?')}  ({len(fields)} rooms read)")
    print(f"map temperature: {temp if temp is not None else float('nan'):+.3f}  "
          f"(aggregate panic {mean_panic if mean_panic is not None else float('nan'):.3f}, "
          f"peak {peak_panic:.3f})")

    ring = deadband_ring(fields, peak_panic, args.threshold)
    if ring is None:
        print(f"deadband: quiet (threshold {args.threshold:g})")
        return 0
    names = []
    for c in ring["coords"]:
        for r in map_data.get("rooms", []):
            if tuple(r["coord"]) == c:
                names.append(r.get("name", f"room-{c}"))
    print(f"⚡ THE DEADBAND RINGS — {len(ring['coords'])} rooms crossed the band:")
    for n in names:
        print(f"    - {n}")
    print(f"  ring center: {ring['center']}  "
          f"(map field {ring['map_field']:.2f} ≥ threshold {ring['threshold']:.2f})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
