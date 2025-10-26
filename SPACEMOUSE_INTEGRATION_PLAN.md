# SpaceMouse Integration Plan for RedRing

CAD/CAM研究プラットフォーム RedRing への 3DConnexion SpaceMouse 対応実装計画

**作成日**: 2025年10月26日  
**対象**: RedRing v0.1.0  
**現在のブランチ**: feature/geometry-foundation-final

## 🎯 概要

3DConnexion SpaceMouse は6軸（6DOF）入力デバイスで、CADアプリケーションにおいて3D空間のナビゲーションを大幅に向上させる。RedRing の現在のマウス+キーボード操作に加えて、プロフェッショナルなCAD操作環境を提供する。

## 🔍 技術調査結果

### 実装アプローチ

**推奨方法: HIDAPI使用**
- ✅ クロスプラットフォーム対応（Windows/Linux/macOS）
- ✅ 3DxWare SDKより軽量・シンプル
- ✅ Rustクレート `hidapi` が利用可能
- ✅ ドライバー依存が少ない

**参考情報:**
- StackOverflow: GLFW C++での実装例でHIDAPIが推奨されている
- Python実装: PySpaceMouseでHIDAPI使用実績あり
- OpenSCADでも HidApiInputDriver が動作確認済み

### 必要なRustクレート

```toml
[dependencies]
hidapi = "2.6"          # HID デバイス通信
serde = "1.0"           # データシリアライゼーション（設定用）
```

## 📊 SpaceMouse仕様

### 入力データ形式

- **6軸データ**: 
  - X, Y, Z（移動: 前後/左右/上下）
  - RX, RY, RZ（回転: ピッチ/ヨー/ロール）
- **拡張データ（SpaceMouse Compact）**:
  - **移動ベクトル**: 平行移動情報
  - **回転ベクトル**: 回転軸・角度情報  
  - **スケールベクトル**: 均等/非均等スケーリング（可能性あり）
  - **射影スケール**: 通常は含まれない（予想）
- **ボタン**: デバイス依存（0-32個）
- **データレート**: 最大1000Hz
- **値域**: -1023 〜 +1023（16bit精度）
- **ベンダーID**: 0x046D（3Dconnexion）

### 対応デバイス

- SpaceMouse Enterprise
- SpaceMouse Pro Wireless
- SpaceMouse Pro
- SpaceMouse Wireless
- **SpaceMouse Compact** ⭐（所有デバイス）
- Space Navigator（旧モデル）

**SpaceMouse Compact 特性:**
- 6軸入力: 回転ベクトル + 移動ベクトル
- **ハードウェア制約**: 6軸操作で非等方スケール指示は物理的に不可能
- **等方スケール**: プッシュ/プル操作による統一スケール変更のみ
- 2ボタン構成
- USB-C接続
- 座標変換マトリックス処理に最適化
- Matrix×Quaternion 2回演算での効率的変換

## 🏗️ 実装計画

### Phase 1: 基盤準備（SpaceMouse Compact対応）

#### 1.1 SpaceMouseInputモジュール作成

**ファイル**: `view/app/src/spacemouse_input.rs`

