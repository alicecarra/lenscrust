use image::{GenericImage, GenericImageView, ImageReader};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "LensCrust",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx); // image support
            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    loaded_texture: Option<egui::TextureHandle>,
    loaded_texture_name: Option<String>,
    loaded_image: Option<image::DynamicImage>,

    edited_texture: Option<egui::TextureHandle>,
    edited_image: Option<image::DynamicImage>,

    quantization_levels: u16,
    jpeg_quality: u8,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            loaded_texture: None,
            loaded_texture_name: None,
            loaded_image: None,
            edited_texture: None,
            edited_image: None,
            quantization_levels: 256,
            jpeg_quality: 85,
        }
    }
}

impl MyApp {
    fn load_texture(&mut self, ui: &mut egui::Ui) {
        let current_dir = std::env::current_dir().unwrap();

        let Some(image_path) = rfd::FileDialog::new()
            .add_filter("image", &["jpg", "png"])
            .set_directory(&current_dir)
            .pick_file()
        else {
            return;
        };

        self.loaded_texture_name = Some(image_path.file_name().unwrap().display().to_string());

        let loaded_image: image::DynamicImage =
            ImageReader::open(&image_path).unwrap().decode().unwrap();

        let size = [
            loaded_image.width() as usize,
            loaded_image.height() as usize,
        ];
        let rgba = loaded_image.to_rgba8();
        let image_to_render =
            egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_flat_samples().as_slice());

