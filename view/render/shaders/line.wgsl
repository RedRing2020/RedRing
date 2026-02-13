// 線分レンダリング用シェーダー
// LineList トポロジーで使用（2頂点で1線分）

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,  // 線分では使用しないが、MeshVertexとの互換性のため保持
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
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

    // カメラ行列を適用して正しい3D変換を実施
    // model 行列でワールド座標に変換 → view_proj 行列でクリップ空間に変換
    let world_position = uniforms.model * vec4<f32>(input.position, 1.0);
    out.clip_position = uniforms.view_proj * world_position;

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // 明るい黄色の線（暗い背景でもはっきり見える）
    return vec4<f32>(1.0, 1.0, 0.0, 1.0);
}
