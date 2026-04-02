use std::sync::Arc;

use wgpu::{BackendOptions, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ColorTargetState, ColorWrites, ComputePipeline, ComputePipelineDescriptor, Device, DeviceDescriptor, ExperimentalFeatures, Extent3d, Features, FragmentState, Instance, InstanceDescriptor, InstanceFlags, Limits, MemoryBudgetThresholds, MemoryHints, MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, SamplerDescriptor, ShaderStages, Surface, SurfaceConfiguration, TextureDescriptor, TextureUsages, TextureViewDescriptor, VertexState};
use winit::window::Window;

#[derive(PartialEq, Eq)]
enum RenderStage {
    First,
    Second
}

pub struct Renderer<'a> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    render_stage: RenderStage,
    bind_group_compute_1: BindGroup,
    bind_group_compute_2: BindGroup,
    bind_group_render_1: BindGroup,
    bind_group_render_2: BindGroup,
    compute_pipeline: ComputePipeline,
    render_pipeline: RenderPipeline,
    window: Arc<Window>
}

impl<'a> Renderer<'a> {
    fn create_bind_group(
        config: &wgpu::wgt::SurfaceConfiguration<Vec<wgpu::TextureFormat>>,
        device: &Device
    ) -> (BindGroupLayout, BindGroupLayout, BindGroup, BindGroup, BindGroup, BindGroup) {

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

        let sampler = device.create_sampler(&SamplerDescriptor::default());

        let view_1 = storage_texture_1.create_view(&TextureViewDescriptor::default());
        let view_2 = storage_texture_2.create_view(&TextureViewDescriptor::default());

        let bind_group_layout_compute = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture { 
                    access: wgpu::StorageTextureAccess::WriteOnly, 
                    format: wgpu::TextureFormat::Rgba8Unorm, 
                    view_dimension: wgpu::TextureViewDimension::D2
                },
                count: None
            }]
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

        let bind_group_compute_1 = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout_compute,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view_1)
                }
            ]
        });
        let bind_group_compute_2 = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout_compute,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view_2)
                }
            ]
        });

        let bind_group_render_1 = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout_render,
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
            layout: &bind_group_layout_render,
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

        return (
            bind_group_layout_compute, 
            bind_group_layout_render, 
            bind_group_compute_1, 
            bind_group_compute_2,
            bind_group_render_1,
            bind_group_render_2
        );
    }

    fn create_compute_pipeline(bind_group_layout: &BindGroupLayout, device: &Device) -> ComputePipeline {
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(bind_group_layout)],
            immediate_size: 0
        });

        let module = device.create_shader_module(wgpu::include_wgsl!("../shaders/compute.wgsl"));

        return device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None
        });
    }

    fn create_render_pipeline(
        bind_group_layout: &BindGroupLayout, 
        device: &Device,
        config: &wgpu::wgt::SurfaceConfiguration<Vec<wgpu::TextureFormat>>
    ) -> RenderPipeline {

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(bind_group_layout)],
            immediate_size: 0
        });

        let module = device.create_shader_module(wgpu::include_wgsl!("../shaders/render.wgsl"));

        return device.create_render_pipeline(&RenderPipelineDescriptor {
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
                    blend: None,
                    write_mask: ColorWrites::all()
                })]
            }),
            multiview_mask: None,
            cache: None
        });
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
            bind_group_layout_compute, 
            bind_group_layout_render, 
            bind_group_compute_1,
            bind_group_compute_2,
            bind_group_render_1,
            bind_group_render_2

        ) = Self::create_bind_group(&config, &device);

        let compute_pipeline = Self::create_compute_pipeline(&bind_group_layout_compute, &device);
        let render_pipeline = Self::create_render_pipeline(&bind_group_layout_render, &device, &config);

        surface.configure(&device, &config);

        Ok(
            Self {
                surface,
                device,
                queue,
                config,
                render_stage: RenderStage::First,
                bind_group_compute_1,
                bind_group_compute_2,
                bind_group_render_1,
                bind_group_render_2,
                compute_pipeline,
                render_pipeline,
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

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,

            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                self.surface.configure(&self.device, &self.config);
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

        let compute_bind_group = if self.render_stage == RenderStage::First {
            &self.bind_group_compute_1
        } else {
            &self.bind_group_compute_2
        };

        compute_pass.set_bind_group(0, Some(compute_bind_group), &[]);
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.dispatch_workgroups((self.config.width + 7) / 8, (self.config.height + 7) / 8, 1);

        drop(compute_pass);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        let render_bind_group = if self.render_stage == RenderStage::First {
            &self.bind_group_render_2
        } else {
            &self.bind_group_render_1
        };

        render_pass.set_bind_group(0, Some(render_bind_group), &[]);
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw(0..3, 0..1);

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        match self.render_stage {
            RenderStage::First => {self.render_stage = RenderStage::Second},
            RenderStage::Second => {self.render_stage = RenderStage::First}
        }

        Ok(())

    }
}