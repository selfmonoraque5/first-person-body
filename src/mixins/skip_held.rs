// `held_item_geometry`: with the vanilla hand, the model's own hands hold
// nothing (the hand already shows the items).
fn first_person_body_skip_held() -> ::core::ops::ControlFlow<()> {
    use mod_first_person_body::body;
    if body::active() && body::arms_hidden(&body::current()) {
        return ::core::ops::ControlFlow::Break(());
    }
    ::core::ops::ControlFlow::Continue(())
}
