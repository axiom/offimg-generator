use std::{
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, TryRecvError},
};

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use image::{GrayImage, RgbImage};

use crate::{
    HEIGHT, Result, WIDTH,
    dither::Algorithm,
    export,
    mono::MonoImage,
    pipeline::{BRIGHTNESS_RANGE, CONTRAST_RANGE, Framing, GAMMA_RANGE, Settings, Source},
};

struct Prepared {
    source: Source,
    fit: GrayImage,
    crop: GrayImage,
    thumbnail: RgbImage,
}

impl Prepared {
    fn open(path: &Path) -> Result<Self> {
        let source = Source::open(path)?;
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
}

pub struct Editor {
    settings: Settings,
    loaded: Option<Loaded>,
    pending: Option<Pending>,
    zoom: u32,
    error: Option<String>,
    status: String,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            loaded: None,
            pending: None,
            zoom: 1,
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
        });
        self.error = None;
        Ok(())
    }

    fn refresh(&mut self, previous: &Settings) -> Result<()> {
        if let Some(loaded) = &mut self.loaded
            && self.settings != *previous
        {
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
            "Fill · center crop",
        );
        ui.add_space(8.0);
        ui.heading("Dithering");
        for (algorithm, label) in [
            (Algorithm::FloydSteinberg, "Floyd–Steinberg"),
            (Algorithm::Bayer, "Ordered Bayer"),
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
            self.settings = Settings::default();
            self.zoom = 1;
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
            if let Some(loaded) = &self.loaded {
                // Integer physical-pixel zoom, independent of the desktop's UI scaling.
                let scale = self.zoom as f32 / ctx.pixels_per_point();
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        for (label, texture) in [
                            ("Source", &loaded.source_texture),
                            ("400 × 240 · export preview", &loaded.result_texture),
                        ] {
                            egui::Frame::group(ui.style())
                                .inner_margin(12.0)
                                .show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        ui.strong(label);
                                        ui.add_space(8.0);
                                        ui.image((texture.id(), texture.size_vec2() * scale));
                                    });
                                });
                        }
                    });
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Open an image to create your calculator's OFF screen.");
                });
            }
        });
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
            source,
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
