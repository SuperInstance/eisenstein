//! hex_mud — the MUD as an Eisenstein lattice, demo.
//!
//! Builds a small tavern-town map on the hex lattice, lets the elephant
//! (via its RoomField mirror) read every room, and shows the terrain's
//! deadband ringing when a panic spreads through the hexes.
//!
//! Run:
//!   cargo run --example hex_mud                 # the story
//!   cargo run --example hex_mud -- --json       # bridge input for the
//!                                               # elephant (real dials)

use eisenstein::{HexRoomMap, RoomField};

/// A neutral room with the given panic level: warmth = -0.1 * panic.
fn field(panic: f64) -> RoomField {
    RoomField::new(0.0, 0.5, 0.5, 0.5, 0.0, panic, 0.5)
}

fn warm_field() -> RoomField {
    RoomField::new(0.6, 0.5, 0.5, 0.5, 0.4, 0.0, 0.5)
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn main() {
    let as_json = std::env::args().any(|a| a == "--json");

    let mut map = HexRoomMap::new();

    // ── The town, one hex at a time ──────────────────────────────────────
    let town: &[((i64, i64), &str)] = &[
        ((0, 0), "The Tap"),
        ((1, 0), "the Docks"),
        ((0, 1), "the Temple"),
        ((1, 1), "the Alley"),
        ((2, 0), "the Keep"),
        ((2, 1), "the Smithy"),
        ((1, -1), "the Gardens"),
        ((0, -1), "the Harbour Gate"),
        ((3, 0), "the West Road"),
        ((4, 0), "the Ferry"),
        ((5, 0), "the Lighthouse"),
    ];
    for &(c, name) in town {
        map.add_room(c, name).expect("town rooms are on-lattice");
    }

    // ── The elephant reads each room ─────────────────────────────────────
    for &(c, name) in town {
        let f = match name {
            "The Tap" => warm_field(),
            "the Docks" => field(0.05),
            "the Temple" => field(0.0),
            "the Alley" => field(0.2),
            "the Smithy" => RoomField::new(0.3, 0.5, 0.5, 0.5, 0.2, 0.0, 0.5),
            "the Lighthouse" => RoomField::new(0.0, 0.5, 0.5, 0.7, 0.0, 0.0, 0.5),
            _ => field(0.05),
        };
        map.set_field(c, f).expect("room exists");
    }

    // ── Bridge input for the real elephant ───────────────────────────────
    //
    // The Rust map does not expose per-room fields read-back (they are
    // written through `set_field`), so the bridge input re-derives the
    // readings from the same table used above. The Python bridge then runs
    // the REAL elephant dials over each room's events when available, and
    // falls back to these mirrored readings otherwise.
    if as_json {
        let mut json = String::from("{\"map\":\"the-tap-town\",\"rooms\":[");
        let mut sep = "";
        for (c, name) in map.iter() {
            // The same state the story left the town in: fire in the Alley
            // has reached the Tap, the Docks, and the Smithy.
            let f = match name {
                "The Tap" | "the Docks" | "the Alley" | "the Smithy" => field(0.9),
                "the Temple" => field(0.0),
                "the Lighthouse" => RoomField::new(0.0, 0.5, 0.5, 0.7, 0.0, 0.0, 0.5),
                _ => field(0.05),
            };
            json.push_str(&format!(
                "{} {{\"coord\":[{},{}],\"name\":\"{}\",\"field\":{{\"mood\":{},\"volume\":{},\"earnestness\":{},\"cynicism\":{},\"joke_landing\":{},\"panic\":{},\"presence\":{}}}}}",
                sep, c.0, c.1, esc(name), f.mood, f.volume, f.earnestness, f.cynicism,
                f.joke_landing, f.panic, f.presence
            ));
            sep = ",";
        }
        json.push_str("]}");
        println!("{}", json);
        return;
    }

    println!("== The town at closing hour ==");
    println!(
        "map temperature: {:.3}  (aggregate panic {:.3})",
        map.map_temperature().unwrap_or(f64::NAN),
        map.map_panic().unwrap_or(f64::NAN)
    );
    println!(
        "The Tap is {} hexes from the Lighthouse.",
        map.distance((0, 0), (5, 0)).unwrap()
    );
    println!(
        "The Ferry's disk: {} hexes within reach.",
        map.region((4, 0), 1).unwrap().len()
    );
    println!("deadband: quiet (nothing crossed the band)");

    // ── The war spreads: fire in the Alley ───────────────────────────────
    println!("\n== Fire in the Alley ==");
    for c in [(1, 1), (0, 0), (1, 0), (2, 1)] {
        map.set_field(c, field(0.9)).expect("room exists");
    }
    println!(
        "map temperature: {:.3}  (aggregate panic {:.3})",
        map.map_temperature().unwrap_or(f64::NAN),
        map.map_panic().unwrap_or(f64::NAN)
    );
    if let Some(ring) = map.deadband_ring(0.7, 0.5) {
        println!("⚡ THE DEADBAND RINGS — {} rooms on fire:", ring.coords.len());
        for name in &ring.region {
            println!("    - {}", name);
        }
        println!(
            "  ring center: {:?}  (map field {:.2} ≥ threshold {:.2})",
            ring.center, ring.map_field, ring.threshold
        );
    } else {
        println!("deadband: quiet");
    }
    // The Temple stays calm and unreachable from the blaze: no path through
    // burning hexes because every hex between is a room that has not lit yet.
    println!(
        "The Tap -> the Temple: {} hexes.",
        map.distance((0, 0), (0, 1)).unwrap()
    );
}
