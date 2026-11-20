// `player_model_geometry` starts: in first person, the patches below it act
// until it returns (`end.rs`), with the body moved back from the view.
fn first_person_body_begin(game: &Game<'_>, out: &Vec<horizon_client_render::render_geometry::Vertex>) {
    use mod_first_person_body::body;
    body::finish();
    if !body::shown() {
        return;
    }
    let crouching = game.player_model.as_ref().is_some_and(|p| p.extra.crouching);
    body::set_active(body::body_offset(&body::current(), game.yaw, game.pitch, crouching), out.len());
}
