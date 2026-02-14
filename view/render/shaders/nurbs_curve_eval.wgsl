// NURBS曲線 GPU評価シェーダー
// Vertex Shaderで直接NURBS basis functionsを計算し、LineStripで描画

// === @group(0): カメラ行列 ===
struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// === @group(1): NURBSデータ ===
@group(1) @binding(0)
var<storage, read> params: array<f32>; // 評価パラメータ列

@group(1) @binding(1)
var<storage, read> control_points: array<f32>; // flatten: [x0,y0,z0, x1,y1,z1, ...]

@group(1) @binding(2)
var<storage, read> weights: array<f32>; // 重み（非有理の場合は全て1.0のダミー）

@group(1) @binding(3)
var<storage, read> knots: array<f32>; // ノットベクトル

@group(1) @binding(4)
var<storage, read> degree_storage: array<u32>; // [degree] (1要素配列)

// === ヘルパー関数 ===

/// ノットスパン検索（Cox-de Boor再帰用）
fn find_knot_span(u: f32, degree: u32, num_cp: u32) -> u32 {
    let n = num_cp - 1u;
    let p = degree;
    
    // 最後のノット値の場合
    if u >= knots[n + p + 1u] {
        return n;
    }
    
    // バイナリサーチ
    var low = p;
    var high = n + 1u;
    var mid = (low + high) / 2u;
    
    while u < knots[mid] || u >= knots[mid + 1u] {
        if u < knots[mid] {
            high = mid;
        } else {
            low = mid;
        }
        mid = (low + high) / 2u;
    }
    
    return mid;
}

/// NURBS basis function (Cox-de Boor再帰)
/// 注: WGSLでは再帰深度に制限がある可能性があるが、通常のdegree（3-5程度）なら許容範囲
fn basis_function(i: u32, p: u32, u: f32) -> f32 {
    // p=0の場合（ベース）
    if p == 0u {
        if u >= knots[i] && u < knots[i + 1u] {
            return 1.0;
        } else {
            return 0.0;
        }
    }
    
    // 再帰的計算
    let left_num = u - knots[i];
    let left_den = knots[i + p] - knots[i];
    var left = 0.0;
    if abs(left_den) > 1e-10 {
        left = left_num / left_den * basis_function(i, p - 1u, u);
    }
    
    let right_num = knots[i + p + 1u] - u;
    let right_den = knots[i + p + 1u] - knots[i + 1u];
    var right = 0.0;
    if abs(right_den) > 1e-10 {
        right = right_num / right_den * basis_function(i + 1u, p - 1u, u);
    }
    
    return left + right;
}

/// NURBS曲線評価（有理・非有理両対応）
fn evaluate_nurbs(u: f32, degree: u32, num_cp: u32) -> vec3<f32> {
    let span = find_knot_span(u, degree, num_cp);
    
    var numerator = vec3<f32>(0.0, 0.0, 0.0);
    var denominator = 0.0;
    
    // スパンに影響する制御点のみ計算（degree+1個）
    for (var j = 0u; j <= degree; j = j + 1u) {
        let idx = span - degree + j;
        let N = basis_function(idx, degree, u);
        let w = weights[idx];
        
        let cp_base = idx * 3u;
        let cp = vec3<f32>(
            control_points[cp_base],
            control_points[cp_base + 1u],
            control_points[cp_base + 2u]
        );
        
        numerator = numerator + N * w * cp;
        denominator = denominator + N * w;
    }
    
    if abs(denominator) > 1e-10 {
        return numerator / denominator;
    } else {
        return numerator; // 非有理の場合（全てw=1.0）
    }
}

// === Vertex Shader ===
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    let degree = degree_storage[0];
    let num_cp = arrayLength(&control_points) / 3u;
    
    // パラメータ取得
    let u = params[vertex_index];
    
    // NURBS評価
    let position = evaluate_nurbs(u, degree, num_cp);
    
    // ワールド座標 -> クリップ座標
    let world_position = uniforms.model * vec4<f32>(position, 1.0);
    out.clip_position = uniforms.view_proj * world_position;
    
    return out;
}

// === Fragment Shader ===
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // NURBS曲線は青色で表示
    return vec4<f32>(0.0, 0.5, 1.0, 1.0);
}
