// While F3 is open, SunnyF3 draws the overlay in place of the game's text.
fn sunnyf3_overlay(
    game: &crate::game::Game<'_>,
    batch: &mut sunrise_client::hud::HudBatch,
    size: [u32; 2],
) -> ::core::ops::ControlFlow<()> {
    if crate::game::sunnyf3_hud::draw(game, batch, size) {
        ::core::ops::ControlFlow::Break(())
    } else {
        ::core::ops::ControlFlow::Continue(())
    }
}
