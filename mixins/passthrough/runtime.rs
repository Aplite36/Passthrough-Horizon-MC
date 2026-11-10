// Passthrough: a right-click on a wall sign, wall banner, item frame or
// painting goes to the block it hangs on. This file holds the rules, the
// geometry and the mod's state. It is both the Cargo library the tests run
// against and the module the bundle adds to the game, so nothing here names a
// type of the game: the helper files next to it do that.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

/// The mod's id (its settings folder), name and version.
pub const ID: &str = "passthrough";
pub const NAME: &str = "Passthrough";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Right-click through wall signs, wall banners, item frames and paintings to use the block behind them. Sneak to click the decoration itself.";

/// How far a use reaches a block and an entity in survival. Creative reaches
/// further (5 blocks for both); past these the game's own click runs.
pub const BLOCK_REACH: f64 = 4.5;
pub const ENTITY_REACH: f64 = 3.0;

/// What a click can pass through.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decoration {
    Sign,
    Banner,
    Frame,
    Painting,
}

impl Decoration {
    pub const ALL: [Decoration; 4] = [
        Decoration::Sign,
        Decoration::Banner,
        Decoration::Frame,
        Decoration::Painting,
    ];

    /// The name of its setting file.
    pub fn key(self) -> &'static str {
        match self {
            Decoration::Sign => "signs",
            Decoration::Banner => "banners",
            Decoration::Frame => "item_frames",
            Decoration::Painting => "paintings",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Decoration::Sign => "Wall signs",
            Decoration::Banner => "Wall banners",
            Decoration::Frame => "Item frames",
            Decoration::Painting => "Paintings",
        }
    }
}

// Settings: each read once from its file, then kept. 0 is not read yet.
const UNREAD: u8 = 0;
const ON: u8 = 1;
const OFF: u8 = 2;
static ENABLED: [AtomicU8; 4] = [
    AtomicU8::new(UNREAD),
    AtomicU8::new(UNREAD),
    AtomicU8::new(UNREAD),
    AtomicU8::new(UNREAD),
];

/// A stored setting: on unless the file says `false`.
pub fn read_flag(stored: Option<String>) -> bool {
    stored.is_none_or(|v| v.trim() != "false")
}

/// Whether clicks pass through `decoration`, read once through `get`.
pub fn enabled(decoration: Decoration, get: impl FnOnce(&str) -> Option<String>) -> bool {
    let slot = &ENABLED[decoration as usize];
    match slot.load(Ordering::Relaxed) {
        ON => true,
        OFF => false,
        _ => {
            let on = read_flag(get(decoration.key()));
            slot.store(if on { ON } else { OFF }, Ordering::Relaxed);
            on
        }
    }
}

pub fn set_enabled(decoration: Decoration, on: bool) {
    ENABLED[decoration as usize].store(if on { ON } else { OFF }, Ordering::Relaxed);
}

/// A settings button: `Wall signs: ON`.
pub fn label(decoration: Decoration, on: bool) -> String {
    format!("{}: {}", decoration.title(), if on { "ON" } else { "OFF" })
}

/// The sneak key, as the last movement input read it.
static SNEAKING: AtomicBool = AtomicBool::new(false);

pub fn set_sneaking(sneaking: bool) {
    SNEAKING.store(sneaking, Ordering::Relaxed);
}

pub fn sneaking() -> bool {
    SNEAKING.load(Ordering::Relaxed)
}

/// The unit step of a block state's `facing` value.
pub fn facing_offset(facing: &str) -> Option<[i32; 3]> {
    Some(match facing {
        "down" => [0, -1, 0],
        "up" => [0, 1, 0],
        "north" => [0, 0, -1],
        "south" => [0, 0, 1],
        "west" => [-1, 0, 0],
        "east" => [1, 0, 0],
        _ => return None,
    })
}

