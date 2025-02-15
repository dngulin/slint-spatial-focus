use slint::Weak;
use slint_spatial_focus::{SpatialAxis, SpatialDirection, SpatialFocusExtensions};

slint::slint! {
    import { SpatialFocusHandler, VerticalFocusGroup, HorizontalFocusGroup, SpatialFocus } from "res/spatial-focus.slint";
    export { SpatialFocus }

    component Item inherits Rectangle {
        background: fs.has-focus ? yellow : white;
        border-color: black;
        border-width: 3px;
        forward-focus: fs;
        fs := FocusScope {}
    }

    export component App inherits Window {
        forward-focus: fh;
        fh := SpatialFocusHandler {
            HorizontalFocusGroup {
                HorizontalFocusGroup { for i in 3: Item {} }
                VerticalFocusGroup { for i in 3: Item {} }
                HorizontalFocusGroup { for i in 3: Item {} }
                VerticalFocusGroup {  for i in 3: Item {} }
            }
        }
    }
}

fn main() {
    let app = App::new().unwrap();

    let sf = app.global::<SpatialFocus>();

    let weak = app.as_weak();
    sf.on_move_up(move || mv_focus(&weak, SpatialAxis::Vertical, SpatialDirection::Backward));

    let weak = app.as_weak();
    sf.on_move_dn(move || mv_focus(&weak, SpatialAxis::Vertical, SpatialDirection::Forward));

    let weak = app.as_weak();
    sf.on_move_l(move || mv_focus(&weak, SpatialAxis::Horizontal, SpatialDirection::Backward));

    let weak = app.as_weak();
    sf.on_move_r(move || mv_focus(&weak, SpatialAxis::Horizontal, SpatialDirection::Forward));

    app.run().unwrap();
}

fn mv_focus(weak: &Weak<App>, axis: SpatialAxis, dir: SpatialDirection) {
    if let Some(app) = weak.upgrade() {
        app.window().move_focus(axis, dir);
    }
}
