// SPDX-License-Identifier: MIT
// The wheel on SunnyF3's settings card.
fn sunnyf3_card_options_scroll(
    game: &mut crate::game::Game<'_>,
    amount: f32,
) -> ::core::ops::ControlFlow<Result<()>> {
    if !crate::game::sunnyf3_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::sunnyf3_card::scroll(game, amount))
}