/// The unit step of a direction's protocol id (`Direction.get3DDataValue`).
pub fn direction_offset(id: i32) -> Option<[i32; 3]> {
    facing_offset(match id {
        0 => "down",
        1 => "up",
        2 => "north",
        3 => "south",
        4 => "west",
        5 => "east",
        _ => return None,
    })
}

/// The protocol id of a unit step.
pub fn face_id(offset: [i32; 3]) -> i32 {
    match offset {
        [0, -1, 0] => 0,
        [0, 0, -1] => 2,
        [0, 0, 1] => 3,
        [-1, 0, 0] => 4,
        [1, 0, 0] => 5,
        _ => 1,
    }
}

/// A vanilla block or item id without its namespace. Other namespaces give
/// None: the mod only knows vanilla's.
fn vanilla(id: &str) -> Option<&str> {
    match id.split_once(':') {
        Some(("minecraft", name)) => Some(name),
        Some(_) => None,
        None => Some(id),
    }
}

/// The decoration a block is, for the ones that hang on a wall. Standing
/// signs, standing banners and hanging signs are not.
pub fn wall_decoration(block: &str) -> Option<Decoration> {
    let name = vanilla(block)?;
    if name.ends_with("_wall_sign") {
        Some(Decoration::Sign)
    } else if name.ends_with("_wall_banner") {
        Some(Decoration::Banner)
    } else {
        None
    }
}

/// The decoration an entity type is.
pub fn hanging_kind(entity: &str) -> Option<Decoration> {
    match vanilla(entity)? {
        "item_frame" | "glow_item_frame" => Some(Decoration::Frame),
        "painting" => Some(Decoration::Painting),
        _ => None,
    }
}

/// How a block answers a right-click that is not a sneak.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Use {
    /// Nothing happens with an empty hand: stone, air, a cauldron.
    Never,
    /// It opens or toggles whatever is held: a chest, a door, a lever.
    Always,
    /// It reacts to an empty hand, and differently to some items: a jukebox
    /// gives its disc back, or takes the one held.
    EmptyHand,
}

const ALWAYS: [&str; 26] = [
    "barrel",
    "beacon",
    "bell",
    "blast_furnace",
    "brewing_stand",
    "cartography_table",
    "chest",
    "comparator",
    "crafter",
    "crafting_table",
    "daylight_detector",
    "dispenser",
    "dropper",
    "enchanting_table",
    "ender_chest",
    "furnace",
    "grindstone",
    "hopper",
    "lever",
    "loom",
    "repeater",
    "smithing_table",
    "smoker",
    "stonecutter",
    "anvil",
    "trapped_chest",
];
const ALWAYS_SUFFIX: [&str; 8] = [
    "_chest",
    "shulker_box",
    "_button",
    "_door",
    "_trapdoor",
    "_fence_gate",
    "_bed",
    "_anvil",
];
/// Redstone opens these, a hand does not.
const NEVER: [&str; 2] = ["iron_door", "iron_trapdoor"];
const EMPTY_HAND: [&str; 10] = [
    "cake",
    "cave_vines",
    "cave_vines_plant",
    "chiseled_bookshelf",
    "composter",
    "decorated_pot",
    "flower_pot",
    "jukebox",
    "lectern",
    "note_block",
];
const EMPTY_HAND_SUFFIX: [&str; 2] = ["candle_cake", "_shelf"];

pub fn block_use(block: &str) -> Use {
    let Some(name) = vanilla(block) else {
        return Use::Never;
    };
    if NEVER.contains(&name) {
        return Use::Never;
    }
    if ALWAYS.contains(&name) || ALWAYS_SUFFIX.iter().any(|s| name.ends_with(s)) {
        return Use::Always;
    }
    if EMPTY_HAND.contains(&name)
        || name.starts_with("potted_")
        || EMPTY_HAND_SUFFIX.iter().any(|s| name.ends_with(s))
    {
        return Use::EmptyHand;
    }
    Use::Never
}

