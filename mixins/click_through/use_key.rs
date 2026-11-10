// The use key is about to act on the crosshair. When the main hand aims at a
// decoration whose block behind answers a click, the click goes to that block
// instead: the server is sent the same packet a click on the block sends, so
// it opens the chest through its own path. Anything else is left to the game.
fn click_through_use_key(
    game: &mut crate::game::Game<'_>,
    button: i32,
    hand: horizon_client::first_person::Hand,
    canceled: &mut bool,
) {
    use crate::game::click_through as m;
    if button != 1
        || *canceled
        || !matches!(hand, horizon_client::first_person::Hand::Main)
        || !game.native_on(m::ID)
        || m::sneaking()
    {
        return;
    }
    let Some(state) = game.native_client_state() else {
        return;
    };
    let Some(eye) = state.eye_position else {
        return;
    };
    let dir = m::view_vector(state.yaw, state.pitch);

    // What the crosshair is on: the nearer of the first block and the first
    // hanging entity along the view.
    let block = game.native_raycast(m::BLOCK_REACH as f32).map(|hit| {
        let distance = (0..3)
            .map(|a| ([hit.x, hit.y, hit.z][a] - eye[a]).powi(2))
            .sum::<f64>()
            .sqrt();
        (hit, distance)
    });
    let hanging = m::hanging_hit(eye, dir, |id| {
        let entity = game.native_entity(u32::try_from(id).ok()?)?;
        game.native_entity_bounding_box(entity)
    })
    .filter(|(_, d)| block.as_ref().is_none_or(|(_, b)| d < b));

    let (decoration, facing, behind, distance) = match (hanging, block) {
        (Some((h, distance)), _) => {
            let point = m::along(eye, dir, distance);
            (h.kind, h.facing, m::block_behind_point(point, h.facing), distance)
        }
        (None, Some((hit, distance))) => {
            let Some(decoration) = m::wall_decoration(&hit.block) else {
                return;
            };
            let pos = [hit.block_x, hit.block_y, hit.block_z];
            let Some(facing) = game
                .native_block_property(pos, "facing")
                .and_then(|f| m::facing_offset(&f))
            else {
                return;
            };
            (decoration, facing, m::block_behind(pos, facing), distance)
        }
        (None, None) => return,
    };
    if !m::enabled(decoration, |key| game.native_setting(m::ID, key)) {
        return;
    }
    let held = game.native_held().map(|(item, _, _)| item);
    if !m::passes(decoration, &game.native_block(behind), held.as_deref()) {
        return;
    }
    // A mob or a player standing in front of the decoration takes the click.
    let blocked = game.native_client_entities().iter().any(|e| {
        e.alive && m::ray_box(eye, dir, e.bounds[0], e.bounds[1], distance).is_some()
    });
    if blocked {
        return;
    }
    let packet = horizon_client::mod_api::ServerboundPlayPacket::UseItemOn(
        horizon_protocol::packets::play::UseItemOn {
            hand: 0,
            pos: behind,
            face: m::face_id(facing),
            cursor: m::cursor(facing),
            inside: false,
            world_border: false,
            // Nothing is predicted on this side, so there is no change for
            // the server to acknowledge.
            sequence: 0,
        },
    );
    // Cancel the game's own click only once the redirected one is on its way.
    if game.native_send_packet(packet).is_ok() {
        *canceled = true;
    }
}
