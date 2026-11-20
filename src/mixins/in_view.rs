// `own_body_in_view`: in first person, the body is drawn too, while the
// player stands or crouches (not swimming, crawling, gliding or spinning),
// isn't a spectator and, unless the settings say otherwise, isn't riding.
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