/// An item a sign takes: dye, an ink sac to glow or stop glowing, wax.
pub fn edits_sign(item: &str) -> bool {
    vanilla(item).is_some_and(|name| {
        name.ends_with("_dye") || matches!(name, "glow_ink_sac" | "ink_sac" | "honeycomb")
    })
}

/// Whether a click on `decoration` goes to the block `behind` it, with `held`
/// in the main hand. The server uses what is held on the block when the block
/// lets it, so a block that reacts to items only takes an empty hand's click:
/// the held item is never used.
pub fn passes(decoration: Decoration, behind: &str, held: Option<&str>) -> bool {
    if decoration == Decoration::Sign && held.is_some_and(edits_sign) {
        return false;
    }
    match block_use(behind) {
        Use::Always => true,
        Use::EmptyHand => held.is_none(),
        Use::Never => false,
    }
}

/// The unit vector a player looks along (`Entity.calculateViewVector`): yaw 0
/// faces south, pitch 90 looks down.
pub fn view_vector(yaw: f32, pitch: f32) -> [f64; 3] {
    let (yaw, pitch) = (f64::from(yaw).to_radians(), f64::from(pitch).to_radians());
    [
        -yaw.sin() * pitch.cos(),
        -pitch.sin(),
        yaw.cos() * pitch.cos(),
    ]
}

/// How far along `dir` from `eye` the ray enters the box, within `reach`. An
/// eye inside the box is at distance 0.
pub fn ray_box(
    eye: [f64; 3],
    dir: [f64; 3],
    lo: [f64; 3],
    hi: [f64; 3],
    reach: f64,
) -> Option<f64> {
    let (mut near, mut far) = (0.0f64, reach);
    for a in 0..3 {
        if dir[a].abs() < 1e-9 {
            if eye[a] < lo[a] || eye[a] > hi[a] {
                return None;
            }
            continue;
        }
        let (mut t0, mut t1) = ((lo[a] - eye[a]) / dir[a], (hi[a] - eye[a]) / dir[a]);
        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }
        near = near.max(t0);
        far = far.min(t1);
        if near > far {
            return None;
        }
    }
    Some(near)
}

/// The point `distance` along the ray.
pub fn along(eye: [f64; 3], dir: [f64; 3], distance: f64) -> [f64; 3] {
    [0, 1, 2].map(|a| eye[a] + dir[a] * distance)
}

/// The block a hanging entity is mounted on, from the point of its box the
/// click met: half a block against its facing. A painting spans several
/// blocks, and this picks the one behind the point clicked.
pub fn block_behind_point(point: [f64; 3], facing: [i32; 3]) -> [i32; 3] {
    [0, 1, 2].map(|a| (point[a] - f64::from(facing[a]) * 0.5).floor() as i32)
}

/// The block a wall block is mounted on.
pub fn block_behind(pos: [i32; 3], facing: [i32; 3]) -> [i32; 3] {
    [0, 1, 2].map(|a| pos[a] - facing[a])
}

/// Where the click lands on the block behind: the middle of the face the
/// decoration hangs on, in the block's own coordinates.
pub fn cursor(facing: [i32; 3]) -> [f32; 3] {
    facing.map(|f| 0.5 + 0.5 * f as f32)
}

/// A hanging entity the server showed this client.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hanging {
    pub id: i32,
    pub kind: Decoration,
    pub facing: [i32; 3],
    pub position: [f64; 3],
}

/// What the client knows of the server's hanging entities. `kinds` remembers
/// which entity type ids are decorations on this server, so each type is
/// named once.
struct Tracked {
    kinds: Vec<(i32, Option<Decoration>)>,
    hanging: Vec<Hanging>,
}

static TRACKED: Mutex<Tracked> = Mutex::new(Tracked {
    kinds: Vec::new(),
    hanging: Vec::new(),
});

fn tracked() -> std::sync::MutexGuard<'static, Tracked> {
    // A poisoned lock still holds plain data: keep going.
    TRACKED.lock().unwrap_or_else(|e| e.into_inner())
}

