use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        mpsc::{self, Receiver, TryRecvError},
    },
};

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use image::{GrayImage, RgbImage};

use crate::{
    HEIGHT, Result, WIDTH,
    dither::Algorithm,
    export,
    mono::MonoImage,
    pipeline::{
        BRIGHTNESS_RANGE, CONTRAST_RANGE, CROP_ZOOM_RANGE, Crop, Framing, GAMMA_RANGE, Settings,
        Source,
    },
};

struct Prepared {
    source: Arc<Source>,
    fit: GrayImage,
    crop: GrayImage,
    thumbnail: RgbImage,
}

impl Prepared {
    fn open(path: &Path) -> Result<Self> {
        let source = Arc::new(Source::open(path)?);
        let fit = source.prepare(Framing::Fit);
        let crop = source.prepare(Framing::Crop);
        let thumbnail = image::imageops::thumbnail(source.image(), WIDTH, HEIGHT);
        Ok(Self {
            source,
            fit,
            crop,
            thumbnail,
        })
    }

    fn gray(&self, framing: Framing) -> &GrayImage {
        match framing {
            Framing::Fit => &self.fit,
            Framing::Crop => &self.crop,
        }
    }
}

struct Pending {
    path: PathBuf,
    receiver: Receiver<std::result::Result<Prepared, String>>,
}

struct Loaded {
    path: PathBuf,
    prepared: Prepared,
    mono: MonoImage,
    source_texture: TextureHandle,
    result_texture: TextureHandle,
    cached_crop: Crop,
    pending_crop: Option<PendingCrop>,
}

struct PendingCrop {
    selection: Crop,
    receiver: Receiver<std::result::Result<GrayImage, String>>,
}

pub struct Editor {
    settings: Settings,
    loaded: Option<Loaded>,
    pending: Option<Pending>,
    zoom: u32,
    crop: Crop,
    error: Option<String>,
    status: String,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            loaded: None,
            pending: None,
            zoom: 2,
            crop: Crop::default(),
            error: None,
            status: "Open a PNG, JPEG, BMP, or WebP image to begin.".into(),
        }
    }
}

fn preview(mono: &MonoImage) -> ColorImage {
    ColorImage::from_gray([WIDTH as usize, HEIGHT as usize], mono.pixels().as_raw())
}

impl Editor {
    fn open(&mut self, path: &Path, ctx: &egui::Context) -> Result<()> {
        if self.pending.is_some() {
            return Err("an image is already loading".into());
        }
        let (sender, receiver) = mpsc::sync_channel(1);
        let worker_path = path.to_path_buf();
        let ctx = ctx.clone();
        // Dropping the handle detaches: closing the window never joins a decoder.
        std::thread::Builder::new()
            .name("image-loader".into())
            .spawn(move || {
                let result = Prepared::open(&worker_path).map_err(|error| error.to_string());
                let _ = sender.send(result);
                ctx.request_repaint();
            })?;
        self.pending = Some(Pending {
            path: path.to_path_buf(),
            receiver,
        });
        self.error = None;
        self.status = format!("Loading {}…", path.display());
        Ok(())
    }