        self.loaded_texture = Some(ui.ctx().load_texture(
            self.loaded_texture_name.as_deref().unwrap_or("image"),
            image_to_render,
            Default::default(),
        ));
        self.loaded_image = Some(loaded_image);
        self.reset_edited_image();
    }

    fn reset_edited_image(&mut self) {
        self.edited_image = self.loaded_image.clone();
        self.edited_texture = self.loaded_texture.clone();
    }

    fn update_edited_texture(&mut self, ctx: &egui::Context) {
        let Some(image) = &self.edited_image else {
            return;
        };
        let size = [image.width() as usize, image.height() as usize];
        let rgba = image.to_rgba8();
        let color_image =
            egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_flat_samples().as_slice());
        self.edited_texture = Some(ctx.load_texture("edited", color_image, Default::default()));
    }

    fn save_image(&mut self) {
        let Some(image) = &self.loaded_image else {
            return;
        };

        let Some(save_path) = rfd::FileDialog::new()
            .add_filter("image", &["jpg", "png"])
            .set_file_name(self.loaded_texture_name.as_deref().unwrap_or("image.png"))
            .save_file()
        else {
            return;
        };

        let is_jpeg = save_path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("jpg") || ext.eq_ignore_ascii_case("jpeg"));

        if is_jpeg {
            let file = std::fs::File::create(&save_path).unwrap();
            let encoder =
                image::codecs::jpeg::JpegEncoder::new_with_quality(file, self.jpeg_quality);
            image.write_with_encoder(encoder).unwrap();
        } else {
            image.save(save_path).unwrap();
        }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Load Image").clicked() {
                    self.load_texture(ui);
                } else if ui
                    .add_enabled(self.loaded_image.is_some(), egui::Button::new("Save Image"))
                    .clicked()
                {
                    self.save_image();
                } else if ui.button("Quit").clicked() {
                    ui.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });

        egui::CentralPanel::default().show(ui, |_ui| {});

        if let Some(texture) = &self.loaded_texture {
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

        if let Some(texture) = &self.edited_texture {
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
                ui.add_enabled_ui(self.loaded_image.is_some(), |ui| {
                    if ui.button("Reset").clicked() {
                        self.reset_edited_image();
                    }

                    ui.separator();

                    if ui.button("Mirror Horizontal").clicked() {
                        mirror_horizontal(self.edited_image.as_mut().unwrap());
                        self.update_edited_texture(ui.ctx());
                    }
                    if ui.button("Mirror Vertical").clicked() {
                        mirror_vertical(self.edited_image.as_mut().unwrap());
                        self.update_edited_texture(ui.ctx());
                    }
                    if ui.button("Luminance").clicked() {
                        luminance(self.edited_image.as_mut().unwrap());
                        self.update_edited_texture(ui.ctx());
                    }

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Quantization levels:");
                        ui.add(egui::Slider::new(&mut self.quantization_levels, 1..=256));
                    });
                    if ui.button("Apply Quantization").clicked() {
                        // force image to be in grayscale
                        luminance(self.edited_image.as_mut().unwrap());

                        quantize(
                            self.edited_image.as_mut().unwrap(),
                            self.quantization_levels,
                        );
                        self.update_edited_texture(ui.ctx());
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

fn mirror_horizontal(image: &mut image::DynamicImage) {
    let (width, height) = image.dimensions();

    for y in 0..height {
        for x in 0..width / 2 {
            let left = image.get_pixel(x, y);
            let right = image.get_pixel(width - 1 - x, y);
            image.put_pixel(x, y, right);
            image.put_pixel(width - 1 - x, y, left);
        }
    }
}

fn mirror_vertical(image: &mut image::DynamicImage) {
    let (width, height) = image.dimensions();

    for y in 0..height / 2 {
        for x in 0..width {
            let top = image.get_pixel(x, y);
            let bottom = image.get_pixel(x, height - 1 - y);
            image.put_pixel(x, y, bottom);
            image.put_pixel(x, height - 1 - y, top);
        }
    }
}

fn luminance(image: &mut image::DynamicImage) {
    match image {
        // already on grayscale, do nothing
        image::DynamicImage::ImageLuma8(..)
        | image::DynamicImage::ImageLumaA8(..)
        | image::DynamicImage::ImageLuma16(..)
        | image::DynamicImage::ImageLumaA16(..) => return,
        _ => {
            let mut gray = image::GrayImage::new(image.width(), image.height());
            for x in 0..image.width() {
                for y in 0..image.height() {
                    let pixel = image.get_pixel(x, y);
                    let gray_pixel = (pixel[0] as f64 * 0.299
                        + pixel[1] as f64 * 0.587
                        + pixel[2] as f64 * 0.114)
                        .round() as u8;
                    gray.put_pixel(x, y, image::Luma([gray_pixel]));
                }
            }
            *image = image::DynamicImage::ImageLuma8(gray);
        }
    }
}

fn quantize(image: &mut image::DynamicImage, levels: u16) {
    let luma_image = image.as_mut_luma8().expect("Image not in luma!!!");

    // maybe use a more idiomatic way?
    let mut min_tone = 255;
    let mut max_tone = 0;
    for pixel in luma_image.pixels() {
        let pixel = pixel[0];
        if pixel > max_tone {
            max_tone = pixel;
        }
        if pixel < min_tone {
            min_tone = pixel;
        }
    }
    // sanity check
    assert!(max_tone >= min_tone);

    let tone_range_size = (max_tone - min_tone) as u16 + 1;

    if levels >= tone_range_size {
        // maybe give feedback to user?
        return;
    }

    let bin_width = tone_range_size as f64 / levels as f64;

    for pixel in luma_image.pixels_mut() {
        let t_orig = pixel[0] as f64;
        let bin_index = (((t_orig - (min_tone as f64 - 0.5)) / bin_width).floor() as i64)
            .clamp(0, levels as i64 - 1);
        let quantized_tone = (min_tone as f64 - 0.5) + (bin_index as f64 + 0.5) * bin_width;
        pixel[0] = quantized_tone.round() as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, GrayImage, Luma, Rgb, RgbImage};
    use std::collections::HashSet;

    fn make_rgb_image(width: u32, height: u32, pixels: &[[u8; 3]]) -> DynamicImage {
        let mut img = RgbImage::new(width, height);
        for (i, p) in pixels.iter().enumerate() {
            let x = (i as u32) % width;
            let y = (i as u32) / width;
            img.put_pixel(x, y, Rgb(*p));
        }
        DynamicImage::ImageRgb8(img)
    }

    fn make_gray_image(width: u32, height: u32, pixels: &[u8]) -> DynamicImage {
        let mut img = GrayImage::new(width, height);
        for (i, &p) in pixels.iter().enumerate() {
            let x = (i as u32) % width;
            let y = (i as u32) / width;
            img.put_pixel(x, y, Luma([p]));
        }
        DynamicImage::ImageLuma8(img)
    }

    #[test]
    fn mirror_horizontal_reverses_rows() {
        let mut img = make_rgb_image(
            3,
            2,
            &[[1, 1, 1], [2, 2, 2], [3, 3, 3], [4, 4, 4], [5, 5, 5], [6, 6, 6]],
        );
        mirror_horizontal(&mut img);
        let expected = make_rgb_image(
            3,
            2,
            &[[3, 3, 3], [2, 2, 2], [1, 1, 1], [6, 6, 6], [5, 5, 5], [4, 4, 4]],
        );
        assert_eq!(img.to_rgba8(), expected.to_rgba8());
    }

    #[test]
    fn mirror_horizontal_twice_is_identity() {
        let original = make_rgb_image(
            3,
            2,
            &[[1, 1, 1], [2, 2, 2], [3, 3, 3], [4, 4, 4], [5, 5, 5], [6, 6, 6]],
        );
        let mut img = original.clone();
        mirror_horizontal(&mut img);
        mirror_horizontal(&mut img);
        assert_eq!(img.to_rgba8(), original.to_rgba8());
    }

    #[test]
    fn mirror_vertical_reverses_columns() {
        let mut img = make_rgb_image(
            2,
            3,
            &[[1, 1, 1], [2, 2, 2], [3, 3, 3], [4, 4, 4], [5, 5, 5], [6, 6, 6]],
        );
        mirror_vertical(&mut img);
        let expected = make_rgb_image(
            2,
            3,
            &[[5, 5, 5], [6, 6, 6], [3, 3, 3], [4, 4, 4], [1, 1, 1], [2, 2, 2]],
        );
        assert_eq!(img.to_rgba8(), expected.to_rgba8());
    }

    #[test]
    fn mirror_vertical_twice_is_identity() {
        let original = make_rgb_image(
            2,
            3,
            &[[1, 1, 1], [2, 2, 2], [3, 3, 3], [4, 4, 4], [5, 5, 5], [6, 6, 6]],
        );
        let mut img = original.clone();
        mirror_vertical(&mut img);
        mirror_vertical(&mut img);
        assert_eq!(img.to_rgba8(), original.to_rgba8());
    }

    #[test]
    fn luminance_applies_bt601_formula() {
        // 0.299*255 = 76.245 -> 76 ; 0.587*255 = 149.685 -> 150
        // 0.114*255 = 29.07 -> 29 ; sum of weights * 255 = 255
        let mut img = make_rgb_image(
            2,
            2,
            &[[255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 255]],
        );
        luminance(&mut img);
        let gray = img.as_luma8().expect("image should be grayscale");
        assert_eq!(gray.get_pixel(0, 0)[0], 76);
        assert_eq!(gray.get_pixel(1, 0)[0], 150);
        assert_eq!(gray.get_pixel(0, 1)[0], 29);
        assert_eq!(gray.get_pixel(1, 1)[0], 255);
    }

    #[test]
    fn luminance_is_noop_on_already_grayscale() {
        let mut img = make_gray_image(2, 1, &[10, 20]);
        luminance(&mut img);
        let gray = img.as_luma8().unwrap();
        assert_eq!(gray.get_pixel(0, 0)[0], 10);
        assert_eq!(gray.get_pixel(1, 0)[0], 20);
    }

    #[test]
    fn quantize_noop_when_levels_cover_full_range() {
        // descending order so the min/max scan tracks correctly (see the bug
        // test below): min=10, max=20, tone_range_size=11
        let mut img = make_gray_image(2, 1, &[20, 10]);
        quantize(&mut img, 11);
        let gray = img.as_luma8().unwrap();
        assert_eq!(gray.get_pixel(0, 0)[0], 20);
        assert_eq!(gray.get_pixel(1, 0)[0], 10);
    }

    #[test]
    fn quantize_produces_at_most_requested_levels() {
        let pixels: Vec<u8> = (0..16).map(|x| (x * 17) as u8).collect();
        let mut img = make_gray_image(16, 1, &pixels);
        quantize(&mut img, 4);
        let gray = img.as_luma8().unwrap();
        let distinct: HashSet<u8> = gray.pixels().map(|p| p[0]).collect();
        assert!(distinct.len() <= 4);
    }

    #[test]
    fn quantize_tracks_min_even_when_first_pixel_is_the_minimum() {
        // Regression test: the min/max scan used to use `if .. else if ..`,
        // so a nonzero first pixel that was also the true minimum (like 10
        // here) never got checked against min_tone, leaving it stuck at 255
        // and crashing the later `assert!(max_tone >= min_tone)`.
        let mut img = make_gray_image(2, 1, &[10, 200]);
        quantize(&mut img, 200); // tone_range_size = 191, levels(200) >= it -> no-op
        let gray = img.as_luma8().unwrap();
        assert_eq!(gray.get_pixel(0, 0)[0], 10);
        assert_eq!(gray.get_pixel(1, 0)[0], 200);
    }
}
