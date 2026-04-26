use wgpu::{BindGroup, BindGroupLayout, Buffer, ComputePipeline, Texture};

/// Decides which set of bind groups needs to be set for which pipeline.
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

    pub fade_out_1: BindGroup,
    pub fade_out_2: BindGroup
}

pub struct BindGroupLayouts {
    pub compute: BindGroupLayout,
    pub fade_out_compute: BindGroupLayout
}

pub struct ComputePipelines {
    pub main: ComputePipeline,
    pub fade_out: ComputePipeline
}