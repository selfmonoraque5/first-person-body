// The worn helmet while the own first-person model is drawn, if the player left
// it hidden: it would sit around the camera. A layer whose drawn pieces are all
// the head's (the helmet's model, its trim and overlays) is left out; the rest
// of the armour is drawn as usual, and third person and the inventory's preview
// keep the helmet.
fn first_person_body_skip_helmet(
    value: Vec<(&Part, [f32; 4], u32, u32, bool)>,
) -> Vec<(&Part, [f32; 4], u32, u32, bool)> {
    use mod_first_person_body::body;
    if !(body::active() && body::helmet_hidden(&body::current())) {
        return value;
    }
    let head_only = |part: &Part| {
        let mut parts = vec![(part, false)];
        let mut drawn = false;
        while let Some((p, in_head)) = parts.pop() {
            let in_head = in_head || p.name == "head" || p.name == "hat";
            if !p.polygons.is_empty() {
                if !in_head {
                    return false;
                }
                drawn = true;
            }
            parts.extend(p.children.iter().map(|c| (c, in_head)));
        }
        drawn
    };
    value.into_iter().filter(|layer| !head_only(layer.0)).collect()
}
