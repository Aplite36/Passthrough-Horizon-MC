// The server's play packets, read only: the item frames and paintings it adds
// and removes, so a click can tell when the crosshair is on one. Every packet
// comes through here, so the match is first and the rest only runs for these four.
fn click_through_packet(
    game: &crate::game::Game<'_>,
    packet: &horizon_client::mod_api::ClientboundPlayPacket,
) {
    use crate::game::click_through as m;
    use horizon_client::mod_api::ClientboundPlayPacket as P;
    match packet {
        P::AddEntity(e) => {
            // Off, the mod keeps nothing: what it knew may be stale by the
            // time it is turned on again.
            if !game.native_on(m::ID) {
                m::forget_all();
                return;
            }
            m::track(e.id, e.kind, e.position, e.data, || {
                game.native_registry_name("minecraft:entity_type", e.kind)
            });
        }
        P::RemoveEntities(r) => m::forget(&r.ids),
        P::Respawn(_) => m::forget_all(),
        P::Login(_) => m::reset(),
        _ => {}
    }
}
