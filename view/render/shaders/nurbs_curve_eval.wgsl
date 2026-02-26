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

/// ノットスパン検索（バイナリサーチ）
fn find_knot_span(u: f32, degree: u32, num_cp: u32) -> u32 {
    let n = num_cp - 1u;
    let p = degree;
    
    // 最後のノット値の場合
    if u >= knots[n + p + 1u] {
        return n;
    }
    
    // バイナリサーチ（無限ループ対策付き）
    var low = p;
    var high = n + 1u;
    
    // high <= n を確認して無限ループを防止
    let max_iterations = 32u; // log2(2^32) より十分大きい値
    var iterations = 0u;
    
    while low < high && iterations < max_iterations {
        let mid = (low + high) / 2u;
        
        if u < knots[mid] {
            high = mid;
        } else if u < knots[mid + 1u] || mid == n {
            // 見つかった
            return mid;
        } else {
            low = mid + 1u;
        }
        
        iterations = iterations + 1u;
    }
    
    return low;
}

/// NURBS basis functions (反復型Cox-de Boor算法)
/// ノットスパンに影響する基底関数値（degree個）を計算
/// 戻り値: basis[0..degree] = N_{span-degree+j, degree}(u)
fn basis_functions_iter(span: u32, p: u32, u: f32) -> array<f32, 6> {
    var basis = array<f32, 6>(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    var left = array<f32, 6>(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    var right = array<f32, 6>(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    
    basis[0] = 1.0;
    
    // j = 1..p の反復計算（通常p=3立方B-スプラインの場合）
    for (var j = 1u; j <= p; j = j + 1u) {
        left[j] = u - knots[span + 1u - j];
        right[j] = knots[span + j] - u;
        
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

/// NURBS曲線評価（有理・非有理両対応）
fn evaluate_nurbs(u: f32, degree: u32, num_cp: u32) -> vec3<f32> {
    let span = find_knot_span(u, degree, num_cp);
    
    // 基底関数値を計算（反復型）
    let basis = basis_functions_iter(span, degree, u);
    
    var numerator = vec3<f32>(0.0, 0.0, 0.0);
    var denominator = 0.0;
    
    // スパンに影響する制御点のみ計算（degree+1個）
    for (var j = 0u; j <= degree; j = j + 1u) {
        let idx = span - degree + j;
        let N = basis[j];
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
    // 【確認用】曲線を青色で表示
    return vec4<f32>(0.0, 0.5, 1.0, 1.0);
}
