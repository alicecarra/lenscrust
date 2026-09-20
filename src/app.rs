use image::{DynamicImage, ImageReader};

use crate::errors::AppError;
use crate::operations::{Kernel, Operation, compute_histogram};

const IDENTITY_KERNEL: [[f64; 3]; 3] = [[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.0]];

#[derive(Clone, Copy, PartialEq)]
enum ConvolutionKernelType {
    GaussianLowPass,
    LaplacianHighPass,
    GenericHighPass,
    PrewittHorizontalGradient,
    PrewittVerticalGradient,
    SobelHorizontalGradient,
    SobelVerticalGradient,
    Custom,
}

impl ConvolutionKernelType {
    const ALL: [ConvolutionKernelType; 8] = [
        ConvolutionKernelType::GaussianLowPass,
        ConvolutionKernelType::LaplacianHighPass,
        ConvolutionKernelType::GenericHighPass,
        ConvolutionKernelType::PrewittHorizontalGradient,
        ConvolutionKernelType::PrewittVerticalGradient,
        ConvolutionKernelType::SobelHorizontalGradient,
        ConvolutionKernelType::SobelVerticalGradient,
        ConvolutionKernelType::Custom,
    ];

    fn label(&self) -> &'static str {
        match self {
            ConvolutionKernelType::GaussianLowPass => "Gaussian (low-pass / blur)",
            ConvolutionKernelType::LaplacianHighPass => "Laplacian (high-pass)",
            ConvolutionKernelType::GenericHighPass => "Generic high-pass",
            ConvolutionKernelType::PrewittHorizontalGradient => "Prewitt horizontal gradient",
            ConvolutionKernelType::PrewittVerticalGradient => "Prewitt vertical gradient",
            ConvolutionKernelType::SobelHorizontalGradient => "Sobel horizontal gradient",
            ConvolutionKernelType::SobelVerticalGradient => "Sobel vertical gradient",
            ConvolutionKernelType::Custom => "Custom",
        }
    }

    fn to_kernel(self, custom_weights: [[f64; 3]; 3]) -> Kernel {
        match self {
            ConvolutionKernelType::GaussianLowPass => Kernel::GAUSSIAN_LOW_PASS,
            ConvolutionKernelType::LaplacianHighPass => Kernel::POSITIVE_LAPLACIAN_HIGH_PASS,
            ConvolutionKernelType::GenericHighPass => Kernel::NEGATIVE_LAPLACIAN_HIGH_PASS,
            ConvolutionKernelType::PrewittHorizontalGradient => Kernel::PREWITT_HORIZONTAL_GRADIENT,
            ConvolutionKernelType::PrewittVerticalGradient => Kernel::PREWITT_VERTICAL_GRADIENT,
            ConvolutionKernelType::SobelHorizontalGradient => Kernel::SOBEL_HORIZONTAL_GRADIENT,
            ConvolutionKernelType::SobelVerticalGradient => Kernel::SOBEL_VERTICAL_GRADIENT,
            ConvolutionKernelType::Custom => Kernel {
                weights: custom_weights,
                bias_before_clamping: false,
            },
        }
    }
}

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
    reference: ImageSlot,

    quantization_levels: u16,
    jpeg_quality: u8,

    brightness_delta: i16,
    contrast_factor: f64,

    zoom_out_factor_x: f64,
    zoom_out_factor_y: f64,

    convolution_kernel_choice: ConvolutionKernelType,
    custom_convolution_weights: [[f64; 3]; 3],

    histogram: Option<[u32; 256]>,
    loaded_histogram: Option<[u32; 256]>,

    last_error: Option<AppError>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            loaded: ImageSlot::default(),
            loaded_name: None,
            edited: ImageSlot::default(),
            reference: ImageSlot::default(),
            quantization_levels: 256,
            jpeg_quality: 85,
            brightness_delta: 0,
            contrast_factor: 1.0,
            zoom_out_factor_x: 2.0,
            zoom_out_factor_y: 2.0,
            convolution_kernel_choice: ConvolutionKernelType::GaussianLowPass,
            custom_convolution_weights: IDENTITY_KERNEL,
            histogram: None,
            loaded_histogram: None,
            last_error: None,
        }
    }
}

fn to_color_image(image: &DynamicImage) -> egui::ColorImage {
    let size = [image.width() as usize, image.height() as usize];
    let rgba = image.to_rgba8();
    egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_flat_samples().as_slice())
}