```rust
use hidapi::{HidApi, HidDevice};

pub struct SpaceMouseInput {
    device: Option<HidDevice>,
    // 6軸データ（SpaceMouse Compact最適化）
    pub translation: [f32; 3],  // X, Y, Z移動ベクトル
    pub rotation: [f32; 3],     // RX, RY, RZ回転ベクトル
    pub scale: [f32; 3],        // SX, SY, SZ スケールベクトル（可能性あり）
    pub buttons: u32,           // ボタン状態（2ボタン）
    // 設定
    pub sensitivity: f32,
    pub deadzone: f32,
    pub enabled: bool,
    // マトリックス統合用
    pub coordinate_system: SpaceMouseCoordinateSystem,
    // 変換処理フラグ
    pub matrix_operations: MatrixOperationMode,
}

#[derive(Debug, Clone, Copy)]
pub enum SpaceMouseOperation {
    MatrixTransform,   // 座標変換マトリックス統合（推奨）
    DirectNavigation,  // 従来の直接ナビゲーション
    Disabled,          // 無効
}

/// Matrix×Quaternion演算モード
#[derive(Debug, Clone, Copy)]
pub enum MatrixOperationMode {
    /// 2回演算: Translation→Rotation→Scale
    TwoStageTransform,
    /// 単一演算: 統合マトリックス
    SingleTransform,
    /// 分離演算: 各軸独立処理
    SeparateAxisTransform,
}

    impl SpaceMouseInput {
    pub fn new() -> Self {
        Self {
            device: None,
            translation: [0.0; 3],
            rotation: [0.0; 3],
            scale: [1.0; 3],        // スケールはデフォルト1.0
            buttons: 0,
            sensitivity: 1.0,
            deadzone: 0.05,
            enabled: false,
            coordinate_system: SpaceMouseCoordinateSystem::default(),
            matrix_operations: MatrixOperationMode::TwoStageTransform,
        }
    }    /// SpaceMouse Compact専用の接続処理
    pub fn connect_compact(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        let api = HidApi::new()?;
        
        // SpaceMouse Compact固有のProduct ID検索
        for device_info in api.device_list() {
            if device_info.vendor_id() == 0x046D && 
               device_info.product_id() == 0xC652 { // SpaceMouse Compact
                let device = device_info.open_device(&api)?;
                self.device = Some(device);
                self.enabled = true;
                return Ok(true);
            }
        }
        
        Err("SpaceMouse Compact not found".into())
    }

    /// マトリックス統合用データ更新
    pub fn update_matrix_mode(&mut self) -> bool {
        if let Some(device) = &mut self.device {
            let mut buffer = [0u8; 8];
            
            match device.read_timeout(&mut buffer, 100) {
                Ok(bytes_read) if bytes_read > 0 => {
                    // SpaceMouse Compactのデータフォーマット解析
                    self.parse_compact_data(&buffer);
                    self.apply_deadzone();
                    self.update_coordinate_system();
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }
    
    /// SpaceMouse Compactデータ解析
    fn parse_compact_data(&mut self, buffer: &[u8]) {
        // SpaceMouse Compactの具体的なデータフォーマットに基づく解析
        // 移動ベクトル（X, Y, Z）
        self.translation[0] = parse_axis_data(&buffer[0..2]);
        self.translation[1] = parse_axis_data(&buffer[2..4]);
        self.translation[2] = parse_axis_data(&buffer[4..6]);
        
        // 回転ベクトル（RX, RY, RZ）- 別パケットまたは追加データで取得
        if buffer.len() >= 12 {
            self.rotation[0] = parse_axis_data(&buffer[6..8]);
            self.rotation[1] = parse_axis_data(&buffer[8..10]);
            self.rotation[2] = parse_axis_data(&buffer[10..12]);
        }
        
        // スケールベクトル（SX, SY, SZ）- 拡張データが利用可能な場合
        if buffer.len() >= 18 {
            self.scale[0] = parse_scale_data(&buffer[12..14]);
            self.scale[1] = parse_scale_data(&buffer[14..16]);
            self.scale[2] = parse_scale_data(&buffer[16..18]);
        }
        
        // 実装詳細はデバイステストで確定
    }
    
    fn update_coordinate_system(&mut self) {
        self.coordinate_system = SpaceMouseCoordinateSystem::from(self);
    }
}

/// 軸データの解析（16bit符号付き整数→正規化float）
fn parse_axis_data(bytes: &[u8]) -> f32 {
    let raw_value = i16::from_le_bytes([bytes[0], bytes[1]]);
    (raw_value as f32) / 1023.0 // -1.0 〜 +1.0 に正規化
}

/// スケールデータの解析（16bit符号付き整数→スケール値）
fn parse_scale_data(bytes: &[u8]) -> f32 {
    let raw_value = i16::from_le_bytes([bytes[0], bytes[1]]);
    1.0 + ((raw_value as f32) / 1023.0) * 0.1 // 0.9 〜 1.1 のスケール範囲
}
```

