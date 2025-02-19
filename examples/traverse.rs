slint::slint! {
    import { SpatialFocusHandler, SpatialFocus } from "res/spatial-focus.slint";
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
            VerticalLayout {
                HorizontalLayout {
                    HorizontalLayout { for i in 3: Item {} }
                    VerticalLayout { for i in 3: Item {} }
                    HorizontalLayout { for i in 3: Item {} }
                    VerticalLayout {  for i in 3: Item {} }
                }
                GridLayout {
                    Row { Item {} Item {} }
                    Row { Item {} Item {} }
                }
                HorizontalLayout {
                    VerticalLayout { for i in 2: Item {} }
                    HorizontalLayout { for i in 2: Item {} }
                    VerticalLayout {  for i in 2: Item {} }
                    HorizontalLayout { for i in 2: Item {} }
                }
            }

        }
    }
}

fn main() {
    let app = App::new().unwrap();
    slint_spatial_focus::init!(&app);
    app.run().unwrap();
}
