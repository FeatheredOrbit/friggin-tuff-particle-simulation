use egui::{ImageSource, Pos2, Ui, Window, load::SizedTexture, Vec2, Area};
use egui_wgpu::ScreenDescriptor;

use crate::renderer::misc::RenderStage;
use crate::renderer::TextureIds;

pub fn render_ui(
    ui: &mut Ui,
    screen_descriptor: &ScreenDescriptor, 
    render_stage: &RenderStage,
    texture_ids: &TextureIds
) {
    let screen_width = screen_descriptor.size_in_pixels[0] as f32;
    let screen_height = screen_descriptor.size_in_pixels[1] as f32;

    Window::new("Tuff ahh menu")
    .fixed_pos(Pos2::ZERO)
    .show(ui.ctx(), |ui| {



    });

    Area::new("Texture Area".into())
    .fixed_pos(Pos2::ZERO)
    .show(ui.ctx(), |ui| {

        ui.image(
            ImageSource::Texture(
                SizedTexture::new(
                    if *render_stage == RenderStage::First { texture_ids.texture_2 } else { texture_ids.texture_1 },
                    Vec2::new(screen_width, screen_height)
                )
            )
        );

    });


}