#### 1.2 デバイス検出機能

```rust
const SPACEMOUSE_VENDOR_ID: u16 = 0x046D;  // 3Dconnexion

#[derive(Debug, Clone)]
pub struct SpaceMouseDevice {
    pub product_id: u16,
    pub serial_number: Option<String>,
    pub product_string: Option<String>,
}

pub fn detect_spacemouse_devices() -> Result<Vec<SpaceMouseDevice>, Box<dyn std::error::Error>> {
    let api = HidApi::new()?;
    let mut devices = Vec::new();
    
    for device_info in api.device_list() {
        if device_info.vendor_id() == SPACEMOUSE_VENDOR_ID {
            devices.push(SpaceMouseDevice {
                product_id: device_info.product_id(),
                serial_number: device_info.serial_number().map(|s| s.to_string()),
                product_string: device_info.product_string().map(|s| s.to_string()),
            });
        }
    }
    
    Ok(devices)
}
```

### Phase 2: カメラシステム統合

#### 2.1 Camera拡張（座標変換マトリックス統合）

**ファイル**: `viewmodel/graphics/src/camera.rs`

```rust
impl Camera {
    /// SpaceMouseからの6軸入力でカメラ操作（マトリックス統合版）
    pub fn update_from_spacemouse(&mut self, input: &SpaceMouseInput) {
        if !input.enabled {
            return;
        }

        // SpaceMouseからの生の6軸データ
        let translation_vec = Vec3f::new(
            input.translation[0] * input.sensitivity,
            input.translation[1] * input.sensitivity,
            input.translation[2] * input.sensitivity,
        );
        
        let rotation_vec = Vec3f::new(
            input.rotation[0] * input.sensitivity,
            input.rotation[1] * input.sensitivity,
            input.rotation[2] * input.sensitivity,
        );
        
        let scale_vec = Vec3f::new(
            input.scale[0],
            input.scale[1], 
            input.scale[2],
        );
        
        // Matrix×Quaternion 2回演算による統一処理
        match input.matrix_operations {
            MatrixOperationMode::TwoStageTransform => {
                self.apply_two_stage_transform(translation_vec, rotation_vec, scale_vec);
            }
            MatrixOperationMode::SingleTransform => {
                self.apply_single_transform(translation_vec, rotation_vec, scale_vec);
            }
            MatrixOperationMode::SeparateAxisTransform => {
                self.apply_separate_axis_transform(translation_vec, rotation_vec, scale_vec);
            }
        }
    }

    /// 2段階変換: Matrix×Quaternion演算を2回実行
    fn apply_two_stage_transform(&mut self, translation: Vec3f, rotation: Vec3f, scale: Vec3f) {
        // 第1段階: 現在のカメラ変換マトリックス取得
        let view_matrix = self.view_matrix();
        let camera_transform = view_matrix.inverse().unwrap_or(Matrix4f::identity());
        
        // 第2段階: SpaceMouse座標系からカメラ座標系への変換
        let local_translation = camera_transform.transform_vector(translation);
        let local_rotation = camera_transform.transform_vector(rotation);
        
        // 平行移動の適用
        self.target += local_translation * 0.01;
        
        // 回転の適用（Quaternion演算）
        self.apply_rotation_vector(local_rotation * 0.001);
        
        // スケール操作（ズーム距離調整）
        let avg_scale = (scale.x() + scale.y() + scale.z()) / 3.0;
        if (avg_scale - 1.0).abs() > 0.001 { // デッドゾーン
            self.distance *= avg_scale;
            self.distance = self.distance.clamp(0.1, 200.0);
        }
    }
    
    /// 単一変換: 統合マトリックス処理
    fn apply_single_transform(&mut self, translation: Vec3f, rotation: Vec3f, scale: Vec3f) {
        // 統合変換マトリックス構築
        let transform_matrix = self.build_spacemouse_transform_matrix(translation, rotation, scale);
        
        // 現在のカメラ状態に統合マトリックス適用
        self.apply_transform_matrix(transform_matrix);
    }
    
    /// 分離軸変換: 各軸独立処理
    fn apply_separate_axis_transform(&mut self, translation: Vec3f, rotation: Vec3f, scale: Vec3f) {
        // X, Y, Z軸を独立して処理（軸拘束モード等で有用）
        self.apply_axis_constrained_transform(translation, rotation, scale);
    }
}
    
    /// 回転ベクトルからクォータニオン回転への変換
    fn apply_rotation_vector(&mut self, rotation_vec: Vec3f) {
        // 回転ベクトルの大きさ（角度）
        let angle = rotation_vec.magnitude();
        
        if angle > 0.0001 { // デッドゾーン
            // 回転軸の正規化
            let axis = rotation_vec / angle;
            
            // クォータニオン回転の生成と適用
            let rotation_quat = Quaternionf::from_axis_angle(&axis, angle);
            self.rotation = (self.rotation * rotation_quat).normalize()
                .unwrap_or(Quaternionf::identity());
        }
    }
}
```

