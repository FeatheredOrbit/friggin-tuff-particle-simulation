use std::sync::Arc;

use egui::{Context, CornerRadius, TextureId, Visuals};
use egui_wgpu::{RendererOptions, ScreenDescriptor};
use egui_winit::State;
use wgpu::{BackendOptions, Backends, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BufferUsages, Color, CommandEncoder, ComputePipelineDescriptor, Device, DeviceDescriptor, ExperimentalFeatures, Extent3d, Features, Instance, InstanceDescriptor, InstanceFlags, Limits, MemoryBudgetThresholds, MemoryHints, Operations, PipelineCompilationOptions, PipelineLayoutDescriptor, Queue, RequestAdapterOptions, ShaderStages, Surface, SurfaceConfiguration, TextureDescriptor, TextureUsages, TextureView, TextureViewDescriptor, util::{BufferInitDescriptor, DeviceExt}, wgt::BufferDescriptor, FilterMode};
use winit::{event::WindowEvent, window::Window};

use crate::{constants::{MAX_PARTICLES, NUMBER_OF_PARTICLES}, renderer::{misc::{BindGroupLayouts, BindGroups, Buffers, ComputePipelines, RenderStage, Textures}, shader_data::{ParticleData, Uniforms}, ui::render_ui}};

pub mod shader_data;
mod misc;
mod ui;

pub struct TextureIds {
    texture_1: TextureId,
    texture_2: TextureId,
}

struct EguiRenderer {
    renderer: egui_wgpu::Renderer,
    state: State,
    context: Context,
    texture_ids: TextureIds
}
impl EguiRenderer {
    fn render(
        &mut self, 
        device: &Device, 
        queue: &Queue, 
        surface_view: &TextureView, 
        encoder: &mut CommandEncoder, 
        window: Arc<Window>,
        render_stage: &RenderStage
    ) {
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [window.inner_size().width, window.inner_size().height],
            pixels_per_point: window.scale_factor() as f32
        };

        let raw_input = self.state.take_egui_input(&window);
        let full_output = self.context.run_ui(raw_input, |ui| {
            render_ui(ui, &screen_descriptor, render_stage, &self.texture_ids);
        });

        self.state.handle_platform_output(&window, full_output.platform_output);

        let tris = self.context.tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(device, queue, *id, image_delta);
        }

        self.renderer.update_buffers(device, queue, encoder, &tris, &screen_descriptor);

        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { 
            label: None, 
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment { 
                    view: surface_view, 
                    depth_slice: None, 
                    resolve_target: None, 
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(Color::BLACK),
                        store: wgpu::StoreOp::Store
                    }
                })
            ], 
            depth_stencil_attachment: None, 
            timestamp_writes: None, 
            occlusion_query_set: None, 
            multiview_mask: None 
        });

        self.renderer.render(&mut render_pass.forget_lifetime(), &tris, &screen_descriptor);

        for texture_id in full_output.textures_delta.free {
            self.renderer.free_texture(&texture_id);
        }
    }
}

pub struct Renderer<'a> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    egui_renderer: EguiRenderer,
    render_stage: RenderStage,
    buffers: Buffers,
    bind_groups: BindGroups,
    compute_pipelines: ComputePipelines,
    window: Arc<Window>
}

