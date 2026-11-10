// The sneak key as this tick's movement read it.
fn passthrough_input(sneak: bool) {
    crate::game::passthrough::set_sneaking(sneak);
}