    fn poll_load(&mut self, ctx: &egui::Context) {
        let Some(pending) = &self.pending else { return };
        let result = match pending.receiver.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err("image loader stopped unexpectedly".into()),
        };
        let pending = self.pending.take().unwrap();
        if let Err(error) = result
            .map_err(Into::into)
            .and_then(|prepared| self.install(pending.path.clone(), prepared, ctx))
        {
            self.error = Some(format!("Cannot load {}: {error}", pending.path.display()));
            self.status = "Load failed; previous image retained.".into();
        }
    }

    fn install(&mut self, path: PathBuf, prepared: Prepared, ctx: &egui::Context) -> Result<()> {
        // Build the complete replacement before dropping the previous valid image.
        // Controls remain live while loading; render with the latest settings.
        let mono = self.settings.render(prepared.gray(self.settings.framing))?;
        let thumbnail = &prepared.thumbnail;
        let source_texture = ctx.load_texture(
            "source",
            ColorImage::from_rgb(
                [thumbnail.width() as usize, thumbnail.height() as usize],
                thumbnail.as_raw(),
            ),
            TextureOptions::NEAREST,
        );
        let result_texture = ctx.load_texture("result", preview(&mono), TextureOptions::NEAREST);
        self.status = format!(
            "Loaded {} · {}×{}",
            path.display(),
            prepared.source.image().width(),
            prepared.source.image().height()
        );
        self.loaded = Some(Loaded {
            path,
            prepared,
            mono,
            source_texture,
            result_texture,
            cached_crop: Crop::default(),
            pending_crop: None,
        });
        self.crop = Crop::default();
        self.error = None;
        Ok(())
    }

    fn refresh(&mut self, previous: &Settings) -> Result<()> {
        if self.settings != *previous {
            self.render_preview()?;
        }
        Ok(())
    }

    fn render_preview(&mut self) -> Result<()> {
        if let Some(loaded) = &mut self.loaded {
            let mono = self
                .settings
                .render(loaded.prepared.gray(self.settings.framing))?;
            loaded
                .result_texture
                .set(preview(&mono), TextureOptions::NEAREST);
            loaded.mono = mono;
            self.error = None;
            if self.pending.is_none() {
                self.status = "Preview updated.".into();
            }
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.settings = Settings::default();
        self.zoom = 2;
        self.crop = Crop::default();
    }

    fn crop_updating(&self) -> bool {
        self.settings.framing == Framing::Crop
            && self
                .loaded
                .as_ref()
                .is_some_and(|loaded| loaded.cached_crop != self.crop)
    }

    fn update_crop(&mut self, ctx: &egui::Context) -> Result<()> {
        let Some(loaded) = &mut self.loaded else {
            return Ok(());
        };
        let mut changed = false;
        if let Some(pending) = &loaded.pending_crop {
            let result = match pending.receiver.try_recv() {
                Ok(result) => Some(result),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => {
                    Some(Err("crop worker stopped unexpectedly".into()))
                }
            };
            if let Some(result) = result {
                let pending = loaded.pending_crop.take().unwrap();
                if pending.selection == self.crop {
                    loaded.prepared.crop =
                        result.map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;
                    loaded.cached_crop = pending.selection;
                    changed = self.settings.framing == Framing::Crop;
                }
            }
        }
        if self.settings.framing == Framing::Crop
            && loaded.cached_crop != self.crop
            && loaded.pending_crop.is_none()
        {
            self.crop = self.crop.constrained(
                loaded.prepared.source.image().width(),
                loaded.prepared.source.image().height(),
            )?;
            let selection = self.crop;
            let source = Arc::clone(&loaded.prepared.source);
            let (sender, receiver) = mpsc::sync_channel(1);
            let ctx = ctx.clone();
            // No request queue: while this runs, the UI only retains the latest selection.
            std::thread::Builder::new()
                .name("crop-preparer".into())
                .spawn(move || {
                    let result = source
                        .prepare_crop(selection)
                        .map_err(|error| error.to_string());
                    let _ = sender.send(result);
                    ctx.request_repaint();
                })?;
            loaded.pending_crop = Some(PendingCrop {
                selection,
                receiver,
            });
        }
        if changed {
            self.render_preview()?;
        }
        Ok(())
    }

    fn save_selection(&mut self, selection: Option<&Path>) -> Result<()> {
        let Some(path) = selection else {
            self.status =
                "No export selected. If no dialog appeared, check the desktop portal.".into();
            return Ok(());
        };
        if let Some(loaded) = &self.loaded {
            // The native save dialog approved replacement; new paths still use no-clobber.
            let overwrite = path.symlink_metadata().is_ok();
            export::save(&loaded.mono, &loaded.path, path, overwrite)?;
            self.error = None;
            self.status = format!("Exported {}", path.display());
        }
        Ok(())
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        if ui
            .add_enabled(
                self.pending.is_none(),
                egui::Button::new("Open image…").min_size(egui::vec2(110.0, 32.0)),
            )
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "webp"])
                .pick_file()
            {
                if let Err(error) = self.open(&path, ui.ctx()) {
                    self.error = Some(format!("Cannot load {}: {error}", path.display()));
                }
            } else {
                self.status =
                    "No image selected. If no dialog appeared, check the desktop portal.".into();
            }
        }
        if ui
            .add_enabled(
                self.loaded.is_some() && self.pending.is_none(),
                egui::Button::new("Export BMP…").min_size(egui::vec2(120.0, 32.0)),
            )
            .clicked()
        {
            let selection = rfd::FileDialog::new()
                .add_filter("1-bit BMP", &["bmp"])
                .set_file_name("off-image.bmp")
                .save_file();
            if let Err(error) = self.save_selection(selection.as_deref()) {
                self.error = Some(format!("Cannot export: {error}"));
            }
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Framing");
        ui.radio_value(
            &mut self.settings.framing,
            Framing::Fit,
            "Fit · white margins",
        );
        ui.radio_value(
            &mut self.settings.framing,
            Framing::Crop,
            "Fill · interactive crop",
        );
        if self.settings.framing == Framing::Crop {
            ui.small(format!(
                "Crop zoom: {:.1}× · wheel to zoom, drag to pan",
                self.crop.zoom
            ));
        }
        ui.add_space(8.0);
        ui.heading("Dithering");
        for (algorithm, label) in [
            (Algorithm::FloydSteinberg, "Floyd–Steinberg"),
            (Algorithm::Bayer, "Ordered Bayer"),
            (Algorithm::Atkinson, "Atkinson"),
            (Algorithm::Stucki, "Stucki"),
            (Algorithm::BlueNoise, "Blue noise"),
            (Algorithm::Sierra, "Sierra"),
            (Algorithm::Burkes, "Burkes"),
            (Algorithm::JarvisJudiceNinke, "Jarvis–Judice–Ninke"),
            (Algorithm::Riemersma, "Riemersma"),
            (Algorithm::ClusteredDot, "Clustered dot"),
            (Algorithm::Threshold, "Threshold"),
        ] {
            ui.radio_value(&mut self.settings.algorithm, algorithm, label);
        }
        if self.settings.algorithm == Algorithm::Threshold {
            ui.add(egui::Slider::new(&mut self.settings.threshold, 0..=255).text("Cutoff"));
        }
        ui.add_space(8.0);
        ui.heading("Tone");
        for (label, value, range) in [
            (
                "Brightness",
                &mut self.settings.brightness,
                BRIGHTNESS_RANGE,
            ),
            ("Contrast", &mut self.settings.contrast, CONTRAST_RANGE),
            ("Gamma", &mut self.settings.gamma, GAMMA_RANGE),
        ] {
            let label = ui.label(label);
            ui.add(egui::Slider::new(value, range))
                .labelled_by(label.id);
        }
        ui.checkbox(&mut self.settings.invert, "Invert black / white");
        ui.add_space(8.0);
        ui.separator();
        if ui
            .add_sized(
                [ui.available_width(), 30.0],
                egui::Button::new("Reset controls"),
            )
            .clicked()
        {
            self.reset();
        }
    }

    fn zoom_control(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Preview zoom");
            for zoom in 1..=3 {
                ui.selectable_value(&mut self.zoom, zoom, format!("{zoom}×"));
            }
        });
    }

    fn preview_views(&mut self, ui: &mut egui::Ui) -> egui::scroll_area::ScrollAreaOutput<()> {
        // Integer physical-pixel zoom, independent of the desktop's UI scaling.
        let scale = self.zoom as f32 / ui.ctx().pixels_per_point();
        egui::ScrollArea::both().show(ui, |ui| {
            ui.vertical(|ui| {
                egui::Frame::group(ui.style())
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.strong("Source / crop editor");
                            ui.add_space(8.0);
                            self.source_view(ui, scale);
                        });
                    });
                egui::Frame::group(ui.style())
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.strong("400 × 240 · export preview");
                            ui.add_space(8.0);
                            let loaded = self.loaded.as_ref().unwrap();
                            ui.image((
                                loaded.result_texture.id(),
                                loaded.result_texture.size_vec2() * scale,
                            ));
                            if self.crop_updating() {
                                ui.spinner();
                                ui.label("Updating crop… Export uses the displayed preview.");
                            }
                        });
                    });
            });
        })
    }

    fn source_view(&mut self, ui: &mut egui::Ui, scale: f32) {
        let Some(loaded) = &self.loaded else { return };
        if self.settings.framing == Framing::Fit {
            ui.image((
                loaded.source_texture.id(),
                loaded.source_texture.size_vec2() * scale,
            ));
            return;
        }
        let (width, height) = loaded.prepared.source.image().dimensions();
        let bounds = self.crop.bounds(width, height).expect("validated crop");
        let uv = egui::Rect::from_min_max(
            egui::pos2(
                bounds.x as f32 / width as f32,
                bounds.y as f32 / height as f32,
            ),
            egui::pos2(
                (bounds.x + bounds.width) as f32 / width as f32,
                (bounds.y + bounds.height) as f32 / height as f32,
            ),
        );
        let response = ui.add(
            egui::Image::new((
                loaded.source_texture.id(),
                egui::vec2(WIDTH as f32, HEIGHT as f32) * scale,
            ))
            .uv(uv)
            .sense(egui::Sense::click_and_drag()),
        ).on_hover_text("Wheel: crop zoom (1×–16×). Drag: pan image.\nKeyboard: Tab to frame, arrows pan, +/− zoom.");
        if response.clicked() || response.drag_started() {
            response.request_focus();
        }
        ui.painter().rect_stroke(
            response.rect,
            0.0,
            egui::Stroke::new(1.0_f32, egui::Color32::YELLOW),
            egui::StrokeKind::Inside,
        );
        let mut crop = self.crop;
        if response.has_focus() {
            ui.memory_mut(|memory| {
                memory.set_focus_lock_filter(
                    response.id,
                    egui::EventFilter {
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        ..Default::default()
                    },
                )
            });
            let (dx, dy, zoom) = ui.input_mut(|input| {
                let mut pressed = |key| {
                    i32::from(
                        input.consume_key(egui::Modifiers::NONE, key)
                            || input.consume_key(egui::Modifiers::SHIFT, key),
                    )
                };
                (
                    pressed(egui::Key::ArrowRight) - pressed(egui::Key::ArrowLeft),
                    pressed(egui::Key::ArrowDown) - pressed(egui::Key::ArrowUp),
                    pressed(egui::Key::Plus) + pressed(egui::Key::Equals)
                        - pressed(egui::Key::Minus),
                )
            });
            crop.zoom = (crop.zoom * 1.1_f64.powi(zoom))
                .clamp(*CROP_ZOOM_RANGE.start(), *CROP_ZOOM_RANGE.end());
            match crop.pan(f64::from(dx) * 0.05, f64::from(dy) * 0.05, width, height) {
                Ok(panned) => crop = panned,
                Err(error) => self.error = Some(error.to_string()),
            }
        }
        if response.dragged() {
            let delta = ui.input(|input| input.pointer.delta());
            match crop.pan(
                f64::from(delta.x / response.rect.width()),
                f64::from(delta.y / response.rect.height()),
                width,
                height,
            ) {
                Ok(panned) => crop = panned,
                Err(error) => self.error = Some(error.to_string()),
            }
        }
        if response.hovered() {
            let scroll = ui.input_mut(|input| {
                let delta = input.smooth_scroll_delta.y;
                input.smooth_scroll_delta = egui::Vec2::ZERO;
                input.raw_scroll_delta = egui::Vec2::ZERO;
                delta
            });
            crop.zoom = (crop.zoom * (f64::from(scroll) * 0.005).exp())
                .clamp(*CROP_ZOOM_RANGE.start(), *CROP_ZOOM_RANGE.end());
        }
        match crop.constrained(width, height) {
            Ok(crop) => self.crop = crop,
            Err(error) => self.error = Some(error.to_string()),
        }
    }
}

