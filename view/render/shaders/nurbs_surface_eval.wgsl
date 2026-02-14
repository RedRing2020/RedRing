// NURBS曲面 GPU評価シェーダー
// Vertex Shaderで直接NURBS basis functionsを計算し、三角形メッシュで描画
// メモリ最適化版: 頂点バッファで(u,v)パラメータを受け取り

// === @group(0): カメラ行列 ===
struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// === @group(1): NURBSサーフェスデータ（Storage Buffers: 5個に削減） ===
@group(1) @binding(0)
var<storage, read> control_points: array<f32>; // flatten: [x0,y0,z0, x1,y1,z0, ...]（u方向優先）

@group(1) @binding(1)
var<storage, read> weights: array<f32>; // 重み（非有理の場合は全て1.0のダミー）

@group(1) @binding(2)
var<storage, read> u_knots: array<f32>; // u方向ノットベクトル

@group(1) @binding(3)
var<storage, read> v_knots: array<f32>; // v方向ノットベクトル

@group(1) @binding(4)
var<storage, read> metadata: array<u32>; // [u_degree, v_degree, u_count, v_count]

// === 頂点出力 ===
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>, // デバッグ用UV座標
}

// === ヘルパー関数 ===

/// u方向ノットスパン検索
fn find_knot_span_u(u: f32) -> u32 {
    let p = metadata[0]; // u_degree
    let n = metadata[2] - 1u; // u_count - 1
    
    if u >= u_knots[n + p + 1u] {
        return n;
    }
    
    var low = p;
    var high = n + 1u;
    let max_iterations = 32u;
    var iterations = 0u;
    
    while low < high && iterations < max_iterations {
        let mid = (low + high) / 2u;
        
        if u < u_knots[mid] {
            high = mid;
        } else if u < u_knots[mid + 1u] || mid == n {
            return mid;
        } else {
            low = mid + 1u;
        }
        
        iterations = iterations + 1u;
    }
    
    return low;
}

/// v方向ノットスパン検索
fn find_knot_span_v(v: f32) -> u32 {
    let q = metadata[1]; // v_degree
    let m = metadata[3] - 1u; // v_count - 1
    
    if v >= v_knots[m + q + 1u] {
        return m;
    }
    
    var low = q;
    var high = m + 1u;
    let max_iterations = 32u;
    var iterations = 0u;
    
    while low < high && iterations < max_iterations {
        let mid = (low + high) / 2u;
        
        if v < v_knots[mid] {
            high = mid;
        } else if v < v_knots[mid + 1u] || mid == m {
            return mid;
        } else {
            low = mid + 1u;
        }
        
        iterations = iterations + 1u;
    }
    
    return low;
}

