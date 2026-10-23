fn main() -> eframe::Result {
    eframe::run_native(
        "R47 OFF-image generator",
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1180.0, 740.0]),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(offimg_generator::gui::Editor::default()))),
    )
}