#### 2.2 マトリックス統合設計

```rust
/// SpaceMouse座標系定義
#[derive(Debug, Clone, Copy)]
pub struct SpaceMouseCoordinateSystem {
    /// X軸: 左右移動（-: 左, +: 右）
    pub x_translation: f32,
    /// Y軸: 前後移動（-: 前, +: 後）
    pub y_translation: f32,
    /// Z軸: 上下移動（-: 下, +: 上）
    pub z_translation: f32,
    /// RX軸: ピッチ回転（-: 下向き, +: 上向き）
    pub rx_rotation: f32,
    /// RY軸: ヨー回転（-: 左回り, +: 右回り）
    pub ry_rotation: f32,
    /// RZ軸: ロール回転（-: 左傾き, +: 右傾き）
    pub rz_rotation: f32,
    /// SX軸: X軸スケール
    pub sx_scale: f32,
    /// SY軸: Y軸スケール  
    pub sy_scale: f32,
    /// SZ軸: Z軸スケール
    pub sz_scale: f32,
}

impl From<&SpaceMouseInput> for SpaceMouseCoordinateSystem {
    fn from(input: &SpaceMouseInput) -> Self {
        Self {
            x_translation: input.translation[0],
            y_translation: input.translation[1],
            z_translation: input.translation[2],
            rx_rotation: input.rotation[0],
            ry_rotation: input.rotation[1],
            rz_rotation: input.rotation[2],
            sx_scale: input.scale[0],
            sy_scale: input.scale[1],
            sz_scale: input.scale[2],
        }
    }
}

impl Default for SpaceMouseCoordinateSystem {
    fn default() -> Self {
        Self {
            x_translation: 0.0,
            y_translation: 0.0,
            z_translation: 0.0,
            rx_rotation: 0.0,
            ry_rotation: 0.0,
            rz_rotation: 0.0,
            sx_scale: 1.0,
            sy_scale: 1.0,
            sz_scale: 1.0,
        }
    }
}
```

#### 2.2 操作モード

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpaceMouseMode {
    /// オブジェクト中心回転（CAD標準）
    ObjectCentric,
    /// カメラ飛行モード
    FlyThrough,
    /// ターゲット追従モード
    TargetLock,
}
```

### Phase 3: アプリケーション統合

#### 3.1 AppState拡張

**ファイル**: `view/app/src/app_state.rs`

```rust
use crate::spacemouse_input::SpaceMouseInput;

pub struct AppState {
    // 既存フィールド
    pub window: Arc<Window>,
    pub graphic: Graphic,
    pub renderer: AppRenderer,
    pub camera: Camera,
    pub mouse_input: MouseInput,
    
