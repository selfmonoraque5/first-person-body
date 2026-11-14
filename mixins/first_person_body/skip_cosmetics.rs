// Horizon's cosmetics and other layers drawn on your own model (hats, wings...):
// not on the own first-person model, a hat would sit over the camera.
fn first_person_body_skip_cosmetics() -> core::ops::ControlFlow<()> {
    if horizon_client::mobs::first_person_body::active().is_some() {
        return core::ops::ControlFlow::Break(());
    }
    core::ops::ControlFlow::Continue(())
}
