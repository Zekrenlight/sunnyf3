// SPDX-License-Identifier: MIT
// Keys on SunnyF3's settings card; Escape (search let go) is Sunrise's, to leave.
fn sunnyf3_card_options_key(
    game: &mut crate::game::Game<'_>,
    key: KeyCode,
    text: Option<&str>,
) -> ::core::ops::ControlFlow<Result<()>> {
    if !crate::game::sunnyf3_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    match crate::game::sunnyf3_card::key(game, key, text) {
        Some(done) => ::core::ops::ControlFlow::Break(done),
        None => ::core::ops::ControlFlow::Continue(()),
    }
}
