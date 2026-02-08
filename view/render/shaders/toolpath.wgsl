// CAM工具経路レンダリング用シェーダー
// LineList トポロジーで使用（2頂点で1線分）
// 頂点カラーに対応

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // モデル座標をワールド座標に変換
    let world_pos = uniforms.model * vec4<f32>(input.position, 1.0);

    // ワールド座標をクリップ座標に変換
    out.clip_position = uniforms.view_proj * world_pos;

    // 頂点カラーをフラグメントシェーダーに渡す
    out.color = input.color;

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