    // 新規追加
    pub spacemouse_input: SpaceMouseInput,
}

impl AppState {
    pub fn new(window: Arc<Window>) -> Self {
        // 既存の初期化...
        let mut spacemouse_input = SpaceMouseInput::new();
        
        // SpaceMouse接続試行
        if let Err(e) = spacemouse_input.connect() {
            tracing::warn!("SpaceMouse接続失敗: {}", e);
        } else {
            tracing::info!("SpaceMouse接続成功");
        }

        Self {
            // 既存フィールド...
            spacemouse_input,
        }
    }

    /// メインループでの更新処理
    pub fn update(&mut self) {
        // SpaceMouseデータ更新
        if self.spacemouse_input.update() {
            self.camera.update_from_spacemouse(&self.spacemouse_input);
            self.update_camera_uniforms();
        }
    }
}
```

#### 3.2 キーボードショートカット追加

既存のキーボード処理に追加:

```rust
// view/app/src/app_state.rs の handle_keyboard_input に追加
"s" => {
    // SpaceMouse有効/無効切替
    self.spacemouse_input.enabled = !self.spacemouse_input.enabled;
    let status = if self.spacemouse_input.enabled { "有効" } else { "無効" };
    tracing::info!("SpaceMouse: {}", status);
}
"=" | "+" => {
    // 感度アップ
    self.spacemouse_input.sensitivity = (self.spacemouse_input.sensitivity * 1.2).min(5.0);
    tracing::info!("SpaceMouse感度: {:.2}", self.spacemouse_input.sensitivity);
}
"-" => {
    // 感度ダウン
    self.spacemouse_input.sensitivity = (self.spacemouse_input.sensitivity * 0.8).max(0.1);
    tracing::info!("SpaceMouse感度: {:.2}", self.spacemouse_input.sensitivity);
}
```

## 🎛️ CAD操作への最適化（SpaceMouse Compact特化）

### 座標変換マトリックス統合の利点

1. **直感的操作**: SpaceMouseの移動ベクトル/回転ベクトル/スケールベクトルを直接マトリックス変換
2. **一貫性**: 現在のカメラシステムのマトリックス処理と統一
3. **拡張性**: 将来のオブジェクト操作でも同一の変換ロジック使用可能
4. **精度**: 中間変換を減らして数値誤差を最小化
5. **効率性**: Matrix×Quaternion 2回演算での最適化

### Matrix×Quaternion 2回演算の技術的詳細

```rust
/// 第1回演算: SpaceMouse→カメラ座標系変換
let camera_matrix = view_matrix.inverse();
let local_translation = camera_matrix.transform_vector(spacemouse_translation);
let local_rotation = camera_matrix.transform_vector(spacemouse_rotation);