impl eframe::App for Editor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_load(ctx);
        let previous = self.settings.clone();
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("R47 OFF-image");
                ui.add_space(24.0);
                self.toolbar(ui);
                if self.pending.is_some() {
                    ui.spinner();
                    ui.label("Loading image…");
                }
            });
            ui.add_space(8.0);
        });
        egui::SidePanel::left("controls")
            .default_width(240.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(12.0);
                ui.spacing_mut().item_spacing.y = 8.0;
                ui.spacing_mut().slider_width = 150.0;
                egui::ScrollArea::vertical().show(ui, |ui| self.controls(ui));
            });
        if let Err(error) = self.refresh(&previous) {
            self.error = Some(error.to_string());
        }
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::RED, format!("Error: {error}"));
            }
            ui.label(&self.status);
            ui.small("400×240 · 1-bit BMP · R47 hardware compatibility not yet verified");
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.zoom_control(ui);
            ui.add_space(16.0);
            if self.loaded.is_some() {
                self.preview_views(ui);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Open an image to create your calculator's OFF screen.");
                });
            }
        });
        if let Err(error) = self.update_crop(ctx) {
            self.error = Some(error.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Wait for a real worker without sleeps, then exercise the UI's polling path.
    fn load(editor: &mut Editor, path: &Path, ctx: &egui::Context) {
        editor.open(path, ctx).unwrap();
        let pending = editor.pending.as_mut().unwrap();
        let result = pending.receiver.recv().unwrap();
        let (sender, receiver) = mpsc::sync_channel(1);
        sender
            .send(result)
            .unwrap_or_else(|_| panic!("receiver dropped"));
        pending.receiver = receiver;
        editor.poll_load(ctx);
        assert!(editor.pending.is_none());
    }

    fn finish_crop(editor: &mut Editor, ctx: &egui::Context) {
        let pending = editor
            .loaded
            .as_mut()
            .unwrap()
            .pending_crop
            .as_mut()
            .unwrap();
        let result = pending.receiver.recv().unwrap();
        let (sender, receiver) = mpsc::sync_channel(1);
        sender.send(result).unwrap();
        pending.receiver = receiver;
        editor.update_crop(ctx).unwrap();
    }

    #[test]
    fn previews_stack_at_integer_zoom_and_scroll_without_changing_export() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.bmp");
        let output = directory.path().join("output.bmp");
        std::fs::write(&input, include_bytes!("../numbers.bmp")).unwrap();
        for pixels_per_point in [1.0, 1.5] {
            for zoom in 1..=3 {
                for framing in [Framing::Fit, Framing::Crop] {
                    let ctx = egui::Context::default();
                    ctx.set_pixels_per_point(pixels_per_point);
                    let mut editor = Editor::default();
                    load(&mut editor, &input, &ctx);
                    editor.zoom = zoom;
                    editor.settings.framing = framing;
                    let pixels = editor.loaded.as_ref().unwrap().mono.clone();
                    let mut view = None;
                    let mut frame = |size, events| {
                        ctx.run(
                            egui::RawInput {
                                screen_rect: Some(egui::Rect::from_min_size(
                                    egui::Pos2::ZERO,
                                    size,
                                )),
                                events,
                                ..Default::default()
                            },
                            |ctx| {
                                egui::CentralPanel::default().show(ctx, |ui| {
                                    view = Some(editor.preview_views(ui));
                                });
                            },
                        )
                    };
                    let large = egui::vec2(2000.0, 2000.0);
                    frame(large, vec![]);
                    let rendered = frame(large, vec![]);
                    let images: Vec<_> = rendered
                        .shapes
                        .iter()
                        .filter_map(|shape| match &shape.shape {
                            egui::Shape::Rect(rect)
                                if rect.fill_texture_id() != egui::TextureId::default() =>
                            {
                                Some(rect.rect)
                            }
                            _ => None,
                        })
                        .collect();
                    assert_eq!(images.len(), 2);
                    let expected =
                        egui::vec2(WIDTH as f32, HEIGHT as f32) * (zoom as f32 / pixels_per_point);
                    for rect in &images {
                        assert!((rect.size() - expected).length() < 1.0);
                    }
                    assert!(images[0].bottom() < images[1].top());
                    assert_eq!(images[0].left(), images[1].left());

                    let small = egui::vec2(300.0, 200.0);
                    frame(small, vec![]);
                    frame(
                        small,
                        vec![
                            // Over the card label, not the wheel-zoom crop frame.
                            egui::Event::PointerMoved(egui::pos2(30.0, 25.0)),
                            egui::Event::MouseWheel {
                                unit: egui::MouseWheelUnit::Point,
                                delta: egui::vec2(-2000.0, -2000.0),
                                modifiers: egui::Modifiers::NONE,
                            },
                        ],
                    );
                    let view = view.unwrap();
                    assert!(view.content_size.x >= expected.x);
                    assert!(view.content_size.y >= expected.y * 2.0);
                    assert!(view.state.offset.y > 0.0);
                    if expected.x > view.inner_rect.width() {
                        assert!(view.state.offset.x > 0.0);
                    }
                    assert_eq!(editor.crop, Crop::default());
                    assert_eq!(editor.loaded.as_ref().unwrap().mono, pixels);
                    editor.save_selection(Some(&output)).unwrap();
                    assert_eq!(&image::open(&output).unwrap().into_luma8(), pixels.pixels());
                }
            }
        }
    }

    #[test]
    fn crop_editor_drag_and_wheel_do_not_scroll_the_surrounding_view() {
        fn frame(editor: &mut Editor, ctx: &egui::Context, events: Vec<egui::Event>) -> egui::Vec2 {
            let mut offset = egui::Vec2::ZERO;
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(600.0, 400.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        offset = editor.preview_views(ui).state.offset;
                    });
                },
            );
            offset
        }
        let ctx = egui::Context::default();
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.bmp");
        std::fs::write(&input, include_bytes!("../numbers.bmp")).unwrap();
        let mut editor = Editor::default();
        load(&mut editor, &input, &ctx);
        editor.settings.framing = Framing::Crop;
        editor.zoom = 1;
        editor.crop.zoom = 2.0;
        frame(
            &mut editor,
            &ctx,
            vec![egui::Event::PointerMoved(egui::pos2(100.0, 100.0))],
        );
        let offset = frame(
            &mut editor,
            &ctx,
            vec![egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, 20.0),
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert!(editor.crop.zoom > 2.0);
        assert_eq!(offset, egui::Vec2::ZERO);
        frame(
            &mut editor,
            &ctx,
            vec![egui::Event::PointerButton {
                pos: egui::pos2(100.0, 100.0),
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        let before = editor.crop;
        let offset = frame(
            &mut editor,
            &ctx,
            vec![egui::Event::PointerMoved(egui::pos2(140.0, 120.0))],
        );
        assert!(editor.crop.center_x < before.center_x && editor.crop.center_y < before.center_y);
        assert_eq!(offset, egui::Vec2::ZERO);
        frame(
            &mut editor,
            &ctx,
            vec![
                egui::Event::PointerButton {
                    pos: egui::pos2(140.0, 120.0),
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerMoved(egui::pos2(100.0, 320.0)),
            ],
        );
        let zoom = editor.crop.zoom;
        let offset = frame(
            &mut editor,
            &ctx,
            vec![egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, -50.0),
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert_eq!(editor.crop.zoom, zoom);
        assert!(offset.y > 0.0);
    }

    #[test]
    fn crop_frame_supports_keyboard_zoom_and_pan_without_sliders() {
        fn frame(editor: &mut Editor, ctx: &egui::Context, events: Vec<egui::Event>) {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(600.0, 1200.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| editor.source_view(ui, 1.0));
                },
            );
        }
        let ctx = egui::Context::default();
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.bmp");
        std::fs::write(&input, include_bytes!("../numbers.bmp")).unwrap();
        let mut editor = Editor::default();
        load(&mut editor, &input, &ctx);
        editor.settings.framing = Framing::Crop;
        editor.crop.zoom = 2.0;
        let key = |key| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        frame(&mut editor, &ctx, vec![]);
        frame(&mut editor, &ctx, vec![key(egui::Key::Tab)]);
        frame(&mut editor, &ctx, vec![]);
        let before = editor.crop;
        frame(
            &mut editor,
            &ctx,
            vec![
                key(egui::Key::ArrowRight),
                key(egui::Key::ArrowDown),
                key(egui::Key::Plus),
            ],
        );
        assert!(editor.crop.zoom > before.zoom);
        assert!(editor.crop.center_x < before.center_x);
        assert!(editor.crop.center_y < before.center_y);
        frame(&mut editor, &ctx, vec![key(egui::Key::Minus)]);
        assert!((editor.crop.zoom - before.zoom).abs() < 1e-10);
    }

    #[test]
    fn crop_coalesces_requests_rejects_stale_results_and_exports_displayed_pixels() {
        let ctx = egui::Context::default();
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("asymmetric.png");
        let output = directory.path().join("output.bmp");
        RgbImage::from_fn(1200, 720, |x, y| {
            image::Rgb(
                [if (400..800).contains(&x) && (120..360).contains(&y) {
                    0
                } else {
                    255
                }; 3],
            )
        })
        .save(&input)
        .unwrap();
        let mut editor = Editor::default();
        load(&mut editor, &input, &ctx);
        let before = editor.settings.clone();
        editor.settings.framing = Framing::Crop;
        editor.settings.algorithm = Algorithm::Threshold;
        editor.refresh(&before).unwrap();
        let displayed = editor.loaded.as_ref().unwrap().mono.clone();

        // Hold one request open so coalescing is deterministic, independent of CPU speed.
        let first = Crop {
            zoom: 2.0,
            ..Crop::default()
        };
        let (sender, receiver) = mpsc::sync_channel(1);
        editor.loaded.as_mut().unwrap().pending_crop = Some(PendingCrop {
            selection: first,
            receiver,
        });
        for step in 0..100 {
            editor.crop = Crop {
                center_x: 0.5,
                center_y: 1.0 / 3.0,
                zoom: 3.0 + f64::from(step) / 100.0,
            };
            editor.update_crop(&ctx).unwrap();
            assert_eq!(
                editor
                    .loaded
                    .as_ref()
                    .unwrap()
                    .pending_crop
                    .as_ref()
                    .unwrap()
                    .selection,
                first
            );
        }
        editor.crop = Crop {
            center_x: 0.5,
            center_y: 1.0 / 3.0,
            zoom: 3.0,
        };
        assert!(editor.crop_updating());
        editor.save_selection(Some(&output)).unwrap();
        assert_eq!(
            &image::open(&output).unwrap().into_luma8(),
            displayed.pixels()
        );

        sender
            .send(Ok(GrayImage::from_pixel(WIDTH, HEIGHT, image::Luma([255]))))
            .unwrap();
        editor.update_crop(&ctx).unwrap();
        let loaded = editor.loaded.as_ref().unwrap();
        assert_eq!(loaded.mono, displayed);
        assert_eq!(loaded.pending_crop.as_ref().unwrap().selection, editor.crop);

        // A completing crop must use current controls, not a worker's stale snapshot.
        let previous = editor.settings.clone();
        editor.settings.invert = true;
        editor.settings.gamma = 1.7;
        editor.refresh(&previous).unwrap();
        finish_crop(&mut editor, &ctx);
        assert!(!editor.crop_updating());
        let loaded = editor.loaded.as_ref().unwrap();
        assert!(loaded.pending_crop.is_none());
        let selected = loaded.prepared.source.prepare_crop(editor.crop).unwrap();
        assert!(selected.pixels().all(|p| p[0] == 0));
        assert_eq!(loaded.mono, editor.settings.render(&selected).unwrap());
        let cached = loaded.prepared.crop.as_ptr();
        let pixels = loaded.mono.clone();
        editor.zoom = 3;
        editor.update_crop(&ctx).unwrap();
        assert_eq!(
            editor.loaded.as_ref().unwrap().prepared.crop.as_ptr(),
            cached
        );
        assert_eq!(editor.loaded.as_ref().unwrap().mono, pixels);
        editor.save_selection(Some(&output)).unwrap();
        let decoded = image::open(&output).unwrap().into_luma8();
        assert_eq!(&decoded, pixels.pixels());
        for (pixel, color) in decoded.pixels().zip(preview(&pixels).pixels) {
            assert_eq!(color.to_array(), [pixel[0], pixel[0], pixel[0], 255]);
        }
    }

    #[test]
    fn crop_survives_fit_toggle_but_reset_and_new_image_center_it() {
        let ctx = egui::Context::default();
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.bmp");
        std::fs::write(&input, include_bytes!("../numbers.bmp")).unwrap();
        let mut editor = Editor::default();
        assert_eq!(editor.zoom, 2);
        load(&mut editor, &input, &ctx);
        editor.crop = Crop {
            center_x: 0.4,
            center_y: 0.3,
            zoom: 4.0,
        };
        let before = editor.settings.clone();
        editor.settings.framing = Framing::Crop;
        editor.settings.brightness = 0.2;
        editor.refresh(&before).unwrap();
        editor.update_crop(&ctx).unwrap();
        finish_crop(&mut editor, &ctx);
        let selection = editor.crop;
        let cached = editor.loaded.as_ref().unwrap().prepared.crop.as_ptr();
        for framing in [Framing::Fit, Framing::Crop] {
            let before = editor.settings.clone();
            editor.settings.framing = framing;
            editor.refresh(&before).unwrap();
            editor.update_crop(&ctx).unwrap();
            assert_eq!(editor.crop, selection);
            assert_eq!(editor.settings.brightness, 0.2);
            assert_eq!(
                editor.loaded.as_ref().unwrap().prepared.crop.as_ptr(),
                cached
            );
            assert!(editor.loaded.as_ref().unwrap().pending_crop.is_none());
        }
        let before = editor.settings.clone();
        editor.zoom = 3;
        editor.reset();
        editor.refresh(&before).unwrap();
        assert_eq!(editor.settings, Settings::default());
        assert_eq!(editor.zoom, 2);
        assert_eq!(editor.crop, Crop::default());
        let loaded = editor.loaded.as_ref().unwrap();
        assert_eq!(
            loaded.mono,
            loaded
                .prepared
                .source
                .convert(&Settings::default())
                .unwrap()
        );

        let before = editor.settings.clone();
        editor.settings.framing = Framing::Crop;
        editor.refresh(&before).unwrap();
        editor.update_crop(&ctx).unwrap();
        finish_crop(&mut editor, &ctx);
        assert_eq!(editor.loaded.as_ref().unwrap().cached_crop, Crop::default());

        // Loading a replacement drops the old crop receiver; no old-source result can install.
        let (sender, receiver) = mpsc::sync_channel(1);
        editor.crop = selection;
        editor.loaded.as_mut().unwrap().pending_crop = Some(PendingCrop {
            selection,
            receiver,
        });
        editor.settings.gamma = 1.5;
        load(&mut editor, &input, &ctx);
        assert!(sender.send(Ok(GrayImage::new(WIDTH, HEIGHT))).is_err());
        assert_eq!(editor.crop, Crop::default());
        assert_eq!(editor.settings.gamma, 1.5);
        assert!(editor.loaded.as_ref().unwrap().pending_crop.is_none());
        assert_eq!(
            editor.loaded.as_ref().unwrap().mono,
            editor
                .loaded
                .as_ref()
                .unwrap()
                .prepared
                .source
                .convert(&editor.settings)
                .unwrap()
        );
    }

    #[test]
    #[ignore = "manual crop timings on a generated 3840×2160 input"]
    fn large_image_crop_timings() {
        use std::time::Instant;
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("large.png");
        RgbImage::from_fn(3840, 2160, |x, y| {
            image::Rgb([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8])
        })
        .save(&input)
        .unwrap();
        let source = Source::open(&input).unwrap();
        for zoom in [1.0, 2.0, 16.0] {
            let start = Instant::now();
            source
                .prepare_crop(Crop {
                    zoom,
                    ..Crop::default()
                })
                .unwrap();
            eprintln!("3840×2160 crop zoom {zoom}: {:?}", start.elapsed());
        }
        let ctx = egui::Context::default();
        let mut editor = Editor::default();
        load(&mut editor, &input, &ctx);
        editor.settings.framing = Framing::Crop;
        editor.crop.zoom = 2.0;
        editor.update_crop(&ctx).unwrap();
        let start = Instant::now();
        for step in 0..200 {
            editor.crop.center_x = 0.4 + f64::from(step) / 2000.0;
            editor.update_crop(&ctx).unwrap();
        }
        eprintln!(
            "200 UI crop-request updates while worker runs: {:?}",
            start.elapsed()
        );
        while editor.crop_updating() {
            finish_crop(&mut editor, &ctx);
        }
        assert_eq!(editor.loaded.as_ref().unwrap().cached_crop, editor.crop);
    }

    #[test]
    #[ignore = "manual stage timings; set OFFIMG_BENCH_IMAGE to a representative wallpaper"]
    fn image_loading_timings() {
        use std::time::Instant;

        let path = std::env::var_os("OFFIMG_BENCH_IMAGE").expect("set OFFIMG_BENCH_IMAGE");
        let start = Instant::now();
        let source = Source::open(Path::new(&path)).unwrap();
        eprintln!(
            "decode/orient/composite {:?}: {:?}",
            source.image().dimensions(),
            start.elapsed()
        );
        for framing in [Framing::Fit, Framing::Crop, Framing::Fit, Framing::Crop] {
            let start = Instant::now();
            let gray = source.prepare(framing);
            eprintln!("full-source {framing} prepare: {:?}", start.elapsed());
            let start = Instant::now();
            let mono = Settings::default().render(&gray).unwrap();
            let ctx = egui::Context::default();
            let _texture = ctx.load_texture("result", preview(&mono), TextureOptions::NEAREST);
            eprintln!("cached render/texture: {:?}", start.elapsed());
        }
        let start = Instant::now();
        let thumbnail = image::imageops::thumbnail(source.image(), WIDTH, HEIGHT);
        eprintln!("thumbnail: {:?}", start.elapsed());
        let start = Instant::now();
        let prepared = Prepared {
            fit: source.prepare(Framing::Fit),
            crop: source.prepare(Framing::Crop),
            source: Arc::new(source),
            thumbnail,
        };
        eprintln!("both framing caches: {:?}", start.elapsed());
        let ctx = egui::Context::default();
        let mut editor = Editor::default();
        let start = Instant::now();
        editor.install(PathBuf::from(path), prepared, &ctx).unwrap();
        eprintln!("UI install/render/textures: {:?}", start.elapsed());
        for framing in [Framing::Crop, Framing::Fit, Framing::Crop, Framing::Fit] {
            let previous = editor.settings.clone();
            editor.settings.framing = framing;
            let start = Instant::now();
            editor.refresh(&previous).unwrap();
            eprintln!("cached {framing} render/texture: {:?}", start.elapsed());
        }
    }

    #[test]
    fn gui_preview_export_cache_reset_and_failures() {
        let ctx = egui::Context::default();
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.png");
        let output = directory.path().join("output.bmp");
        image::RgbImage::from_fn(320, 200, |x, y| image::Rgb([((x + y) % 256) as u8; 3]))
            .save(&input)
            .unwrap();
        let mut editor = Editor::default();
        load(&mut editor, &input, &ctx);
        let cached = editor.loaded.as_ref().unwrap().prepared.fit.as_ptr();
        let before = editor.settings.clone();
        editor.settings.gamma = 1.5;
        editor.settings.invert = true;
        let start = std::time::Instant::now();
        editor.refresh(&before).unwrap();
        eprintln!(
            "cached GUI tone/dither/texture update: {:?}",
            start.elapsed()
        );
        assert_eq!(
            editor.loaded.as_ref().unwrap().prepared.fit.as_ptr(),
            cached
        );
        editor.save_selection(Some(&output)).unwrap();
        let loaded = editor.loaded.as_ref().unwrap();
        let decoded = image::open(&output).unwrap().into_luma8();
        assert_eq!(&decoded, loaded.mono.pixels());
        let texture = preview(&loaded.mono);
        for (pixel, color) in decoded.pixels().zip(texture.pixels) {
            assert_eq!(color.to_array(), [pixel[0], pixel[0], pixel[0], 255]);
        }
        let old_mono = loaded.mono.clone();
        load(&mut editor, &directory.path().join("missing"), &ctx);
        assert!(editor.error.as_ref().unwrap().contains("Cannot load"));
        assert_eq!(editor.loaded.as_ref().unwrap().mono, old_mono);
        assert!(editor.save_selection(Some(&input)).is_err());
        assert_eq!(image::open(&output).unwrap().into_luma8(), decoded);
        assert!(
            editor
                .save_selection(Some(&directory.path().join("missing/output.bmp")))
                .is_err()
        );
        assert_eq!(editor.loaded.as_ref().unwrap().mono, old_mono);
        let previous = editor.settings.clone();
        editor.settings.framing = Framing::Crop;
        editor.refresh(&previous).unwrap();
        assert_eq!(
            editor.loaded.as_ref().unwrap().prepared.crop,
            editor
                .loaded
                .as_ref()
                .unwrap()
                .prepared
                .source
                .prepare(Framing::Crop)
        );
        let previous = editor.settings.clone();
        editor.settings = Settings::default();
        editor.refresh(&previous).unwrap();
        let loaded = editor.loaded.as_ref().unwrap();
        assert_eq!(
            loaded.mono,
            loaded
                .prepared
                .source
                .convert(&Settings::default())
                .unwrap()
        );
    }

    #[test]
    fn new_algorithms_match_crop_tone_preview_and_export_at_every_preview_zoom() {
        let ctx = egui::Context::default();
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("gradient.png");
        let output = directory.path().join("output.bmp");
        RgbImage::from_fn(320, 200, |x, y| {
            image::Rgb([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8])
        })
        .save(&input)
        .unwrap();
        let mut editor = Editor::default();
        load(&mut editor, &input, &ctx);
        editor.settings.framing = Framing::Crop;
        editor.crop = Crop {
            center_x: 0.4,
            center_y: 0.6,
            zoom: 2.0,
        };
        editor.update_crop(&ctx).unwrap();
        finish_crop(&mut editor, &ctx);
        let selected = editor
            .loaded
            .as_ref()
            .unwrap()
            .prepared
            .source
            .prepare_crop(editor.crop)
            .unwrap();
        let cached = editor.loaded.as_ref().unwrap().prepared.crop.as_ptr();
        for algorithm in Algorithm::ALL {
            let previous = editor.settings.clone();
            editor.settings.algorithm = algorithm;
            editor.settings.brightness = -0.1;
            editor.settings.contrast = 1.3;
            editor.settings.gamma = 1.7;
            editor.settings.invert = true;
            editor.refresh(&previous).unwrap();
            let expected = editor.settings.render(&selected).unwrap();
            for zoom in 1..=3 {
                editor.zoom = zoom;
                editor.update_crop(&ctx).unwrap();
                editor.refresh(&editor.settings.clone()).unwrap();
                editor.save_selection(Some(&output)).unwrap();
                let loaded = editor.loaded.as_ref().unwrap();
                assert_eq!(loaded.prepared.crop.as_ptr(), cached);
                assert_eq!(loaded.mono, expected, "{algorithm}, zoom {zoom}");
                let decoded = image::open(&output).unwrap().into_luma8();
                assert_eq!(&decoded, expected.pixels());
                for (pixel, color) in decoded.pixels().zip(preview(&loaded.mono).pixels) {
                    assert_eq!(color.to_array(), [pixel[0], pixel[0], pixel[0], 255]);
                }
            }
        }
    }

    #[test]
    #[ignore = "manual cached 400×240 render/texture timings; no desktop required"]
    fn dithering_switch_timings() {
        let ctx = egui::Context::default();
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("gradient.png");
        RgbImage::from_fn(WIDTH, HEIGHT, |x, y| {
            image::Rgb([((x * 17 + y * 31) % 256) as u8; 3])
        })
        .save(&input)
        .unwrap();
        let mut editor = Editor::default();
        load(&mut editor, &input, &ctx);
        for algorithm in Algorithm::ALL {
            editor.settings.algorithm = algorithm;
            editor.render_preview().unwrap();
            let start = std::time::Instant::now();
            for _ in 0..20 {
                editor.render_preview().unwrap();
            }
            eprintln!(
                "{algorithm}: {:?} mean cached render/texture (20 runs)",
                start.elapsed() / 20
            );
        }
    }

    #[test]
    fn pending_load_retains_preview_uses_latest_settings_and_recovers() {
        let ctx = egui::Context::default();
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.bmp");
        std::fs::write(&input, include_bytes!("../numbers.bmp")).unwrap();
        let mut editor = Editor::default();
        load(&mut editor, &input, &ctx);
        let old_texture = editor.loaded.as_ref().unwrap().result_texture.id();
        let old_pixels = editor.loaded.as_ref().unwrap().mono.clone();

        let (sender, receiver) = mpsc::sync_channel(1);
        editor.pending = Some(Pending {
            path: input.clone(),
            receiver,
        });
        editor.poll_load(&ctx);
        assert!(editor.pending.is_some());
        assert_eq!(
            editor.loaded.as_ref().unwrap().result_texture.id(),
            old_texture
        );
        assert_eq!(editor.loaded.as_ref().unwrap().mono, old_pixels);
        assert!(editor.open(&input, &ctx).is_err());

        let previous = editor.settings.clone();
        editor.settings.gamma = 1.7;
        editor.settings.invert = true;
        editor.settings.framing = Framing::Crop;
        editor.refresh(&previous).unwrap();
        sender
            .send(Ok(Prepared::open(&input).unwrap()))
            .unwrap_or_else(|_| panic!("receiver dropped"));
        editor.poll_load(&ctx);
        assert!(editor.pending.is_none());
        let loaded = editor.loaded.as_ref().unwrap();
        assert_eq!(
            loaded.mono,
            loaded.prepared.source.convert(&editor.settings).unwrap()
        );
        assert!(editor.error.is_none());
        let texture = loaded.result_texture.id();
        let pixels = loaded.mono.clone();

        // An oversized real decoder input must follow the same failure/recovery path.
        let oversized = directory.path().join("oversized.bmp");
        let mut bytes = include_bytes!("../numbers.bmp").to_vec();
        bytes[18..22].copy_from_slice(&(crate::pipeline::MAX_DIMENSION + 1).to_le_bytes());
        std::fs::write(&oversized, bytes).unwrap();
        load(&mut editor, &oversized, &ctx);
        assert!(editor.error.as_ref().unwrap().contains("resize it first"));
        assert_eq!(editor.loaded.as_ref().unwrap().mono, pixels);
        assert_eq!(editor.loaded.as_ref().unwrap().result_texture.id(), texture);

        let (sender, receiver) = mpsc::sync_channel(1);
        editor.pending = Some(Pending {
            path: input.clone(),
            receiver,
        });
        drop(sender);
        editor.poll_load(&ctx);
        assert!(editor.pending.is_none());
        assert!(
            editor
                .error
                .as_ref()
                .unwrap()
                .contains("stopped unexpectedly")
        );
        load(&mut editor, &input, &ctx);
        assert!(editor.error.is_none());
        assert_eq!(editor.loaded.as_ref().unwrap().mono, pixels);

        // Dropping an editor with an unfinished request does not join the worker.
        let (sender, receiver) = mpsc::sync_channel(1);
        editor.pending = Some(Pending {
            path: input,
            receiver,
        });
        drop(editor);
        assert!(sender.send(Err("closed".into())).is_err());
    }

    #[test]
    fn prepared_caches_match_original_for_tiny_alpha_and_line_art_inputs() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.png");
        for (width, height) in [(1, 1), (3, 7), (800, 240), (240, 800)] {
            image::RgbaImage::from_fn(width, height, |x, y| {
                image::Rgba([
                    if (x + y) % 2 == 0 { 0 } else { 255 },
                    0,
                    0,
                    (x % 256) as u8,
                ])
            })
            .save(&input)
            .unwrap();
            let prepared = Prepared::open(&input).unwrap();
            assert_eq!(prepared.source.image().dimensions(), (width, height));
            assert!(prepared.thumbnail.width() <= WIDTH && prepared.thumbnail.height() <= HEIGHT);
            for framing in [Framing::Fit, Framing::Crop] {
                assert_eq!(prepared.gray(framing), &prepared.source.prepare(framing));
                assert_eq!(prepared.gray(framing).len(), (WIDTH * HEIGHT) as usize);
            }
        }
    }

    #[test]
    fn native_save_selection_cancellation_replacement_and_failures() {
        use std::{fs, os::unix::fs::symlink};

        let ctx = egui::Context::default();
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.bmp");
        let output = directory.path().join("output.bmp");
        fs::write(&input, include_bytes!("../numbers.bmp")).unwrap();
        fs::write(&output, b"keep this").unwrap();
        let mut editor = Editor::default();
        load(&mut editor, &input, &ctx);
        let previous = editor.settings.clone();
        editor.settings.invert = true;
        editor.refresh(&previous).unwrap();
        let mono = editor.loaded.as_ref().unwrap().mono.clone();
        let texture = editor.loaded.as_ref().unwrap().result_texture.id();

        editor.save_selection(None).unwrap();
        assert_eq!(fs::read(&output).unwrap(), b"keep this");
        assert_eq!(editor.loaded.as_ref().unwrap().mono, mono);
        assert_eq!(editor.loaded.as_ref().unwrap().result_texture.id(), texture);

        editor.save_selection(Some(&output)).unwrap();
        assert_eq!(&image::open(&output).unwrap().into_luma8(), mono.pixels());
        let exported = fs::read(&output).unwrap();

        let sym = directory.path().join("symlink.bmp");
        let hard = directory.path().join("hardlink.bmp");
        symlink(&input, &sym).unwrap();
        fs::hard_link(&input, &hard).unwrap();
        for path in [&input, &sym, &hard] {
            assert!(editor.save_selection(Some(path)).is_err());
            assert_eq!(fs::read(&input).unwrap(), include_bytes!("../numbers.bmp"));
        }
        assert_eq!(editor.loaded.as_ref().unwrap().mono, mono);

        // Force an encoder failure after the temporary file has been created.
        editor.loaded.as_mut().unwrap().mono = MonoImage::from_fn(1, 1, |_, _| true);
        assert!(editor.save_selection(Some(&output)).is_err());
        assert_eq!(fs::read(&output).unwrap(), exported);
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 4);
    }
}
