# Passthrough

A native mod for [Horizon](https://playhorizon.dev), the Rust client for Minecraft: Java Edition 26.3.

Right-click a wall sign, wall banner, item frame or painting, and the click goes to the block it hangs on. A sign on a chest opens the chest.

## What it does

- Only the main hand's right-click changes. Sneak and right-click to use the decoration itself: edit the sign, rotate the framed item.
- The block behind is the one the decoration is mounted on. If that block does nothing when right-clicked (stone, dirt, air), the mod does nothing and the decoration behaves as usual.
- When a click passes through, the decoration doesn't react. The sign editor stays shut and a framed item doesn't rotate.
- Holding dye, an ink sac, a glow ink sac or honeycomb, a click on a sign still reaches the sign, so you can colour and wax it.
- Standing signs, standing banners and hanging signs are left alone.
- Each of the four decorations has its own switch in Options > Mods > Passthrough > Settings. All four start on.

## Where it works

It is a client mod. It works in single player and on any server, with nothing installed on the server. The mod sends the server the same packet a click on the block behind would send, so the server opens the chest through its normal path.

### The held item is never used

The server decides what a click does, and it lets the block answer before the item in your hand. So the rule depends on the block behind:

| Block behind | Passes through |
|---|---|
| Opens or toggles whatever you hold: chests, barrels, shulker boxes, furnaces, hoppers, crafting stations, doors, trapdoors, fence gates, buttons, levers, beds | Always |
| Reacts to an empty hand, and differently to some items: jukebox, lectern, composter, flower pots, cakes, chiseled bookshelf, shelves, decorated pot, note block | Only with an empty main hand |
| Does nothing with an empty hand | Never |

Blocks from other mods are not in the list yet, so a click on a decoration in front of one behaves as it does without this mod.

### Servers and anti-cheat

A vanilla server accepts the redirected click: it checks how far the block is, not whether you can see it. Some servers run stricter plugins or have rules against clients that change clicks. Turn the mod off in Options > Mods on servers that forbid it.

## Limits

- Reach is 4.5 blocks for signs and banners and 3 blocks for frames and paintings, the survival values. Creative players get the game's normal click past those distances.
- A boat, minecart or armor stand standing between you and a decoration is not seen by the mod. Mobs and players are, and they take the click as usual.
- Item frames and paintings that were already loaded when you turned the mod on are picked up the next time the server sends them, for example after you walk away and back or rejoin.

## Install

Once the mod is listed in Horizon's registry, install it from the launcher's Mods page, then press Build. Native mods are compiled into the game by the launcher. Until then, download the `.hnmod` from this repository's Releases page.

To build a release file yourself:

```bash
git clone https://github.com/horizon-rust/horizon-mods
```

```bash
python3 horizon-mods/native/tools/native.py package /path/to/Passthrough-Horizon-MC
```

Releases are signed with [minisign](https://jedisct1.github.io/minisign/). The public key is in `horizon-mods.pub`:

```bash
minisign -V -p horizon-mods.pub -m passthrough-0.1.0.hnmod
```

## How it is built

- `mixins/passthrough/runtime.rs` holds the rules, the geometry and the mod's state, with its tests. It names no type of the game, so `cargo test` runs it alone.
- The other `.rs` files next to it are the hooks. Each is one short function.
- `mixins/passthrough/horizon-mixin.toml` names the hook points: `interaction_key` (the use key), `client_packet` (which frames and paintings exist), `movement_input` (the sneak key), and the three for the Mods list and settings page.

### Cost

- A use-key press runs one raycast and, near item frames or paintings, one box test for each within 7 blocks.
- Each server packet costs one `match`. Only the four packet kinds the mod reads do more.
- Each tick stores the sneak key in one atomic. Nothing else runs per tick or per frame.

Two helpers the mod calls, `native_registry_name` and `native_entity` on a client, are in Horizon's source but have no hook API number yet. The weekly build against Horizon's newest Nightly catches a change to either.

## Develop

```bash
cargo test
```

```bash
cargo clippy --all-targets -- -D warnings
```

```bash
python3 horizon-mods/native/tools/native.py check /path/to/Passthrough-Horizon-MC
```

The last command applies the mod to Horizon's source at the commit in `compatibility.json` and compile-checks the whole game.

## Origin

This is an independent implementation written for Horizon from a description of the behaviour. It contains no code from any other mod.

## Licence

[MIT](LICENSE).