/// 第2回演算: カメラ状態更新
let rotation_quaternion = Quaternionf::from_axis_angle(&rotation_axis, rotation_angle);
camera.rotation = camera.rotation * rotation_quaternion;
camera.target += local_translation;
```

### SpaceMouse Compact最適化パターン

```rust
/// SpaceMouse Compact用のCAD操作モード
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompactCADMode {
    /// ビューナビゲーション（標準）
    ViewNavigation {
        /// 移動ベクトル→カメラターゲット移動
        translation_scale: f32,
        /// 回転ベクトル→カメラ回転
        rotation_scale: f32,
    },
    /// オブジェクト操作（将来拡張）
    ObjectManipulation {
        /// 選択オブジェクトの変換マトリックス操作
        transform_mode: ObjectTransformMode,
    },
    /// 精密位置決め
    PrecisionPositioning {
        /// 微細操作用の低感度設定
        precision_factor: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectTransformMode {
    LocalSpace,    // ローカル座標系
    WorldSpace,    // ワールド座標系
    ViewSpace,     // ビュー座標系
}
```

### SpaceMouse Compact 2ボタン活用

```rust
/// SpaceMouse Compactのボタン割り当て
pub struct CompactButtonMapping {
    /// ボタン1: モード切替
    pub button1_action: CompactButton1Action,
    /// ボタン2: 機能切替
    pub button2_action: CompactButton2Action,
}

#[derive(Debug, Clone, Copy)]
pub enum CompactButton1Action {
    TogglePrecision,     // 精密モード切替
    ResetView,           // ビューリセット
    ToggleMode,          // 操作モード切替
}

#[derive(Debug, Clone, Copy)]
pub enum CompactButton2Action {
    ToggleConstraint,    // 軸拘束切替（X/Y/Z単独操作）
    FitToView,           // ビューフィット
    ToggleProjection,    // 投影モード切替
}
```

### 感度設定（SpaceMouse Compact最適化）

```rust
pub struct CompactSpaceMouseSettings {
    /// 移動ベクトル感度（カメラターゲット移動）
    pub translation_sensitivity: f32,  
    /// 回転ベクトル感度（カメラ回転）
    pub rotation_sensitivity: f32,     
    /// ズーム操作感度（Z軸移動）
    pub zoom_sensitivity: f32,         
    /// デッドゾーン（微細な手の震え無視）
    pub deadzone: f32,                 
    /// 座標変換マトリックス統合モード
    pub matrix_integration: bool,
    /// 軸反転設定（SpaceMouse Compact用）
    pub invert_axes: CompactAxisInversion,
    /// 2ボタン機能設定
    pub button_mapping: CompactButtonMapping,
}

#[derive(Debug, Clone)]
pub struct CompactAxisInversion {
    pub invert_x_translation: bool,
    pub invert_y_translation: bool,
    pub invert_z_translation: bool,
    pub invert_x_rotation: bool,
    pub invert_y_rotation: bool,
    pub invert_z_rotation: bool,
}

impl Default for CompactSpaceMouseSettings {
    fn default() -> Self {
        Self {
            translation_sensitivity: 1.0,
            rotation_sensitivity: 1.0,
            zoom_sensitivity: 1.0,
            deadzone: 0.05,
            matrix_integration: true,  // 座標変換マトリックス統合を標準に
            invert_axes: CompactAxisInversion::default(),
            button_mapping: CompactButtonMapping::default(),
        }
    }
}
```

## 🔧 実装上の考慮事項

### パフォーマンス最適化

- **バックグラウンド処理**: HIDデータ読み取りを別スレッドで実行
- **フレームレート制御**: SpaceMouseデータ更新を60-120Hzに制限
- **差分更新**: 変化がない場合はカメラ更新をスキップ

### エラーハンドリング

```rust
#[derive(Debug, thiserror::Error)]
pub enum SpaceMouseError {
    #[error("デバイス未検出")]
    DeviceNotFound,
    #[error("接続失敗: {0}")]
    ConnectionFailed(String),
    #[error("データ読み取りエラー: {0}")]
    ReadError(String),
    #[error("デバイス切断")]
    DeviceDisconnected,
}
```

### ユーザビリティ

- **デッドゾーン**: 微細な手の震えを無視（デフォルト5%）
- **感度カーブ**: リニア/指数関数的選択可能
- **軸反転**: ユーザー設定で各軸の方向反転
- **ホットプラグ**: デバイス接続/切断の動的検出

## 📁 ファイル構成

```
view/app/src/
├── app_state.rs           # SpaceMouse統合
├── spacemouse_input.rs    # 新規: SpaceMouse処理
├── spacemouse_error.rs    # 新規: エラー定義
└── lib.rs                 # モジュール追加

viewmodel/graphics/src/
└── camera.rs              # SpaceMouse対応拡張

view/app/Cargo.toml        # hidapi依存関係追加
```

## 🚀 実装スケジュール

### Phase 1: 基本実装（1-2週間）
- [ ] hidapi依存関係追加
- [ ] **SpaceMouse接続・データフォーマット検証**
- [ ] SpaceMouseInputモジュール作成
- [ ] デバイス検出機能
- [ ] 基本的なHIDデータ読み取り

### Phase 1.5: データフォーマット検証（実機テスト）
- [ ] SpaceMouse Compact接続確認
- [ ] 生バイトデータの取得とログ出力
- [ ] データ構造の解析（配列/バイト/テキスト形式）
- [ ] 6軸データの実際のマッピング確認
- [ ] スケールベクトル存在確認

### Phase 2: カメラ統合（1週間）
- [ ] Camera拡張メソッド追加
- [ ] 6軸データからカメラ操作への変換
- [ ] 感度・デッドゾーン設定

### Phase 3: アプリケーション統合（1週間）
- [ ] AppState拡張
- [ ] キーボードショートカット追加
- [ ] エラーハンドリング実装

### Phase 4: 最適化・テスト（1週間）
- [ ] パフォーマンス最適化
- [ ] ユーザビリティ改善
- [ ] 複数デバイス対応
- [ ] 設定保存機能

## 🔗 参考資料

- [3Dconnexion Developer Area](https://3dconnexion.com/us/software-developer/)
- [HIDAPI Rust Crate](https://docs.rs/hidapi/)
- [PySpaceMouse Implementation](https://spacemouse.kubaandrysek.cz/)
- [GLFW SpaceMouse Example (C++)](https://stackoverflow.com/questions/75644410/)

## 💡 将来の拡張（SpaceMouse Compact活用）

- **CADオブジェクト操作**: 選択オブジェクトの6軸マトリックス変換
- **精密加工シミュレーション**: 工具パスの6軸調整
- **マルチビューポート**: 複数視点の同期操作
- **コラボレーション**: ネットワーク経由でのSpaceMouse操作共有
- **VR/AR統合**: 物理SpaceMouseと仮想6軸操作の統合
- **CAMパス編集**: 工具経路の6軸リアルタイム調整
- **マクロ記録**: SpaceMouse操作の記録・再生

### SpaceMouse Compact実装の技術的優位性

1. **座標変換統一**: マトリックス×ベクトル演算でカメラとオブジェクト操作を統一
2. **リアルタイム性**: 移動/回転/スケールベクトルの直接的なマトリックス適用
3. **2回演算効率**: Matrix×Quaternion処理の最適化によるパフォーマンス向上
4. **拡張性**: 同一の変換ロジックでCAD/CAM/レンダリング統合
5. **保守性**: 既存のカメラシステムとの自然な統合
6. **スケール対応**: 非均等スケーリングによる詳細なビュー制御

### 実装検証項目

- [ ] **SpaceMouse Compact データフォーマット解析**
  - [ ] 接続確認（Vendor ID: 0x046D, Product ID: 0xC652）
  - [ ] 生バイトデータの構造確認
  - [ ] パケットサイズの特定
    - [ ] 12バイト（6軸: 移動+回転）
    - [ ] 14バイト（7軸: 移動+回転+等方スケール） ⭐ 最有力候補
    - [ ] 18バイト（9軸: 移動+回転+異方スケール）
  - [ ] 移動ベクトル（X,Y,Z）のバイト位置確認
  - [ ] 回転ベクトル（RX,RY,RZ）のバイト位置確認
  - [ ] スケールデータの形式確認
    - [ ] **等方スケール（f32×1個）の可能性** ⭐ 最有力候補
    - [ ] プッシュ/プル操作による統一スケール変更
    - [ ] 異方スケール（SX,SY,SZ×3個）は物理的操作不可能
    - [ ] スケール無し（移動+回転のみ）の可能性
  - [ ] ボタンデータの位置とフォーマット確認
- [ ] Matrix×Quaternion 2回演算のパフォーマンス測定
- [ ] 射影スケール非対応の確認
- [ ] デッドゾーン・感度調整の最適化
- [ ] 2ボタン機能の実用性評価

### データフォーマット検証用コード

```rust
/// SpaceMouse Compact データフォーマット解析用
pub fn debug_spacemouse_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let api = HidApi::new()?;
    
    // SpaceMouse Compact検索
    for device_info in api.device_list() {
        if device_info.vendor_id() == 0x046D && device_info.product_id() == 0xC652 {
            let device = device_info.open_device(&api)?;
            
            println!("🔍 SpaceMouse Compact検出:");
            println!("   Vendor ID: 0x{:04X}", device_info.vendor_id());
            println!("   Product ID: 0x{:04X}", device_info.product_id());
            println!("   Product: {:?}", device_info.product_string());
            println!("   Serial: {:?}", device_info.serial_number());
            
            // データ読み取りテスト
            println!("\n📊 データ読み取りテスト:");
            for i in 0..10 {
                let mut buffer = [0u8; 32]; // 大きめのバッファで検証
                
                match device.read_timeout(&mut buffer, 1000) {
                    Ok(bytes_read) => {
                        println!("  [{}] {} bytes: {:02X?}", i, bytes_read, &buffer[..bytes_read]);
                        
                        // バイト配列の解析
                        if bytes_read >= 6 {
                            self.analyze_data_format(&buffer[..bytes_read]);
                        }
                    }
                    Err(e) => println!("  [{}] 読み取りエラー: {}", i, e),
                }
                
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            
            break;
        }
    }
    
    Ok(())
}

/// バイト配列からデータ構造を推測
fn analyze_data_format(&self, data: &[u8]) {
    println!("    データ解析:");
    
    // 16bit Little Endian として解析
    for (i, chunk) in data.chunks(2).enumerate() {
        if chunk.len() == 2 {
            let value = i16::from_le_bytes([chunk[0], chunk[1]]);
            let normalized = (value as f32) / 1023.0;
            println!("      軸{}: raw={:6} normalized={:7.4}", i, value, normalized);
        }
    }
    
    // 可能なデータ構造パターン
    if data.len() >= 6 {
        println!("    6軸データ候補 (移動+回転のみ):");
        let x = i16::from_le_bytes([data[0], data[1]]) as f32 / 1023.0;
        let y = i16::from_le_bytes([data[2], data[3]]) as f32 / 1023.0;
        let z = i16::from_le_bytes([data[4], data[5]]) as f32 / 1023.0;
        println!("      Translation: X={:.4}, Y={:.4}, Z={:.4}", x, y, z);
    }
    
    if data.len() >= 12 {
        let rx = i16::from_le_bytes([data[6], data[7]]) as f32 / 1023.0;
        let ry = i16::from_le_bytes([data[8], data[9]]) as f32 / 1023.0;
        let rz = i16::from_le_bytes([data[10], data[11]]) as f32 / 1023.0;
        println!("      Rotation: RX={:.4}, RY={:.4}, RZ={:.4}", rx, ry, rz);
    }
    
    if data.len() >= 14 {
        let scale = i16::from_le_bytes([data[12], data[13]]) as f32 / 1023.0;
        println!("      Scale (等方/プッシュプル): {:.4}", scale);
    }
    
    if data.len() >= 18 {
        println!("    18バイト = 想定外（Compactでは異方スケール操作不可）:");
        let sx = i16::from_le_bytes([data[12], data[13]]) as f32 / 1023.0;
        let sy = i16::from_le_bytes([data[14], data[15]]) as f32 / 1023.0;
        let sz = i16::from_le_bytes([data[16], data[17]]) as f32 / 1023.0;
        println!("      Scale: SX={:.4}, SY={:.4}, SZ={:.4}", sx, sy, sz);
        println!("      ⚠️  物理的操作方法が不明（ハードウェア制約）");
    }
}
```

---

**実装優先度**: SpaceMouse Compact所有により、実機テストが可能。座標変換マトリックス統合アプローチで高い親和性を実現。

---

**注意**: この計画は調査段階であり、実装前に詳細な技術検証が必要です。