fn pick_and_decode_image() -> Result<Option<(String, DynamicImage)>, AppError> {
    let current_dir = std::env::current_dir().map_err(AppError::CurrentDir)?;

    let Some(image_path) = rfd::FileDialog::new()
        .add_filter("image", &["jpg", "png"])
        .set_directory(&current_dir)
        .pick_file()
    else {
        return Ok(None);
    };

    let name = image_path
        .file_name()
        .map(|name| name.display().to_string())
        .unwrap_or_else(|| "image".to_string());

    let reader = ImageReader::open(&image_path).map_err(|source| AppError::OpenImage {
        name: name.clone(),
        source,
    })?;
    let decoded_image = reader.decode().map_err(|source| AppError::DecodeImage {
        name: name.clone(),
        source,
    })?;

    Ok(Some((name, decoded_image)))
}

fn load_texture_for(ui: &egui::Ui, label: &str, image: &DynamicImage) -> egui::TextureHandle {
    let color_image = to_color_image(image);
    ui.ctx()
        .load_texture(label, color_image, Default::default())
}

fn render_histogram(ui: &mut egui::Ui, histogram: &[u32; 256]) {
    const BAR_WIDTH: f32 = 2.0;

    let max_count = histogram.iter().copied().max().unwrap_or(0).max(1) as f32;
    let desired_size = egui::vec2(256.0 * BAR_WIDTH, 256.0);
    let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);

    for (tone, &count) in histogram.iter().enumerate() {
        let bar_height = (count as f32 / max_count) * rect.height();
        let bar_top_left = egui::pos2(
            rect.left() + tone as f32 * BAR_WIDTH,
            rect.bottom() - bar_height,
        );
        let bar_bottom_right = egui::pos2(bar_top_left.x + BAR_WIDTH, rect.bottom());
        painter.rect_filled(
            egui::Rect::from_min_max(bar_top_left, bar_bottom_right),
            0.0,
            ui.visuals().text_color(),
        );
    }
}

impl App {
    fn load_image(&mut self, ui: &mut egui::Ui) -> Result<(), AppError> {
        let Some((name, decoded_image)) = pick_and_decode_image()? else {
            return Ok(());
        };

        let texture = load_texture_for(ui, "loaded", &decoded_image);
        self.loaded_name = Some(name);
        self.loaded_histogram = Some(compute_histogram(&decoded_image));
        self.loaded.image = Some(decoded_image);
        self.loaded.texture = Some(texture);
        self.reset_edited_image();

        Ok(())
    }

    fn load_reference_image(&mut self, ui: &mut egui::Ui) -> Result<(), AppError> {
        let Some((_name, mut decoded_image)) = pick_and_decode_image()? else {
            return Ok(());
        };

        // the reference image is only ever used for grayscale histogram
        // matching, so keep its preview consistent with what's actually used
        Operation::Luminance.apply(&mut decoded_image);
        let texture = load_texture_for(ui, "reference", &decoded_image);

        self.reference.image = Some(decoded_image);
        self.reference.texture = Some(texture);

        Ok(())
    }

    fn reset_edited_image(&mut self) {
        self.edited.image = self.loaded.image.clone();
        self.edited.texture = self.loaded.texture.clone();
        self.update_histogram();
    }

