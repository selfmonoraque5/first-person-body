// `own_body_in_view`: in first person, the body is drawn too, while the
// player stands or crouches (not swimming, crawling, gliding or spinning),
// isn't a spectator and, unless the settings say otherwise, isn't riding.
//
// The game reads `own_body_in_view` in more than one place (game.rs, the
// world's entity pass, and player_hooks.rs). While this returns true: the own
// model is drawn in the main pass instead of only a shader pack's shadow pass
// (the point of the mod; shadows are unchanged), the flames are drawn on the
// body while it burns, and other mods' first-person-only geometry
// (`players_after_geometry`) is left out, as in third person.
fn first_person_body_in_view(game: &Game<'_>, value: bool) -> bool {
    use mod_first_person_body::body;
    if value {
        // Third person, asleep or a free camera: the game draws the body itself.
        body::set_shown(false);
        return value;
    }
    let Some(model) = game.player_model.as_ref() else {
        body::set_shown(false);
        return value;
    };
    let extra = &model.extra;
    let upright = matches!(
        extra.avatar_pose,
        crate::pose::Pose::Standing | crate::pose::Pose::Crouching
    );
    let shows = body::body_shows(
        &body::current(),
        upright,
        extra.fall_flying,
        extra.spectator,
        extra.passenger,
    );
    body::set_shown(shows);
    shows
}