/// u方向NURBS basis functions（Cox-de Boor反復式）
fn basis_functions_u(span: u32, u: f32) -> array<f32, 6> {
    let p = metadata[0]; // u_degree
    
    var basis = array<f32, 6>(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    var left = array<f32, 6>(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    var right = array<f32, 6>(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    
    basis[0] = 1.0;
    
    for (var j = 1u; j <= p; j = j + 1u) {
        left[j] = u - u_knots[span + 1u - j];
        right[j] = u_knots[span + j] - u;
        
        var saved = 0.0;
        for (var r = 0u; r < j; r = r + 1u) {
            let temp = basis[r] / (right[r + 1u] + left[j - r]);
            basis[r] = saved + right[r + 1u] * temp;
            saved = left[j - r] * temp;
        }
        basis[j] = saved;
    }
    
    return basis;
}

/// v方向NURBS basis functions（Cox-de Boor反復式）
fn basis_functions_v(span: u32, v: f32) -> array<f32, 6> {
    let q = metadata[1]; // v_degree
    
    var basis = array<f32, 6>(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    var left = array<f32, 6>(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    var right = array<f32, 6>(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    
    basis[0] = 1.0;
    
    for (var j = 1u; j <= q; j = j + 1u) {
        left[j] = v - v_knots[span + 1u - j];
        right[j] = v_knots[span + j] - v;
        
        var saved = 0.0;
        for (var r = 0u; r < j; r = r + 1u) {
            let temp = basis[r] / (right[r + 1u] + left[j - r]);
            basis[r] = saved + right[r + 1u] * temp;
            saved = left[j - r] * temp;
        }
        basis[j] = saved;
    }
    
    return basis;
}

/// NURBS曲面評価（u,v位置）
fn evaluate_nurbs_surface(u: f32, v: f32) -> vec3<f32> {
    let u_span = find_knot_span_u(u);
    let v_span = find_knot_span_v(v);
    
    let u_basis = basis_functions_u(u_span, u);
    let v_basis = basis_functions_v(v_span, v);
    
    let p = metadata[0]; // u_degree
    let q = metadata[1]; // v_degree
    let v_count = metadata[3];
    
    var point = vec3<f32>(0.0, 0.0, 0.0);
    var weight_sum = 0.0;
    
    // テンソル積でbasis functionsを結合
    for (var i = 0u; i <= p; i = i + 1u) {
        for (var j = 0u; j <= q; j = j + 1u) {
            let u_idx = u_span - p + i;
            let v_idx = v_span - q + j;
            
            let cp_idx = (u_idx * v_count + v_idx) * 3u;
            let w_idx = u_idx * v_count + v_idx;
            
            let basis = u_basis[i] * v_basis[j];
            let w = weights[w_idx];
            let weighted_basis = basis * w;
            
            point.x = point.x + weighted_basis * control_points[cp_idx];
            point.y = point.y + weighted_basis * control_points[cp_idx + 1u];
            point.z = point.z + weighted_basis * control_points[cp_idx + 2u];
            
            weight_sum = weight_sum + weighted_basis;
        }
    }
    
    return point / weight_sum; // 有理NURBS正規化
}

/// u方向偏微分（数値微分、法線計算用）
fn evaluate_u_derivative(u: f32, v: f32, delta: f32) -> vec3<f32> {
    let p1 = evaluate_nurbs_surface(u + delta, v);
    let p2 = evaluate_nurbs_surface(u - delta, v);
    return (p1 - p2) / (2.0 * delta);
}

/// v方向偏微分（数値微分、法線計算用）
fn evaluate_v_derivative(u: f32, v: f32, delta: f32) -> vec3<f32> {
    let p1 = evaluate_nurbs_surface(u, v + delta);
    let p2 = evaluate_nurbs_surface(u, v - delta);
    return (p1 - p2) / (2.0 * delta);
}

/// 法線ベクトル計算（偏微分のクロス積）
/// NURBS標準定義: ∂S/∂u × ∂S/∂v が外向き法線（右手系）
fn evaluate_normal(u: f32, v: f32) -> vec3<f32> {
    let delta = 0.001; // 数値微分のステップサイズ
    let du = evaluate_u_derivative(u, v, delta);
    let dv = evaluate_v_derivative(u, v, delta);
    return normalize(cross(du, dv)); // ∂u × ∂v = 外向き法線
}

// === Vertex Shader ===
/// 頂点属性として(u,v)パラメータを受け取り、NURBS曲面を評価
@vertex
fn vs_main(@location(0) uv_param: vec2<f32>) -> VertexOutput {
    let u = uv_param.x;
    let v = uv_param.y;
    
    let position = evaluate_nurbs_surface(u, v);
    let normal = evaluate_normal(u, v);
    
    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * uniforms.model * vec4<f32>(position, 1.0);
    out.world_normal = normalize((uniforms.model * vec4<f32>(normal, 0.0)).xyz);
    
    // UV座標をそのままカラーマッピングに使用
    out.uv = uv_param;
    
    // UV座標ベースのカラーマッピング
    out.color = vec4<f32>(0.3 + 0.7 * u, 0.5, 0.3 + 0.7 * v, 1.0);
    
    return out;
}

// === Fragment Shader ===
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 簡易Lambert拡散反射ライティング
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(in.world_normal, light_dir), 0.0);
    
    // 環境光 + 拡散反射
    let ambient = 0.3;
    let lighting = ambient + (1.0 - ambient) * diffuse;
    
    return in.color * lighting;
}
