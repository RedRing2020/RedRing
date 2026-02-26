# geo_io 使用例

geo_io の STL 入出力・バルク読み込みの使用例をまとめます。

## 標準読み込み（頂点重複削除）

```rust,no_run
use geo_io::stl;
use std::path::Path;

let mesh = stl::load_stl::<f64>(Path::new("model.stl"))?;
stl::save_stl(&mesh, Path::new("output.stl"))?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## 高速バルク読み込み（GPU転送用）

```rust,no_run
use geo_io::StlTriangleBulk;
use std::path::Path;

let bulk = StlTriangleBulk::<f32>::from_binary_stl_fast(Path::new("model.stl"))?;
let vertex_data: &[f32] = bulk.vertices_slice();
let mesh = bulk.to_triangle_mesh()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## インデックス付きバルク読み込み（頂点重複削減 + GPU最適化）

```rust,no_run
use geo_io::StlIndexedBulk;
use std::path::Path;

let indexed = StlIndexedBulk::<f32>::from_binary_stl_fast(Path::new("model.stl"))?;
println!("メモリ削減率: {:.1}%", indexed.memory_reduction());

let vertices = indexed.vertices_slice();
let indices = indexed.indices_slice();
let mesh = indexed.to_triangle_mesh()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```
