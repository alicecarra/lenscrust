use image::ImageReader;

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

#[derive(Default)]
struct MyApp {
    texture: Option<egui::TextureHandle>,
    texture_name: Option<String>,
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

        self.texture_name = Some(image_path.file_name().unwrap().display().to_string());

        let loaded_image: image::DynamicImage =
            ImageReader::open(&image_path).unwrap().decode().unwrap();

        let size = [
            loaded_image.width() as usize,
            loaded_image.height() as usize,
        ];
        let rgba = loaded_image.to_rgba8();
        let image_to_render =
            egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_flat_samples().as_slice());

        self.texture = Some(ui.ctx().load_texture(
            self.texture_name.as_deref().unwrap_or("image"),
            image_to_render,
            Default::default(),
        ));
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Load Image").clicked() {
                    self.load_texture(ui);
                } else if ui.button("Quit").clicked() {
                    ui.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                if let Some(texture) = &self.texture {
                    ui.image(texture);
                }
            });
        });
    }
}
