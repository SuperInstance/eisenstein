"""The elephant reads the hex map — bridge tests.

These run the Python bridge (the real-elephant seam) against map exports in
the Rust crate's JSON shape. They skip cleanly when the elephant package is
not importable (the bridge's documented fallback still works; these tests
just exercise the real seam).

Run:  python3 -m pytest bridge/test_hex_room_map.py -v
"""
import math

import pytest

from hex_room_map import (
    ELEPHANT,
    deadband_ring,
    front_direction,
    hex_distance,
    hex_neighbors,
    map_panic,
    map_temperature,
    mirror_warmth,
    room_fields,
)

pytestmark = pytest.mark.skipif(
    ELEPHANT is None, reason="elephant package not importable — bridge fallback only"
)


def test_hex_geometry_matches_the_rust_map():
    # The same known layouts the Rust integration tests assert.
    cases = [
        ((0, 0), (0, 0), 0),
        ((0, 0), (1, 0), 1),
        ((0, 0), (0, 1), 1),
        ((0, 0), (1, 1), 1),   # a D₆ neighbor
        ((0, 0), (-1, -1), 1),
        ((0, 0), (1, -1), 2),
        ((0, 0), (2, -1), 3),
        ((0, 0), (3, -2), 5),
    ]
    for a, b, expect in cases:
        assert hex_distance(a, b) == expect, f"{a} -> {b}"
    # Six distinct neighbors, all one step away.
    ns = hex_neighbors((0, 0))
    assert len(ns) == len(set(ns)) == 6
    for n in ns:
        assert hex_distance((0, 0), n) == 1


def test_mirror_warmth_matches_the_rust_formula():
    # Neutral readings -> -0.075 (the elephant's cynicism baseline).
    neutral = dict(mood=0.0, volume=0.5, earnestness=0.5, cynicism=0.5,
                   joke_landing=0.0, panic=0.0, presence=0.5)
    assert mirror_warmth(neutral) == pytest.approx(-0.075)
    # The Rust example's warm Tap: mood 0.6, joke 0.4 -> 0.18 + 0.06 - 0.075.
    tap = dict(neutral, mood=0.6, joke_landing=0.4)
    assert mirror_warmth(tap) == pytest.approx(0.165)
    # Panic is cold: -0.1 per unit.
    assert mirror_warmth(dict(neutral, panic=0.9)) == pytest.approx(-0.165)


def test_real_dials_read_an_actual_room():
    # A room where the panic dial has something to feel: alarm words, urgency,
    # a cascade. The REAL elephant dials must cross the deadband threshold.
    map_data = {
        "map": "test",
        "rooms": [
            {"coord": [0, 0], "name": "The Tap",
             "events": ["A fire! Fire in the kitchen!",
                        "Everyone out, NOW!",
                        "Run! The ceiling is coming down!",
                        "All hands, evacuate the building!"]},
            {"coord": [1, 0], "name": "the Docks",
             "events": ["The barrels are safe here.",
                        "A quiet night on the water."]},
        ],
    }
    fields = room_fields(map_data)
    assert len(fields) == 2
    # The elephant actually read the rooms: the Tap is panicking, the Docks
    # are not.
    tap_field = fields[(0, 0)]
    docks_field = fields[(1, 0)]
    panic_tap = tap_field.readings.get("panic", 0.0)
    panic_docks = docks_field.readings.get("panic", 0.0)
    assert panic_tap >= 0.5, f"real PanicDial should spike, got {panic_tap:.2f}"
    assert panic_docks < 0.5, f"calm room should stay calm, got {panic_docks:.2f}"

    # The map's aggregate field and the deadband ring. The trigger is the
    # peak reading — the elephant feeling the loudest room — and the ring
    # names the region whose own panic crossed.
    temp = map_temperature(fields)
    assert temp is not None and math.isfinite(temp)
    assert map_panic(fields) == pytest.approx((panic_tap + panic_docks) / 2.0)
    ring = deadband_ring(fields, max(panic_tap, panic_docks), 0.5)
    assert ring is not None, "crossing the deadband must ring"
    assert (0, 0) in ring["coords"], "the burning room is named"
    assert (1, 0) not in ring["coords"], "the calm room is not named"


def test_mirror_fallback_rings_on_spreading_panic_stays_quiet_when_stable():
    # Mirror-readings map: a district that catches fire, a calm district.
    def room(c, name, panic):
        return {"coord": list(c), "name": name,
                "field": {"mood": 0.0, "volume": 0.5, "earnestness": 0.5,
                          "cynicism": 0.5, "joke_landing": 0.0,
                          "panic": panic, "presence": 0.5}}

    district = [(0, 0), (1, 0), (0, 1), (1, 1), (2, 0), (2, 1), (1, -1)]
    temple = [(6, 0), (7, 0), (6, 1), (7, 1), (6, -1), (7, -1), (8, 0)]
    rooms = [room(c, f"tavern-{c}", 0.1) for c in district]
    rooms += [room(c, f"temple-{c}", 0.05) for c in temple]
    map_data = {"map": "town", "rooms": rooms}
    fields = room_fields(map_data)

    # Stable: quiet.
    assert deadband_ring(fields, 0.05, 0.5) is None
    assert deadband_ring(fields, 0.49, 0.5) is None

    # The district catches fire.
    for c in district:
        for r in map_data["rooms"]:
            if tuple(r["coord"]) == c:
                r["field"]["panic"] = 0.9
    fields = room_fields(map_data)
    assert map_panic(fields) == pytest.approx((7 * 0.9 + 7 * 0.05) / 14.0)
    ring = deadband_ring(fields, 0.8, 0.5)  # the aggregate crossed the band
    assert ring is not None
    assert set(ring["coords"]) == set(district), "the ring names the burning district"
    assert ring["center"] == (0, 0)

    # A lone panicking room far away is a smaller region: the ring still
    # names the district, not the hermit.
    rooms.append(room((12, 0), "lonely-hermit", 0.95))
    map_data["rooms"] = rooms
    fields = room_fields(map_data)
    ring2 = deadband_ring(fields, 0.85, 0.5)
    assert ring2 is not None
    assert set(ring2["coords"]) == set(district)


