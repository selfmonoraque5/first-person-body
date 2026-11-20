// What would sit at the camera while the own first-person model is drawn: a
// pumpkin or skull worn, deadmau5's ears, Horizon's cosmetics (hats, wings).
fn first_person_body_skip_at_camera() -> ::core::ops::ControlFlow<()> {
    if mod_first_person_body::body::active() {
        return ::core::ops::ControlFlow::Break(());
    }
    ::core::ops::ControlFlow::Continue(())
}
