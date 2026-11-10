// The mod's settings page: one button for each decoration.
fn passthrough_page(game: &crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::passthrough as m;
    if mod_id != m::ID {
        return;
    }
    for decoration in m::Decoration::ALL {
        let on = m::enabled(decoration, |key| game.native_setting(m::ID, key));
        buttons.push(m::label(decoration, on));
    }
}
