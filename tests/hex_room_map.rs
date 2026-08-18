//! HexRoomMap integration tests — the MUD as an Eisenstein lattice.
//!
//! These verify the map layer end-to-end through the public API:
//! - every hex has exactly 6 D₆ neighbors
//! - distance is the true hex distance (matches known layouts, exact)
//! - paths are valid and adjacent
//! - regions are hex disks of the right size (3R² + 3R + 1)
//! - the map temperature reads the grid's aggregate field
//! - the terrain's deadband rings on a spreading panic and stays quiet on a
//!   stable map

use eisenstein::{hex_directions, hex_distance, HexRoomMap, MapError, Ring, RoomField, E12};

#[cfg(test)]
mod tests {
    use super::*;

    /// A neutral room (all readings at the elephant's neutral points) that is
    /// calm or panicking: warmth = -0.1 * panic, so a stable map reads ~0.
    fn field(panic: f64) -> RoomField {
        RoomField::new(0.0, 0.5, 0.5, 0.5, 0.0, panic, 0.5)
    }

    #[test]
    fn six_neighbors_per_hex() {
        let m = HexRoomMap::new();
        // The origin's neighbors are exactly the D₆ unit directions.
        let ns = m.neighbors((0, 0)).unwrap();
        assert_eq!(ns.len(), 6);
        assert_eq!(ns, hex_directions());
        // Every hex, wherever it sits, has six distinct neighbors at distance 1.
        for coord in [(0, 0), (7, -3), (-42, 99), (1, 1)] {
            let ns = m.neighbors(coord).unwrap();
            assert_eq!(ns.len(), 6, "neighbors of {:?}", coord);
            for (i, n) in ns.iter().enumerate() {
                assert_eq!(
                    hex_distance(coord, *n),
                    Some(1),
                    "neighbor {} of {:?} must be one hex step away",
                    i,
                    coord
                );
                for (j, o) in ns.iter().enumerate() {
                    if i != j {
                        assert_ne!(n, o);
                    }
                }
            }
        }
        // The D₆ directions are exactly the six units of Z[ω]: the same six
        // vectors the crate's symmetry code produces.
        let dirs = E12::directions();
        for (i, d) in dirs.iter().enumerate() {
            assert_eq!(
                ns_of_origin_as_e12()[i],
                *d,
                "map directions must be the crate's D₆ units"
            );
        }
    }

    fn ns_of_origin_as_e12() -> [E12; 6] {
        let m = HexRoomMap::new();
        let ns = m.neighbors((0, 0)).unwrap();
        [
            E12::new(ns[0].0 as i32, ns[0].1 as i32),
            E12::new(ns[1].0 as i32, ns[1].1 as i32),
            E12::new(ns[2].0 as i32, ns[2].1 as i32),
            E12::new(ns[3].0 as i32, ns[3].1 as i32),
            E12::new(ns[4].0 as i32, ns[4].1 as i32),
            E12::new(ns[5].0 as i32, ns[5].1 as i32),
        ]
    }

    #[test]
    fn distance_matches_hex_layouts() {
        let m = HexRoomMap::new();
        // Known axial-layout distances on the Eisenstein lattice: the minimal
        // number of D₆ unit steps, exactly.
        type DistCase = ((i64, i64), (i64, i64), u64);
        let cases: &[DistCase] = &[
            ((0, 0), (0, 0), 0),
            ((0, 0), (1, 0), 1),
            ((0, 0), (0, 1), 1),
            ((0, 0), (1, 1), 1),   // a D₆ neighbor (1+ω = -ω²)
            ((0, 0), (-1, -1), 1), // its reflection
            ((0, 0), (1, -1), 2),
            ((0, 0), (2, -1), 3),
            ((0, 0), (3, -2), 5),
            ((0, 0), (4, 4), 4),
            ((5, -3), (5, -3), 0),
        ];
        for (a, b, expect) in cases {
            assert_eq!(
                m.distance(*a, *b),
                Some(*expect),
                "hex distance {:?} -> {:?}",
                a,
                b
            );
        }
        // Symmetric and translation-invariant.
        let (a, b) = ((3, -2), (9, 4));
        assert_eq!(m.distance(a, b), m.distance(b, a));
        assert_eq!(
            m.distance((1, 1), (7, 5)),
            m.distance((0, 0), (6, 4)),
            "translation invariance"
        );
        // The map's distance is the lattice-correct one. The crate's own
        // E12::hex_distance uses the axial (|q|+|r|+|q+r|)/2 convention,
        // which reports its own neighbor (1,1) at distance 2 and (3,-2) at 3
        // — inconsistent with E12::directions(). The map metric reports the
        // true step counts: neighbors at 1, (3,-2) at max(3,2,5) = 5.
        assert_eq!(m.distance((0, 0), (3, -2)), Some(5));
        assert_eq!(
            E12::new(3, -2).hex_distance_to(E12::new(0, 0)),
            3,
            "crate axial convention"
        );
    }

