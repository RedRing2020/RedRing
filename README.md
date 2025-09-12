# RedRing

Rust 製 CAD/CAM 研究用プラットフォーム  
3D Viewer / NURBS / プリミティブ形状 / CAM Path 計算 / 切削シミュレーション

---

## 特徴
- NURBS 曲面とプリミティブ形状のサポート
- Rust + wgpu によるリアルタイム可視化
- ボクセル切削シミュレーション
- スタンドアロンで動作

---

## ビルド方法

### 必要環境
- Rust (最新 stable 推奨)
- cargo
- (Windows の場合) Visual Studio Build Tools

### ビルド
```bash
git clone https://github.com/redring2020/RedRing.git
cd RedRing
cargo run
