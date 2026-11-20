// Once a frame: the settings key opens SunnyF3's page.
fn sunnyf3_frame(game: &mut crate::game::Game<'_>) {
    crate::game::sunnyf3_hud::frame(game);
}