    fn update_histogram(&mut self) {
        self.histogram = self.edited.image.as_ref().map(compute_histogram);
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
            self.update_histogram();
        }
    }

    fn commit_edited_as_loaded(&mut self) {
        self.loaded.image = self.edited.image.clone();
        self.loaded.texture = self.edited.texture.clone();
        self.loaded_histogram = self.histogram;
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
            let encoder =
                image::codecs::jpeg::JpegEncoder::new_with_quality(file, self.jpeg_quality);
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

        if let Some(texture) = &self.reference.texture {
            egui::Window::new("Reference")
                .default_pos([780.0, 300.0])
                .auto_sized()
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::both()
                        .id_salt("reference_scroll")
                        .auto_shrink([true, true])
                        .show(ui, |ui| {
                            ui.image(texture);
                        });
                });
        }

        if let Some(histogram) = &self.loaded_histogram {
            egui::Window::new("Original Histogram (Grayscale)")
                .default_pos([440.0, 20.0])
                .default_open(false)
                .show(ui.ctx(), |ui| render_histogram(ui, histogram));
        }

        if let Some(histogram) = &self.histogram {
            egui::Window::new("Edited Histogram (Grayscale)")
                .default_pos([440.0, 60.0])
                .default_open(false)
                .show(ui.ctx(), |ui| render_histogram(ui, histogram));
        }

        egui::Window::new("File")
            .default_pos([20.0, 20.0])
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Load Image").clicked() {
                        if let Err(err) = self.load_image(ui) {
                            self.last_error = Some(err);
                        }
                    }
                    if ui
                        .add_enabled(self.edited.is_some(), egui::Button::new("Save Image"))
                        .clicked()
                    {
                        if let Err(err) = self.save_image() {
                            self.last_error = Some(err);
                        }
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("Load Reference Image").clicked() {
                        if let Err(err) = self.load_reference_image(ui) {
                            self.last_error = Some(err);
                        }
                    }
                });

                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(self.loaded.is_some(), egui::Button::new("Reset to loaded"))
                        .clicked()
                    {
                        self.reset_edited_image();
                    }
                    if ui
                        .add_enabled(
                            self.edited.is_some(),
                            egui::Button::new("Commit as Original"),
                        )
                        .clicked()
                    {
                        self.commit_edited_as_loaded();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("JPEG quality:");
                    ui.add(egui::Slider::new(&mut self.jpeg_quality, 1..=100));
                });
            });

        if self.loaded.is_some() {
            egui::Window::new("Operations")
                .default_pos([20.0, 160.0])
                .show(ui.ctx(), |ui| {
                    if ui.button("Mirror Horizontal").clicked() {
                        self.apply_operation(ui.ctx(), Operation::MirrorHorizontal);
                    }
                    if ui.button("Mirror Vertical").clicked() {
                        self.apply_operation(ui.ctx(), Operation::MirrorVertical);
                    }
                    if ui.button("Rotate 90° Clockwise").clicked() {
                        self.apply_operation(ui.ctx(), Operation::RotateClockwise);
                    }
                    if ui.button("Rotate 90° Counterclockwise").clicked() {
                        self.apply_operation(ui.ctx(), Operation::RotateCounterclockwise);
                    }

                    ui.horizontal(|ui| {
                        ui.label("Zoom out factor X:");
                        ui.add(egui::Slider::new(&mut self.zoom_out_factor_x, 1.0..=10.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Zoom out factor Y:");
                        ui.add(egui::Slider::new(&mut self.zoom_out_factor_y, 1.0..=10.0));
                    });
                    if ui.button("Apply Zoom Out").clicked() {
                        self.apply_operation(
                            ui.ctx(),
                            Operation::ZoomOut {
                                factor_x: self.zoom_out_factor_x,
                                factor_y: self.zoom_out_factor_y,
                            },
                        );
                    }
                    if ui.button("Apply Zoom In (2x2)").clicked() {
                        self.apply_operation(ui.ctx(), Operation::ZoomIn);
                    }

                    ui.separator();

                    if ui.button("Luminance").clicked() {
                        self.apply_operation(ui.ctx(), Operation::Luminance);
                    }
                    if ui.button("Negative").clicked() {
                        self.apply_operation(ui.ctx(), Operation::Negative);
                    }

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Brightness:");
                        ui.add(egui::Slider::new(&mut self.brightness_delta, -255..=255));
                    });
                    if ui.button("Apply Brightness").clicked() {
                        self.apply_operation(
                            ui.ctx(),
                            Operation::Brightness(self.brightness_delta),
                        );
                    }

                    ui.horizontal(|ui| {
                        ui.label("Contrast:");
                        ui.add(egui::Slider::new(&mut self.contrast_factor, 0.01..=5.0));
                    });
                    if ui.button("Apply Contrast").clicked() {
                        self.apply_operation(ui.ctx(), Operation::Contrast(self.contrast_factor));
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

                    if ui.button("Equalize Histogram").clicked() {
                        self.apply_operation(ui.ctx(), Operation::EqualizeHistogram);
                    }
                    if ui
                        .add_enabled(
                            self.reference.is_some(),
                            egui::Button::new("Match Histogram"),
                        )
                        .clicked()
                    {
                        if let Some(reference_image) = self.reference.image.clone() {
                            self.apply_operation(
                                ui.ctx(),
                                Operation::MatchHistogram(reference_image),
                            );
                        }
                    }

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Convolution kernel:");
                        egui::ComboBox::from_id_salt("convolution_kernel_choice")
                            .selected_text(self.convolution_kernel_choice.label())
                            .show_ui(ui, |ui| {
                                for choice in ConvolutionKernelType::ALL {
                                    ui.selectable_value(
                                        &mut self.convolution_kernel_choice,
                                        choice,
                                        choice.label(),
                                    );
                                }
                            });
                    });
                    if self.convolution_kernel_choice == ConvolutionKernelType::Custom {
                        egui::Grid::new("custom_convolution_kernel_3x3_grid").show(ui, |ui| {
                            for row in 0..3 {
                                for column in 0..3 {
                                    ui.add(
                                        egui::DragValue::new(
                                            &mut self.custom_convolution_weights[row][column],
                                        )
                                        .speed(0.05),
                                    );
                                }
                                ui.end_row();
                            }
                        });
                    }
                    if ui.button("Apply Convolution").clicked() {
                        let kernel = self
                            .convolution_kernel_choice
                            .to_kernel(self.custom_convolution_weights);
                        self.apply_operation(ui.ctx(), Operation::Convolve(kernel));
                    }
                });
        }
    }
}
