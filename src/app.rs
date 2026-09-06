use image::{DynamicImage, ImageReader};

use crate::errors::AppError;
use crate::ops::Operation;

#[derive(Default)]
struct ImageSlot {
    image: Option<DynamicImage>,
    texture: Option<egui::TextureHandle>,
}

impl ImageSlot {
    fn is_some(&self) -> bool {
        self.image.is_some()
    }
}

pub struct App {
    loaded: ImageSlot,
    loaded_name: Option<String>,
    edited: ImageSlot,

    quantization_levels: u16,
    jpeg_quality: u8,

    last_error: Option<AppError>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            loaded: ImageSlot::default(),
            loaded_name: None,
            edited: ImageSlot::default(),
            quantization_levels: 256,
            jpeg_quality: 85,
            last_error: None,
        }
    }
}

fn to_color_image(image: &DynamicImage) -> egui::ColorImage {
    let size = [image.width() as usize, image.height() as usize];
    let rgba = image.to_rgba8();
    egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_flat_samples().as_slice())
}

impl App {
    fn load_image(&mut self, ui: &mut egui::Ui) -> Result<(), AppError> {
        let current_dir = std::env::current_dir().map_err(AppError::CurrentDir)?;

        let Some(image_path) = rfd::FileDialog::new()
            .add_filter("image", &["jpg", "png"])
            .set_directory(&current_dir)
            .pick_file()
        else {
            return Ok(());
        };

        let name = image_path
            .file_name()
            .map(|name| name.display().to_string())
            .unwrap_or_else(|| "image".to_string());

        let reader = ImageReader::open(&image_path).map_err(|source| AppError::OpenImage {
            name: name.clone(),
            source,
        })?;
        let loaded_image = reader
            .decode()
            .map_err(|source| AppError::DecodeImage { name: name.clone(), source })?;

        self.loaded_name = Some(name);

        let image_to_render = to_color_image(&loaded_image);
        self.loaded.texture = Some(ui.ctx().load_texture(
            self.loaded_name.as_deref().unwrap_or("image"),
            image_to_render,
            Default::default(),
        ));
        self.loaded.image = Some(loaded_image);
        self.reset_edited_image();

        Ok(())
    }

    fn reset_edited_image(&mut self) {
        self.edited.image = self.loaded.image.clone();
        self.edited.texture = self.loaded.texture.clone();
    }

    fn update_edited_texture(&mut self, ctx: &egui::Context) {
        let Some(image) = &self.edited.image else {
            return;
        };
        let color_image = to_color_image(image);
        self.edited.texture = Some(ctx.load_texture("edited", color_image, Default::default()));
    }

    fn apply_operation(&mut self, ctx: &egui::Context, operation: Operation) {
        if let Some(image) = self.edited.image.as_mut() {
            operation.apply(image);
            self.update_edited_texture(ctx);
        }
    }

    fn save_image(&mut self) -> Result<(), AppError> {
        let Some(image) = &self.edited.image else {
            return Ok(());
        };

        let Some(save_path) = rfd::FileDialog::new()
            .add_filter("image", &["jpg", "png"])
            .set_file_name(self.loaded_name.as_deref().unwrap_or("image.png"))
            .save_file()
        else {
            return Ok(());
        };

        let is_jpeg = save_path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("jpg") || ext.eq_ignore_ascii_case("jpeg"));

        if is_jpeg {
            let file = std::fs::File::create(&save_path).map_err(AppError::CreateFile)?;
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(file, self.jpeg_quality);
            image
                .write_with_encoder(encoder)
                .map_err(AppError::SaveImage)?;
        } else {
            image.save(&save_path).map_err(AppError::SaveImage)?;
        }

        Ok(())
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Load Image").clicked() {
                    if let Err(err) = self.load_image(ui) {
                        self.last_error = Some(err);
                    }
                } else if ui
                    .add_enabled(self.edited.is_some(), egui::Button::new("Save Image"))
                    .clicked()
                {
                    if let Err(err) = self.save_image() {
                        self.last_error = Some(err);
                    }
                } else if ui.button("Quit").clicked() {
                    ui.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });

        egui::CentralPanel::default().show(ui, |_ui| {});

        if self.last_error.is_some() {
            let mut dismiss = false;
            egui::Window::new("Error")
                .default_pos([20.0, 260.0])
                .show(ui.ctx(), |ui| {
                    if let Some(err) = &self.last_error {
                        ui.colored_label(egui::Color32::RED, err.to_string());
                    }
                    dismiss = ui.button("Dismiss").clicked();
                });
            if dismiss {
                self.last_error = None;
            }
        }

        if let Some(texture) = &self.loaded.texture {
            egui::Window::new("Original")
                .default_pos([20.0, 300.0])
                .auto_sized()
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::both()
                        .id_salt("original_scroll")
                        .auto_shrink([true, true])
                        .show(ui, |ui| {
                            ui.image(texture);
                        });
                });
        }

        if let Some(texture) = &self.edited.texture {
            egui::Window::new("Edited")
                .default_pos([440.0, 300.0])
                .auto_sized()
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::both()
                        .id_salt("edited_scroll")
                        .auto_shrink([true, true])
                        .show(ui, |ui| {
                            ui.image(texture);
                        });
                });
        }

        egui::Window::new("Controls")
            .default_pos([20.0, 20.0])
            .show(ui.ctx(), |ui| {
                ui.add_enabled_ui(self.loaded.is_some(), |ui| {
                    if ui.button("Reset").clicked() {
                        self.reset_edited_image();
                    }

                    ui.separator();

                    if ui.button("Mirror Horizontal").clicked() {
                        self.apply_operation(ui.ctx(), Operation::MirrorHorizontal);
                    }
                    if ui.button("Mirror Vertical").clicked() {
                        self.apply_operation(ui.ctx(), Operation::MirrorVertical);
                    }
                    if ui.button("Luminance").clicked() {
                        self.apply_operation(ui.ctx(), Operation::Luminance);
                    }

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Quantization levels:");
                        ui.add(egui::Slider::new(&mut self.quantization_levels, 1..=256));
                    });
                    if ui.button("Apply Quantization").clicked() {
                        self.apply_operation(
                            ui.ctx(),
                            Operation::Quantize(self.quantization_levels),
                        );
                    }

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("JPEG quality:");
                        ui.add(egui::Slider::new(&mut self.jpeg_quality, 1..=100));
                    });
                });
            });
    }
}
