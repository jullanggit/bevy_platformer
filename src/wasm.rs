use bevy::{prelude::*, window::PrimaryWindow};
use web_sys::wasm_bindgen::JsCast;
// Plugin
pub struct WasmPlugin;
impl Plugin for WasmPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, fit_canvas_to_parent);
    }
}

fn fit_canvas_to_parent() {
    let canvas: web_sys::HtmlCanvasElement = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .query_selector("canvas")
        .unwrap()
        .unwrap()
        .unchecked_into();
    let style = canvas.style();
    style.set_property("width", "100%").unwrap();
    style.set_property("height", "100%").unwrap();
}
