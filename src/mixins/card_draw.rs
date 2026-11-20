// SPDX-License-Identifier: MIT
// SunnyF3's settings page: its card is drawn instead of Sunrise's generic page.
fn sunnyf3_card_options_draw(
    game: &mut crate::game::Game<'_>,
    batch: &mut sunrise_client::hud::HudBatch,
) -> ::core::ops::ControlFlow<()> {
    if !crate::game::sunnyf3_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    crate::game::sunnyf3_card::draw(game, batch);
    ::core::ops::ControlFlow::Break(())
}
