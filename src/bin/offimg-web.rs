#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, prelude::*};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window unavailable"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("document unavailable"))?;
    let canvas = document
        .get_element_by_id("offimg-canvas")
        .ok_or_else(|| JsValue::from_str("canvas element missing"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    let runner = eframe::WebRunner::new();
    spawn_local(async move {
        if let Err(error) = runner
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|_| Ok(Box::new(offimg_generator::gui::Editor::default()))),
            )
            .await
        {
            web_sys::console::error_1(&error);
        }
    });
    Ok(())
}
