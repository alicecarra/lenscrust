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
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let texture = self.texture.get_or_insert_with(|| {
            let loaded_image: image::DynamicImage =
                ImageReader::open("test_images/Underwater_53k.jpg")
                    .unwrap()
                    .decode()
                    .unwrap();
            let size = [
                loaded_image.width() as usize,
                loaded_image.height() as usize,
            ];
            let rgba = loaded_image.to_rgba8();
            let image_to_render =
                egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_flat_samples().as_slice());
            ui.ctx()
                .load_texture("underwater", image_to_render, Default::default())
        });

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.image(&*texture).on_hover_text_at_pointer("foobar");
            });
        });
    }
}