/// The server added an entity: keep it if its type is a decoration. `name`
/// names the type id, and is asked once for each id.
pub fn track(
    id: i32,
    kind_id: i32,
    position: [f64; 3],
    data: i32,
    name: impl FnOnce() -> Option<String>,
) {
    let mut t = tracked();
    let kind = match t.kinds.iter().find(|(k, _)| *k == kind_id) {
        Some((_, kind)) => *kind,
        None => {
            // An id that can't be named yet is asked again next time.
            let Some(name) = name() else { return };
            let kind = hanging_kind(&name);
            t.kinds.push((kind_id, kind));
            kind
        }
    };
    let (Some(kind), Some(facing)) = (kind, direction_offset(data)) else {
        return;
    };
    t.hanging.retain(|h| h.id != id);
    t.hanging.push(Hanging {
        id,
        kind,
        facing,
        position,
    });
}

/// The server removed entities.
pub fn forget(ids: &[i32]) {
    let mut t = tracked();
    if !t.hanging.is_empty() {
        t.hanging.retain(|h| !ids.contains(&h.id));
    }
}

/// A new level: its entities are all new.
pub fn forget_all() {
    tracked().hanging.clear();
}

/// A new server: its entity type ids may differ too.
pub fn reset() {
    let mut t = tracked();
    t.hanging.clear();
    t.kinds.clear();
}

/// The hanging entities whose position is within `range` of `eye`.
pub fn hanging_near(eye: [f64; 3], range: f64) -> Vec<Hanging> {
    tracked()
        .hanging
        .iter()
        .filter(|h| {
            (0..3)
                .map(|a| (h.position[a] - eye[a]).powi(2))
                .sum::<f64>()
                <= range * range
        })
        .copied()
        .collect()
}

