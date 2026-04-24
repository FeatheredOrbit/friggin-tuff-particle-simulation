use std::sync::Arc;

use wgpu::{BackendOptions, Backends, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BufferUsages, ColorTargetState, ColorWrites, ComputePipelineDescriptor, Device, DeviceDescriptor, ExperimentalFeatures, Extent3d, Features, FragmentState, Instance, InstanceDescriptor, InstanceFlags, Limits, MemoryBudgetThresholds, MemoryHints, MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPipelineDescriptor, RequestAdapterOptions, SamplerDescriptor, ShaderStages, Surface, SurfaceConfiguration, TextureDescriptor, TextureUsages, TextureViewDescriptor, VertexState, util::{BufferInitDescriptor, DeviceExt}, wgt::BufferDescriptor};
use winit::window::Window;

use crate::{constants::{MAX_PARTICLES, NUMBER_OF_PARTICLES}, renderer::{misc::{BindGroupLayouts, BindGroups, Buffers, ComputePipelines, RenderPipelines, RenderStage, Textures}, shader_data::{ParticleData, Uniforms}}};

pub mod shader_data;
mod misc;

pub struct Renderer<'a> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    render_stage: RenderStage,
    buffers: Buffers,
    bind_groups: BindGroups,
    render_pipelines: RenderPipelines,
    compute_pipelines: ComputePipelines,
    window: Arc<Window>
}

impl<'a> Renderer<'a> {
    fn create_textures(device: &Device, config: &SurfaceConfiguration) -> Textures {
        let storage_texture_1 = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[]
        });

        let storage_texture_2 = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[]
        });

        Textures { 
            storage_1: storage_texture_1, 
            storage_2: storage_texture_2
        }

    }

    fn create_buffers(device: &Device, config: &SurfaceConfiguration) -> Buffers {
        let uniforms = Uniforms {
            data_1: [NUMBER_OF_PARTICLES, config.width, config.height, 0]
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
        device: &Device
    ) -> (Buffers, BindGroupLayouts, BindGroups) {

        let textures = Self::create_textures(device, config);
        let buffers = Self::create_buffers(device, config);
        
        let sampler = device.create_sampler(&SamplerDescriptor::default());

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
        let bind_group_layout_render = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { 
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }, 
                        view_dimension: wgpu::TextureViewDimension::D2, 
                        multisampled: false
                    },
                    count: None
                },

                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None
                }
            ]
        });

        let bind_group_layouts = BindGroupLayouts {
            compute: bind_group_layout_compute,
            fade_out_compute: bind_group_layout_fade_out_pipeline,
            render: bind_group_layout_render
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


        let bind_group_render_1 = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layouts.render,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view_1)
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler)
                }
            ]
        });
        let bind_group_render_2 = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layouts.render,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view_2)
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler)
                }
            ]
        });

        let bind_groups = BindGroups {
            compute_1: bind_group_compute_1,
            compute_2: bind_group_compute_2,
            fade_out_1: bind_group_fade_out_1,
            fade_out_2: bind_group_fade_out_2,
            render_1: bind_group_render_1,
            render_2: bind_group_render_2
        };

        return (
            buffers,
            bind_group_layouts,
            bind_groups
        );
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

    fn create_render_pipelines(
        bind_group_layout: &BindGroupLayout, 
        device: &Device,
        config: &wgpu::wgt::SurfaceConfiguration<Vec<wgpu::TextureFormat>>
    ) -> RenderPipelines {

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(bind_group_layout)],
            immediate_size: 0
        });

        let module = device.create_shader_module(wgpu::include_wgsl!("../../shaders/render.wgsl"));
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[]
            },
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::all()
                })]
            }),
            multiview_mask: None,
            cache: None
        });

        RenderPipelines {
            main: pipeline
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

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::None,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface)
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&DeviceDescriptor {
            label: None,
            required_features: Features::empty(),
            required_limits: Limits::defaults(),
            experimental_features: ExperimentalFeatures::disabled(),
            memory_hints: MemoryHints::default(),
            trace: wgpu::Trace::Off
        }).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let ( 
            buffers,
            bind_group_layouts,
            bind_groups
        ) = Self::create_bind_groups(&config, &device);

        let compute_pipelines = Self::create_compute_pipelines(&bind_group_layouts, &device);
        let render_pipelines = Self::create_render_pipelines(&bind_group_layouts.render, &device, &config);

        surface.configure(&device, &config);

        Ok(
            Self {
                surface,
                device,
                queue,
                config,
                render_stage: RenderStage::First,
                buffers,
                bind_groups,
                compute_pipelines,
                render_pipelines,
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

    pub fn render(&mut self) -> anyhow::Result<()> {
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

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

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

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        let render_bind_group = if self.render_stage == RenderStage::First {
            &self.bind_groups.render_2
        } else {
            &self.bind_groups.render_1
        };

        render_pass.set_bind_group(0, Some(render_bind_group), &[]);
        render_pass.set_pipeline(&self.render_pipelines.main);
        render_pass.draw(0..3, 0..1);

        drop(render_pass);

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