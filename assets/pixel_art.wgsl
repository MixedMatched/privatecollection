#import bevy_pbr::utils
#import bevy_core_pipeline::fullscreen_vertex_shader FullscreenVertexOutput

@group(0) @binding(0)
var screen_texture: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;

const SCALE: f32 = 128.0;

fn downsample(in: vec2<f32>) -> vec2<f32> {
    return floor(in * SCALE) / SCALE;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // downsample
    var uv = downsample(in.uv);

    // color quantize
    var color = textureSample(screen_texture, texture_sampler, uv);
    color = floor(color * 128.0) / 128.0;

    return vec4<f32>(color.rgb, 1.0);
}