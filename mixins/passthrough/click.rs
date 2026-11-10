// A click on the mod's settings page: turn that decoration on or off, and keep it.
fn passthrough_click(game: &crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::passthrough as m;
    if mod_id != m::ID {
        return;
    }
    let Some(&decoration) = m::Decoration::ALL.get(button) else {
        return;
    };
    let on = !m::enabled(decoration, |key| game.native_setting(m::ID, key));
    m::set_enabled(decoration, on);
    game.native_set_setting(m::ID, decoration.key(), if on { "true" } else { "false" });
}
