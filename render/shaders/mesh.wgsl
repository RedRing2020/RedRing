// メッシュレンダリング用シェーダー
// 位置と法線を持つ頂点を処理し、ライティングを適用

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
}

struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    light_position: vec3<f32>,
    light_color: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // モデル行列を適用してワールド座標に変換
    let world_position = uniforms.model * vec4<f32>(input.position, 1.0);
    out.world_position = world_position.xyz;
    
    // ビュー・プロジェクション行列を適用
    out.clip_position = uniforms.view_proj * world_position;
    
    // 法線をワールド座標に変換（回転のみ適用）
    out.world_normal = (uniforms.model * vec4<f32>(input.normal, 0.0)).xyz;
    
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // 法線を正規化
    let normal = normalize(input.world_normal);
    
    // ライトベクトルを計算
    let light_dir = normalize(uniforms.light_position - input.world_position);
    
    // ランバートライティング（拡散反射）
    let diffuse = max(dot(normal, light_dir), 0.1); // 最小値0.1でアンビエント効果
    
    // 基本的なマテリアルカラー（グレー）
    let base_color = vec3<f32>(0.7, 0.7, 0.7);
    
    // 最終カラー
    let final_color = base_color * uniforms.light_color * diffuse;
    
    return vec4<f32>(final_color, 1.0);
}