// SPDX-License-Identifier: MIT
// The mouse buttons on SunnyF3's settings card.
fn sunnyf3_card_options_mouse(
    game: &mut crate::game::Game<'_>,
    pressed: bool,
    right: bool,
) -> ::core::ops::ControlFlow<Result<bool>> {
    if !crate::game::sunnyf3_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::sunnyf3_card::mouse(game, pressed, right).map(|()| true))
}