impl<'a> Renderer<'a> {
    fn create_textures(
        device: &Device,
        config: &SurfaceConfiguration,
        egui_renderer: &mut egui_wgpu::Renderer,
    ) -> (Textures, TextureIds) {
        let storage_texture_1 = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: config.width / 4,
                height: config.height / 4,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[]
        });
        let storage_texture_view_1 = storage_texture_1.create_view(&Default::default());

        let storage_texture_2 = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: config.width / 4,
                height: config.height / 4,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[]
        });
        let storage_texture_view_2 = storage_texture_2.create_view(&Default::default());

        let texture1_id = egui_renderer.register_native_texture(device, &storage_texture_view_1, FilterMode::Nearest);
        let texture2_id = egui_renderer.register_native_texture(device, &storage_texture_view_2, FilterMode::Nearest);

        let texture_ids = TextureIds {
          texture_1: texture1_id,
            texture_2: texture2_id
        };

        (
            Textures {
                storage_1: storage_texture_1,
                storage_2: storage_texture_2
            },
            texture_ids
        )

    }

    fn create_buffers(device: &Device, config: &SurfaceConfiguration) -> Buffers {
        let uniforms = Uniforms {
            data_1: [NUMBER_OF_PARTICLES, config.width / 4, config.height / 4, 0]
        };
        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&uniforms),
            usage: BufferUsages::UNIFORM
        });

        let storage_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: (size_of::<ParticleData>() * MAX_PARTICLES as usize) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        Buffers {
            uniform: uniform_buffer,
            storage: storage_buffer
        }

    }

    fn create_bind_groups(
        config: &wgpu::wgt::SurfaceConfiguration<Vec<wgpu::TextureFormat>>,
        device: &Device,
        egui_renderer: &mut egui_wgpu::Renderer
    ) -> (TextureIds, Buffers, BindGroupLayouts, BindGroups) {

        let (
            textures,
            texture_ids
        ) = Self::create_textures(device, config, egui_renderer);
        let buffers = Self::create_buffers(device, config);

        let bind_group_layout_compute = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture { 
                        access: wgpu::StorageTextureAccess::WriteOnly, 
                        format: wgpu::TextureFormat::Rgba8Unorm, 
                        view_dimension: wgpu::TextureViewDimension::D2
                    },
                    count: None
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: None 
                    },
                    count: None
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                        has_dynamic_offset: false, 
                        min_binding_size: None 
                    },
                    count: None
                }
            ]
        });
        let bind_group_layout_fade_out_pipeline = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture { 
                        access: wgpu::StorageTextureAccess::ReadOnly, 
                        format: wgpu::TextureFormat::Rgba8Unorm, 
                        view_dimension: wgpu::TextureViewDimension::D2
                    },
                    count: None
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture { 
                        access: wgpu::StorageTextureAccess::WriteOnly, 
                        format: wgpu::TextureFormat::Rgba8Unorm, 
                        view_dimension: wgpu::TextureViewDimension::D2
                    },
                    count: None
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: None
                    },
                    count: None
                }
            ]
        });

        let bind_group_layouts = BindGroupLayouts {
            compute: bind_group_layout_compute,
            fade_out_compute: bind_group_layout_fade_out_pipeline
        };

        let view_1 = textures.storage_1.create_view(&TextureViewDescriptor::default());
        let view_2 = textures.storage_2.create_view(&TextureViewDescriptor::default());

        let bind_group_compute_1 = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layouts.compute,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view_1)
                },
                BindGroupEntry {
                    binding: 1,
                    resource: buffers.uniform.as_entire_binding()
                },
                BindGroupEntry {
                    binding: 2,
                    resource: buffers.storage.as_entire_binding()
                },
            ]
        });
        let bind_group_compute_2 = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layouts.compute,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view_2)
                },
                BindGroupEntry {
                    binding: 1,
                    resource: buffers.uniform.as_entire_binding()
                },
                BindGroupEntry {
                    binding: 2,
                    resource: buffers.storage.as_entire_binding()
                },
            ]
        });

        let bind_group_fade_out_1 = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layouts.fade_out_compute,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view_1)
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view_2)
                },
                BindGroupEntry {
                    binding: 2,
                    resource: buffers.uniform.as_entire_binding()
                },
            ]
        });
        let bind_group_fade_out_2 = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layouts.fade_out_compute,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view_2)
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view_1)
                },
                BindGroupEntry {
                    binding: 2,
                    resource: buffers.uniform.as_entire_binding()
                },
            ]
        });

        let bind_groups = BindGroups {
            compute_1: bind_group_compute_1,
            compute_2: bind_group_compute_2,
            fade_out_1: bind_group_fade_out_1,
            fade_out_2: bind_group_fade_out_2
        };

        (
            texture_ids,
            buffers,
            bind_group_layouts,
            bind_groups
        )
    }

    fn create_compute_pipelines(
        bind_group_layouts: &BindGroupLayouts, 
        device: &Device
    ) -> ComputePipelines {

        let main_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bind_group_layouts.compute)],
            immediate_size: 0
        });
        let fade_out_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bind_group_layouts.fade_out_compute)],
            immediate_size: 0
        });

        let module = device.create_shader_module(wgpu::include_wgsl!("../../shaders/compute/main.wgsl"));
        let main_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: None,
            layout: Some(&main_pipeline_layout),
            module: &module,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None
        });

        let module = device.create_shader_module(wgpu::include_wgsl!("../../shaders/compute/fade_out.wgsl"));
        let fade_out_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: None,
            layout: Some(&fade_out_pipeline_layout),
            module: &module,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None
        });

        ComputePipelines {
            main: main_pipeline,
            fade_out: fade_out_pipeline
        }
    }

    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            flags: InstanceFlags::all(),
            memory_budget_thresholds: MemoryBudgetThresholds::default(),
            backend_options: BackendOptions::default(),
            display: None
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::None,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface)
        }).await?;

        let (device, queue) = adapter.request_device(&DeviceDescriptor {
            label: None,
            required_features: Features::empty(),
            required_limits: Limits::defaults(),
            experimental_features: ExperimentalFeatures::disabled(),
            memory_hints: MemoryHints::default(),
            trace: wgpu::Trace::Off
        }).await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let mut renderer_egui = egui_wgpu::Renderer::new(&device, config.format, RendererOptions::default());

        let mut visuals = Visuals::dark();
        visuals.window_corner_radius = CornerRadius::ZERO;

        let context = Context::default();
        context.set_visuals(visuals);

        let state = State::new(context.clone(), context.viewport_id(), &window, None, None, None);

        let (
            texture_ids,
            buffers,
            bind_group_layouts,
            bind_groups
        ) = Self::create_bind_groups(&config, &device, &mut renderer_egui);

        let compute_pipelines = Self::create_compute_pipelines(&bind_group_layouts, &device);

        surface.configure(&device, &config);

        let egui_renderer = EguiRenderer {
            renderer: renderer_egui,
            state,
            context,
            texture_ids
        };

        Ok(
            Self {
                surface,
                device,
                queue,
                config,
                egui_renderer,
                render_stage: RenderStage::First,
                buffers,
                bind_groups,
                compute_pipelines,
                window
            }
        )

    }

    pub fn resize(&mut self, width: u32, height: u32) {
    if width > 0 && height > 0 {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }
}

    pub fn set_particles(&mut self, particles: Vec<ParticleData>) {
        let particles: &[u8] = bytemuck::cast_slice(particles.as_slice());
        
        self.queue.write_buffer(&self.buffers.storage, 0, particles);
    }

    pub fn take_winit_event(&mut self, window: Arc<Window>, window_event: WindowEvent) {
        let _ = self.egui_renderer.state.on_window_event(&window, &window_event);
    }

    pub fn render(&mut self, window: Arc<Window>) -> anyhow::Result<()> {
        self.window.request_redraw();

        let mut texture_is_suboptimal = false;

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,

            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                texture_is_suboptimal = true;
                surface_texture
            }

            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Validation => {
                return Ok(());
            }

            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                anyhow::bail!("Lost device");
            }
        };

        let view = output.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { 
            label: Some("Compute Pass"), 
            timestamp_writes: None
        });

        let fade_out_bind_group = if self.render_stage == RenderStage::First {
            &self.bind_groups.fade_out_2
        } else {
            &self.bind_groups.fade_out_1
        };

        compute_pass.set_bind_group(0, Some(fade_out_bind_group), &[]);
        compute_pass.set_pipeline(&self.compute_pipelines.fade_out);
        compute_pass.dispatch_workgroups((self.config.width + 7) / 8, (self.config.height + 7) / 8, 1);

        let compute_bind_group = if self.render_stage == RenderStage::First {
            &self.bind_groups.compute_1
        } else {
            &self.bind_groups.compute_2
        };

        compute_pass.set_bind_group(0, Some(compute_bind_group), &[]);
        compute_pass.set_pipeline(&self.compute_pipelines.main);
        compute_pass.dispatch_workgroups((NUMBER_OF_PARTICLES + 63) / 64, 1, 1);

        drop(compute_pass);

        self.egui_renderer.render(&self.device, &self.queue, &view, &mut encoder, window, &self.render_stage);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        match self.render_stage {
            RenderStage::First => {self.render_stage = RenderStage::Second},
            RenderStage::Second => {self.render_stage = RenderStage::First}
        }

        if texture_is_suboptimal {
            self.surface.configure(&self.device, &self.config);
        }

        Ok(())

    }
}