// The mod's row in Options > Mods.
fn click_through_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::click_through as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