/// The nearest hanging entity the ray meets within reach, and how far. `bounds`
/// gives an entity's box, or None for one that is gone.
pub fn hanging_hit(
    eye: [f64; 3],
    dir: [f64; 3],
    mut bounds: impl FnMut(i32) -> Option<[[f64; 3]; 2]>,
) -> Option<(Hanging, f64)> {
    // A painting's middle is at most a few blocks from the point clicked.
    let mut best: Option<(Hanging, f64)> = None;
    for h in hanging_near(eye, ENTITY_REACH + 4.0) {
        let Some([lo, hi]) = bounds(h.id) else {
            continue;
        };
        if let Some(d) = ray_box(eye, dir, lo, hi, ENTITY_REACH)
            && best.is_none_or(|(_, b)| d < b)
        {
            best = Some((h, d));
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_blocks_are_decorations_and_standing_ones_are_not() {
        assert_eq!(
            wall_decoration("minecraft:oak_wall_sign"),
            Some(Decoration::Sign)
        );
        assert_eq!(
            wall_decoration("minecraft:pale_oak_wall_sign"),
            Some(Decoration::Sign)
        );
        assert_eq!(
            wall_decoration("minecraft:red_wall_banner"),
            Some(Decoration::Banner)
        );
        assert_eq!(wall_decoration("minecraft:oak_sign"), None);
        assert_eq!(wall_decoration("minecraft:red_banner"), None);
        assert_eq!(wall_decoration("minecraft:oak_wall_hanging_sign"), None);
        assert_eq!(wall_decoration("minecraft:oak_hanging_sign"), None);
        assert_eq!(wall_decoration("othermod:oak_wall_sign"), None);
    }

    #[test]
    fn frames_and_paintings_are_hanging_decorations() {
        assert_eq!(
            hanging_kind("minecraft:item_frame"),
            Some(Decoration::Frame)
        );
        assert_eq!(
            hanging_kind("minecraft:glow_item_frame"),
            Some(Decoration::Frame)
        );
        assert_eq!(
            hanging_kind("minecraft:painting"),
            Some(Decoration::Painting)
        );
        assert_eq!(hanging_kind("minecraft:armor_stand"), None);
        assert_eq!(hanging_kind("minecraft:leash_knot"), None);
    }

    #[test]
    fn blocks_answer_a_click_by_their_kind() {
        for block in [
            "minecraft:chest",
            "minecraft:trapped_chest",
            "minecraft:waxed_oxidized_copper_chest",
            "minecraft:barrel",
            "minecraft:red_shulker_box",
            "minecraft:shulker_box",
            "minecraft:furnace",
            "minecraft:oak_door",
            "minecraft:copper_trapdoor",
            "minecraft:stone_button",
            "minecraft:lever",
            "minecraft:chipped_anvil",
            "minecraft:white_bed",
            "minecraft:oak_fence_gate",
        ] {
            assert_eq!(block_use(block), Use::Always, "{block}");
        }
        for block in [
            "minecraft:air",
            "minecraft:stone",
            "minecraft:dirt",
            "minecraft:iron_door",
            "minecraft:iron_trapdoor",
            "minecraft:cauldron",
            "minecraft:water_cauldron",
            "minecraft:respawn_anchor",
            "minecraft:oak_fence",
            "minecraft:bedrock",
            "minecraft:bookshelf",
            "othermod:chest",
        ] {
            assert_eq!(block_use(block), Use::Never, "{block}");
        }
        for block in [
            "minecraft:jukebox",
            "minecraft:lectern",
            "minecraft:composter",
            "minecraft:flower_pot",
            "minecraft:potted_poppy",
            "minecraft:cake",
            "minecraft:candle_cake",
            "minecraft:red_candle_cake",
            "minecraft:chiseled_bookshelf",
            "minecraft:oak_shelf",
            "minecraft:note_block",
        ] {
            assert_eq!(block_use(block), Use::EmptyHand, "{block}");
        }
    }

    #[test]
    fn a_click_passes_to_a_block_that_answers_it() {
        let chest = "minecraft:chest";
        for d in Decoration::ALL {
            assert!(passes(d, chest, None));
            assert!(passes(d, chest, Some("minecraft:dirt")));
            assert!(!passes(d, "minecraft:stone", None));
            assert!(!passes(d, "minecraft:air", None));
        }
    }

    #[test]
    fn a_sign_keeps_the_click_of_an_item_that_edits_it() {
        for item in [
            "minecraft:red_dye",
            "minecraft:glow_ink_sac",
            "minecraft:ink_sac",
            "minecraft:honeycomb",
        ] {
            assert!(
                !passes(Decoration::Sign, "minecraft:chest", Some(item)),
                "{item}"
            );
            // Banners, frames and paintings have no such exception.
            assert!(
                passes(Decoration::Banner, "minecraft:chest", Some(item)),
                "{item}"
            );
            assert!(
                passes(Decoration::Frame, "minecraft:chest", Some(item)),
                "{item}"
            );
        }
        assert!(passes(
            Decoration::Sign,
            "minecraft:chest",
            Some("minecraft:stick")
        ));
    }

    #[test]
    fn a_block_that_takes_items_only_gets_an_empty_hand() {
        assert!(passes(Decoration::Frame, "minecraft:jukebox", None));
        assert!(!passes(
            Decoration::Frame,
            "minecraft:jukebox",
            Some("minecraft:music_disc_cat")
        ));
        assert!(!passes(
            Decoration::Sign,
            "minecraft:flower_pot",
            Some("minecraft:poppy")
        ));
    }

    #[test]
    fn directions_round_trip() {
        for id in 0..6 {
            let offset = direction_offset(id).unwrap();
            assert_eq!(face_id(offset), id);
            assert_eq!(offset.iter().map(|v| v.abs()).sum::<i32>(), 1);
        }
        assert_eq!(direction_offset(6), None);
        assert_eq!(direction_offset(-1), None);
        assert_eq!(facing_offset("north"), Some([0, 0, -1]));
        assert_eq!(facing_offset("sideways"), None);
    }

    #[test]
    fn the_block_behind_is_one_step_against_the_facing() {
        // A sign facing north hangs on the block to its south.
        assert_eq!(block_behind([10, 64, 5], [0, 0, -1]), [10, 64, 6]);
        assert_eq!(block_behind([-3, 70, -8], [1, 0, 0]), [-4, 70, -8]);
        // An item frame on the east face of block (4, 64, 9): its box sits
        // just past x = 5.
        assert_eq!(block_behind_point([5.03, 64.5, 9.5], [1, 0, 0]), [4, 64, 9]);
        // On the west face of block (-1, 64, 0): just before x = -1.
        assert_eq!(
            block_behind_point([-1.03, 64.5, 0.5], [-1, 0, 0]),
            [-1, 64, 0]
        );
        // A frame lying on top of a block.
        assert_eq!(block_behind_point([0.5, 65.03, 0.5], [0, 1, 0]), [0, 64, 0]);
        // A wide painting: the point clicked picks the block.
        assert_eq!(block_behind_point([7.9, 64.5, 3.03], [0, 0, 1]), [7, 64, 2]);
        assert_eq!(block_behind_point([8.1, 64.5, 3.03], [0, 0, 1]), [8, 64, 2]);
    }

    #[test]
    fn the_click_lands_on_the_face_the_decoration_hangs_on() {
        assert_eq!(cursor([0, 0, -1]), [0.5, 0.5, 0.0]);
        assert_eq!(cursor([1, 0, 0]), [1.0, 0.5, 0.5]);
        assert_eq!(cursor([0, 1, 0]), [0.5, 1.0, 0.5]);
    }

    fn close(a: [f64; 3], b: [f64; 3]) -> bool {
        (0..3).all(|i| (a[i] - b[i]).abs() < 1e-6)
    }

    #[test]
    fn the_view_vector_follows_the_games_angles() {
        assert!(close(view_vector(0.0, 0.0), [0.0, 0.0, 1.0]));
        assert!(close(view_vector(90.0, 0.0), [-1.0, 0.0, 0.0]));
        assert!(close(view_vector(180.0, 0.0), [0.0, 0.0, -1.0]));
        assert!(close(view_vector(-90.0, 0.0), [1.0, 0.0, 0.0]));
        assert!(close(view_vector(0.0, 90.0), [0.0, -1.0, 0.0]));
        assert!(close(view_vector(0.0, -90.0), [0.0, 1.0, 0.0]));
    }

    #[test]
    fn a_ray_meets_a_box_within_reach() {
        let (lo, hi) = ([0.0, 0.0, 2.0], [1.0, 1.0, 2.0625]);
        let eye = [0.5, 0.5, 0.0];
        assert_eq!(ray_box(eye, [0.0, 0.0, 1.0], lo, hi, 3.0), Some(2.0));
        // Too far, looking away, and beside it.
        assert_eq!(ray_box(eye, [0.0, 0.0, 1.0], lo, hi, 1.5), None);
        assert_eq!(ray_box(eye, [0.0, 0.0, -1.0], lo, hi, 3.0), None);
        assert_eq!(ray_box([1.5, 0.5, 0.0], [0.0, 0.0, 1.0], lo, hi, 3.0), None);
        // From inside.
        assert_eq!(
            ray_box([0.5, 0.5, 2.03], [0.0, 0.0, 1.0], lo, hi, 3.0),
            Some(0.0)
        );
        assert!(close(along(eye, [0.0, 0.0, 1.0], 2.0), [0.5, 0.5, 2.0]));
    }

    #[test]
    fn settings_default_on_and_keep_what_was_read() {
        assert!(read_flag(None));
        assert!(read_flag(Some("true".into())));
        assert!(read_flag(Some("anything".into())));
        assert!(!read_flag(Some("false\n".into())));
        // Paintings: no other test reads this slot.
        let d = Decoration::Painting;
        assert!(!enabled(d, |key| {
            assert_eq!(key, "paintings");
            Some("false".into())
        }));
        assert!(!enabled(d, |_| panic!("read twice")));
        set_enabled(d, true);
        assert!(enabled(d, |_| panic!("read after a set")));
        assert_eq!(label(d, true), "Paintings: ON");
        assert_eq!(label(Decoration::Frame, false), "Item frames: OFF");
    }

    #[test]
    fn sneaking_is_kept() {
        set_sneaking(true);
        assert!(sneaking());
        set_sneaking(false);
        assert!(!sneaking());
    }

    /// The table is one static: this test is the only one that uses it.
    #[test]
    fn the_table_follows_the_servers_entities() {
        reset();
        let name = |id: i32| {
            move || {
                Some(
                    match id {
                        7 => "minecraft:item_frame",
                        8 => "minecraft:painting",
                        _ => "minecraft:zombie",
                    }
                    .to_owned(),
                )
            }
        };
        // A frame on the north face of block (0, 64, 3), a painting, a mob.
        track(1, 7, [0.5, 64.5, 2.97], 2, name(7));
        track(2, 8, [20.5, 64.5, 2.97], 2, name(8));
        track(3, 9, [0.5, 64.0, 1.0], 0, name(9));
        // Known ids are not named again.
        track(4, 7, [1.5, 64.5, 2.97], 2, || panic!("named twice"));
        track(5, 9, [0.5, 64.0, 1.0], 0, || panic!("named twice"));
        // A direction that isn't one is left out.
        track(6, 7, [2.5, 64.5, 2.97], 9, || panic!("named twice"));
        // An id that can't be named yet is asked again.
        track(10, 11, [0.5, 64.5, 2.97], 2, || None);
        track(10, 11, [0.5, 64.5, 2.97], 2, || {
            Some("minecraft:glow_item_frame".into())
        });

        let eye = [0.5, 64.5, 0.5];
        let near = hanging_near(eye, 7.0);
        assert_eq!(near.iter().map(|h| h.id).collect::<Vec<_>>(), [1, 4, 10]);
        assert_eq!(near[0].kind, Decoration::Frame);
        assert_eq!(near[0].facing, [0, 0, -1]);
        assert_eq!(hanging_near(eye, 30.0).len(), 4);

        // Looking south at frame 1, whose box the game gives.
        let frame = |id: i32| match id {
            1 | 10 => Some([[0.125, 64.125, 2.9375], [0.875, 64.875, 3.0]]),
            4 => Some([[1.125, 64.125, 2.9375], [1.875, 64.875, 3.0]]),
            _ => None,
        };
        let (hit, distance) = hanging_hit(eye, [0.0, 0.0, 1.0], frame).unwrap();
        assert_eq!(hit.id, 1);
        assert!((distance - 2.4375).abs() < 1e-9);
        let point = along(eye, [0.0, 0.0, 1.0], distance);
        assert_eq!(block_behind_point(point, hit.facing), [0, 64, 3]);
        // Looking away, and at an entity that is gone.
        assert!(hanging_hit(eye, [0.0, 0.0, -1.0], frame).is_none());
        assert!(hanging_hit(eye, [0.0, 0.0, 1.0], |_| None).is_none());

        // A re-added id replaces the old entry.
        track(1, 7, [0.5, 70.5, 2.97], 3, || panic!("named twice"));
        let near = hanging_near([0.5, 70.5, 0.5], 3.0);
        assert_eq!(near.len(), 1);
        assert_eq!(near[0].facing, [0, 0, 1]);

        forget(&[1, 4]);
        assert_eq!(
            hanging_near(eye, 100.0)
                .iter()
                .map(|h| h.id)
                .collect::<Vec<_>>(),
            [2, 10]
        );
        forget_all();
        assert!(hanging_near(eye, 100.0).is_empty());
        // The type ids outlive a level change, not a new server.
        track(1, 7, [0.5, 64.5, 2.97], 2, || panic!("named twice"));
        reset();
        let mut asked = false;
        track(1, 7, [0.5, 64.5, 2.97], 2, || {
            asked = true;
            Some("minecraft:item_frame".into())
        });
        assert!(asked);
        reset();
    }
}
