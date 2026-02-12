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

    // デバッグ: カメラ行列を無視して、ワールド座標を直接クリップ空間にマッピング
    // ワークピース範囲 (0-100, 0-100, 0-50) を (-1,1) の範囲にマッピング
    let x = (input.position.x / 50.0) - 1.0;  // 0-100 → -1 to 1
    let y = (input.position.y / 50.0) - 1.0;  // 0-100 → -1 to 1
    let z = 0.0;  // Z座標は固定（深度なし）
    
    out.clip_position = vec4<f32>(x, y, z, 1.0);

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // 明るい黄色の線（暗い背景でもはっきり見える）
    return vec4<f32>(1.0, 1.0, 0.0, 1.0);
}
