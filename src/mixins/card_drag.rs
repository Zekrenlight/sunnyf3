// SPDX-License-Identifier: MIT
// Dragging on SunnyF3's settings card: its sliders and scrollbar.
fn sunnyf3_card_options_drag(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<Result<()>> {
    if !crate::game::sunnyf3_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::sunnyf3_card::drag(game))
}
