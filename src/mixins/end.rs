// `player_model_geometry` returns: the first-person model's vertices move
// back by the body offset, and the patches below it stop acting.
fn first_person_body_end(value: (), out: &mut Vec<horizon_client_render::render_geometry::Vertex>) {
    use mod_first_person_body::body;
    if let Some((offset, start)) = body::finish() {
        for vertex in out.iter_mut().skip(start) {
            vertex.pos = body::moved(vertex.pos, offset);
        }
    }
    value
}