    #[test]
    fn path_is_valid_and_adjacent() {
        let mut m = HexRoomMap::new();
        // A 19-room disk: three rings of a small town.
        for c in m.region((0, 0), 2).unwrap() {
            m.add_room(c, &format!("town{}{}", c.0, c.1)).unwrap();
        }
        // A distant keep, connected by a winding road of single hexes.
        let road = [(2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (6, 1), (6, 2)];
        for (k, c) in road.iter().enumerate() {
            m.add_room(*c, &format!("road{}", k)).unwrap();
        }
        m.add_room((7, 2), "keep").unwrap();

        // Corner to corner across the town: valid, adjacent, room-only.
        let p = m.path((-2, 0), (2, 2)).unwrap();
        assert_eq!(p.first(), Some(&(-2, 0)));
        assert_eq!(p.last(), Some(&(2, 2)));
        for w in p.windows(2) {
            assert_eq!(m.distance(w[0], w[1]), Some(1), "adjacent step {:?}", w);
            assert!(
                m.contains(w[0]) && m.contains(w[1]),
                "path walks rooms only"
            );
        }
        // Town to keep, along the road: every step is one hex.
        let p2 = m.path((0, 0), (7, 2)).unwrap();
        assert_eq!(p2.first(), Some(&(0, 0)));
        assert_eq!(p2.last(), Some(&(7, 2)));
        assert_eq!(p2.len(), 8, "shortest road walk: town center -> keep");
        for w in p2.windows(2) {
            assert_eq!(m.distance(w[0], w[1]), Some(1));
        }
        // Disconnected island: no path, the war can't reach it.
        m.add_room((100, 100), "island").unwrap();
        assert_eq!(m.path((0, 0), (100, 100)), None);
        // Missing endpoints are None, self-path is the single hex.
        assert_eq!(m.path((0, 0), (500, 500)), None);
        assert_eq!(m.path((500, 500), (0, 0)), None);
        assert_eq!(m.path((0, 0), (0, 0)), Some(vec![(0, 0)]));
    }

    #[test]
    fn region_is_a_hex_disk_of_the_right_size() {
        let m = HexRoomMap::new();
        for r in 0u32..=6 {
            let disk = m.region((0, 0), r).unwrap();
            assert_eq!(
                disk.len(),
                (3 * r * r + 3 * r + 1) as usize,
                "3R²+3R+1 at radius {}",
                r
            );
            for p in &disk {
                assert!(hex_distance((0, 0), *p).unwrap() <= r as u64);
            }
        }
        // The radius-1 disk is exactly the center plus its six neighbors.
        let d1 = m.region((0, 0), 1).unwrap();
        assert_eq!(d1.len(), 7);
        assert!(d1.contains(&(0, 0)));
        for n in m.neighbors((0, 0)).unwrap() {
            assert!(d1.contains(&n));
        }
        // Same count at any center (the lattice is homogeneous).
        assert_eq!(m.region((-17, 33), 4).unwrap().len(), 61);
    }

    #[test]
    fn map_temperature_reads() {
        let mut m = HexRoomMap::new();
        assert_eq!(
            m.map_temperature(),
            None,
            "an unread map has no temperature"
        );
        for c in m.region((0, 0), 1).unwrap() {
            m.add_room(c, &format!("room{}{}", c.0, c.1)).unwrap();
        }
        // Warm the Tap (mood +0.6), chill one room (cynicism 1.0), leave the
        // rest neutral-but-calm.
        let warm = RoomField::new(0.6, 0.5, 0.5, 0.5, 0.0, 0.0, 0.5);
        let chill = RoomField::new(0.0, 0.5, 0.5, 1.0, 0.0, 0.0, 0.5);
        m.set_field((0, 0), warm).unwrap();
        m.set_field((1, 0), chill).unwrap();
        for c in m.region((0, 0), 1).unwrap() {
            if c != (0, 0) && c != (1, 0) {
                m.set_field(c, field(0.0)).unwrap();
            }
        }
        let t = m.map_temperature().unwrap();
        let expected = (warm.warmth() + chill.warmth() + 5.0 * field(0.0).warmth()) / 7.0;
        assert!(
            (t - expected).abs() < 1e-12,
            "temperature {} vs {}",
            t,
            expected
        );
        // The grid's aggregate panic climbs as the panic spreads.
        assert_eq!(m.map_panic().unwrap(), 0.0);
        m.set_field((0, 0), field(0.9)).unwrap();
        assert!((m.map_panic().unwrap() - 0.9 / 7.0).abs() < 1e-12);
    }

    #[test]
    fn deadband_rings_on_spreading_panic_and_stays_quiet_on_stable_map() {
        let mut m = HexRoomMap::new();
        // Two districts and a hermit.
        for c in m.region((0, 0), 1).unwrap() {
            m.add_room(c, &format!("tavern-{}{}", c.0, c.1)).unwrap();
        }
        for c in m.region((6, 0), 1).unwrap() {
            m.add_room(c, &format!("temple-{}{}", c.0, c.1)).unwrap();
        }
        m.add_room((12, 0), "lonely-hermit").unwrap();

        // Stable map: quiet fields, quiet aggregate. Nothing rings.
        for c in m.region((0, 0), 1).unwrap() {
            m.set_field(c, field(0.1)).unwrap();
        }
        for c in m.region((6, 0), 1).unwrap() {
            m.set_field(c, field(0.05)).unwrap();
        }
        m.set_field((12, 0), field(0.0)).unwrap();
        assert!(
            m.deadband_ring(0.05, 0.5).is_none(),
            "stable map stays quiet"
        );
        assert!(
            m.deadband_ring(0.49, 0.5).is_none(),
            "below the band stays quiet"
        );

        // The panic spreads through the tavern district. The deadband rings,
        // and the ring names exactly the region the war has reached.
        for c in m.region((0, 0), 1).unwrap() {
            m.set_field(c, field(0.9)).unwrap();
        }
        let ring: Ring = m
            .deadband_ring(0.8, 0.5)
            .expect("crossing the deadband must ring");
        assert!(ring.map_field >= ring.threshold);
        assert_eq!(ring.coords.len(), 7);
        assert!(ring.region.iter().all(|n| n.starts_with("tavern-")));
        assert!(ring.coords.contains(&(0, 0)), "the Tap is in the ring");

        // A lone panicking hermit is a smaller region than the burning
        // district: the ring keeps naming the war, not the hermitage.
        m.set_field((12, 0), field(0.95)).unwrap();
        let ring2 = m.deadband_ring(0.85, 0.5).unwrap();
        assert_eq!(ring2.coords.len(), 7);
        assert!(ring2.region.iter().all(|n| n.starts_with("tavern-")));

        // A threshold the field does not cross: quiet again.
        assert!(m.deadband_ring(0.8, 0.9).is_none());
    }

    #[test]
    fn ring_names_the_region_and_the_front_of_a_migrating_fight() {
        // The montage: a fight seeded at one hex, migrating hex-by-hex. Each
        // ring names the connected region it has reached AND the front — the
        // D₆ unit the region moved along since the previous ring.
        let mut m = HexRoomMap::new();
        for c in m.region((0, 0), 3).unwrap() {
            m.add_room(c, &format!("town-{}{}", c.0, c.1)).unwrap();
            m.set_field(c, field(0.05)).unwrap(); // a calm town
        }

        // Seed the panic at one hex; the ring names it. First frame of the
        // montage: no front (a fresh blaze has no history to move against).
        m.set_field((0, 0), field(0.9)).unwrap();
        let r1: Ring = m
            .deadband_ring(0.9, 0.5)
            .expect("the seed crosses the band");
        assert_eq!(r1.coords, vec![(0, 0)]);
        assert_eq!(r1.region, vec!["town-00"]);
        assert_eq!(r1.front, None);

        // Propagate to the neighbors, east first: the ring names the grown
        // connected region and the front is the D₆ unit 1 (east).
        m.set_field((1, 0), field(0.9)).unwrap();
        let r2 = m.deadband_ring(0.9, 0.5).unwrap();
        assert_eq!(r2.coords.len(), 2);
        assert!(r2.coords.contains(&(0, 0)) && r2.coords.contains(&(1, 0)));
        assert!(r2.region.iter().all(|n| n.starts_with("town-")));
        assert_eq!(r2.front, Some((1, 0)), "the front points east");
        assert!(hex_directions().contains(&r2.front.unwrap()));

        // The fight keeps migrating east, then turns north-east: the front
        // follows — 1, then 1+ω.
        m.set_field((2, 0), field(0.9)).unwrap();
        let r3 = m.deadband_ring(0.9, 0.5).unwrap();
        assert_eq!(r3.coords.len(), 3);
        assert_eq!(r3.front, Some((1, 0)));

        m.set_field((2, 1), field(0.9)).unwrap();
        let r4 = m.deadband_ring(0.9, 0.5).unwrap();
        assert_eq!(r4.coords.len(), 4);
        assert_eq!(r4.front, Some((1, 1)), "the front turns with the fight");

        // A re-ring over an unchanged region: no front — a standing fire is
        // not a montage.
        let r5 = m.deadband_ring(0.9, 0.5).unwrap();
        assert_eq!(r5.front, None);

        // Quiet frame — the fires die down — and the montage memory resets,
        // so a new seed starts a new sequence, frontless.
        assert!(m.deadband_ring(0.4, 0.5).is_none());
        for c in [(0, 0), (1, 0), (2, 0), (2, 1)] {
            m.set_field(c, field(0.05)).unwrap();
        }
        assert!(m.deadband_ring(0.4, 0.5).is_none(), "the town calms down");
        m.set_field((-3, 0), field(0.9)).unwrap();
        let r6 = m.deadband_ring(0.9, 0.5).unwrap();
        assert_eq!(r6.coords, vec![(-3, 0)]);
        assert_eq!(r6.front, None, "a new montage starts frontless");
    }

    #[test]
    fn guards_reject_impossible_coordinates() {
        let mut m = HexRoomMap::new();
        // Coordinates beyond the crate's i32 E12 lattice are impossible.
        assert_eq!(
            m.add_room((1i64 << 40, 0), "far"),
            Err(MapError::OutOfLatticeRange)
        );
        assert_eq!(
            m.add_room((0, -3_000_000_000), "far"),
            Err(MapError::OutOfLatticeRange)
        );
        // Nameless rooms are not rooms.
        assert_eq!(m.add_room((0, 0), ""), Err(MapError::UnnamedRoom));
        // Reading a hex with no room on it is an error.
        assert_eq!(m.set_field((0, 0), field(0.0)), Err(MapError::RoomNotFound));
        // Adding a room, overwriting it, then reading it works.
        m.add_room((0, 0), "tap").unwrap();
        m.add_room((0, 0), "the-tap").unwrap();
        assert_eq!(m.get((0, 0)), Some("the-tap"));
        assert!(m.contains((0, 0)));
        assert_eq!(m.room_count(), 1);
    }
}
