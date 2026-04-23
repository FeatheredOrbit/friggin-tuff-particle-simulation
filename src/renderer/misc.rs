use wgpu::{BindGroup, BindGroupLayout, Buffer, ComputePipeline, RenderPipeline, Texture};

/// Decides which set of bindgroups needs to be set for which pipeline.
#[derive(PartialEq, Eq)]
pub enum RenderStage {
    First,
    Second
}

pub struct Textures {
    pub storage_1: Texture,
    pub storage_2: Texture
}

pub struct Buffers {
    pub uniform: Buffer,
    pub storage: Buffer
}

pub struct BindGroups {
    pub compute_1: BindGroup,
    pub compute_2: BindGroup,
    pub render_1: BindGroup,
    pub render_2: BindGroup,
}

pub struct BindGroupLayouts {
    pub compute: BindGroupLayout,
    pub render: BindGroupLayout,
}

pub struct RenderPipelines {
    pub main: RenderPipeline
}

pub struct ComputePipelines {
    pub main: ComputePipeline
}