use egui::{CentralPanel, ImageSource, Pos2, Ui, Window, load::SizedTexture};
use egui_wgpu::ScreenDescriptor;

use crate::renderer::misc::RenderStage;


pub fn render_ui(
    ui: &mut Ui,
    screen_descriptor: &ScreenDescriptor, 
    render_stage: &RenderStage
) {
    let screen_width = screen_descriptor.size_in_pixels[0];
    let screen_height = screen_descriptor.size_in_pixels[1];

    Window::new("Tuff ahh menu")
    .fixed_pos(Pos2::ZERO)
    .show(ui.ctx(), |ui| {



    });

    CentralPanel::default()
    .show_inside(ui, |ui| {

        // ui.image(
        //     ImageSource::Texture(
            
        //     )
        // );

    });


}