def test_aggregate_crosses_but_no_room_does():
    # The whole map warms gently past the deadband: no single room crosses,
    # but the aggregate does — the ring names every read room.
    map_data = {
        "map": "warm",
        "rooms": [
            {"coord": [k, 0], "name": f"warm-{k}",
             "field": {"mood": 0.0, "volume": 0.5, "earnestness": 0.5,
                       "cynicism": 0.5, "joke_landing": 0.0,
                       "panic": 0.1, "presence": 0.5}}
            for k in range(3)
        ],
    }
    fields = room_fields(map_data)
    ring = deadband_ring(fields, 0.6, 0.5)
    assert ring is not None
    assert len(ring["coords"]) == 3


def test_front_direction_agrees_with_the_rust_map():
    # The same cases the Rust unit test asserts — both sides snap the
    # centroid displacement to the same D₆ unit, by the same exact formula.
    assert front_direction([(0, 0)], [(0, 0), (1, 0)]) == (1, 0)        # east: 1
    assert front_direction([(0, 0)], [(0, 0), (0, 1)]) == (0, 1)        # ω
    assert front_direction([(1, 0)], [(1, 0), (0, 0)]) == (-1, 0)       # −1
    assert front_direction([(0, 0)], [(0, 0), (-1, -1)]) == (-1, -1)    # ω²
    assert front_direction([(0, 0), (1, 0)],
                           [(0, 0), (1, 0), (2, 0)]) == (1, 0)
    # The turn north-east mid-march: displacement (3,3) -> the unit 1+ω.
    assert front_direction([(0, 0), (1, 0), (2, 0)],
                           [(0, 0), (1, 0), (2, 0), (2, 1)]) == (1, 1)
    # Settled or empty: no front.
    assert front_direction([(0, 0), (1, 0)], [(0, 0), (1, 0)]) is None
    assert front_direction([], [(0, 0)]) is None
    assert front_direction([(0, 0)], []) is None


def test_ring_names_the_region_and_the_front_of_a_migrating_fight():
    # The montage: a fight seeded at one hex, migrating hex-by-hex. The Rust
    # map remembers its last blaze internally; the bridge takes both frames
    # explicitly (`previous`) — same quantities, same numbers.
    def room(c, panic):
        return {"coord": list(c), "name": f"town-{c[0]}{c[1]}",
                "field": {"mood": 0.0, "volume": 0.5, "earnestness": 0.5,
                          "cynicism": 0.5, "joke_landing": 0.0,
                          "panic": panic, "presence": 0.5}}

    calm = lambda: {"map": "town", "rooms": [room(c, 0.05) for c in [
        (0, 0), (1, 0), (2, 0), (2, 1), (0, 1), (-1, 0)]]}

    def fields_with(panics):
        data = calm()
        for c, p in panics.items():
            for r in data["rooms"]:
                if tuple(r["coord"]) == c:
                    r["field"]["panic"] = p
        return room_fields(data)

    # Frame 1: seeded at one hex. First frame of the montage: no front.
    fields = fields_with({(0, 0): 0.9})
    r1 = deadband_ring(fields, 0.9, 0.5)
    assert r1["coords"] == [(0, 0)]
    assert r1["front"] is None

    # Frame 2: propagates east — the ring names the connected region and
    # the front is the D₆ unit 1.
    fields = fields_with({(0, 0): 0.9, (1, 0): 0.9})
    r2 = deadband_ring(fields, 0.9, 0.5, previous=r1["coords"])
    assert set(r2["coords"]) == {(0, 0), (1, 0)}
    assert r2["front"] == (1, 0)

    # Frames 3-4: keeps moving east, then turns north-east — the front
    # follows, exactly as the Rust integration test asserts.
    fields = fields_with({(0, 0): 0.9, (1, 0): 0.9, (2, 0): 0.9})
    r3 = deadband_ring(fields, 0.9, 0.5, previous=r2["coords"])
    assert len(r3["coords"]) == 3
    assert r3["front"] == (1, 0)

    fields = fields_with({(0, 0): 0.9, (1, 0): 0.9, (2, 0): 0.9, (2, 1): 0.9})
    r4 = deadband_ring(fields, 0.9, 0.5, previous=r3["coords"])
    assert len(r4["coords"]) == 4
    assert r4["front"] == (1, 1)

    # Settled: a standing fire is not a montage.
    r5 = deadband_ring(fields, 0.9, 0.5, previous=r4["coords"])
    assert r5["front"] is None
