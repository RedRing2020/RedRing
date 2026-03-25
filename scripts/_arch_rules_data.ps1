# RedRing Architecture Rules - Shared Data
# This file is dot-sourced by check_architecture_dependencies*.ps1
# DO NOT execute directly.

# Workspace crate path mapping
$ARCH_LAYER_MAPPING = @{
    analysis       = "foundation/analysis"
    geo_contracts  = "model/geo_contracts"
    geo_commons    = "model/geo_commons"
    geo_core       = "model/geo_core"
    geo_primitives = "model/geo_primitives"
    geo_algorithms = "model/geo_algorithms"
    geo_nurbs      = "model/geo_nurbs"
    geo_io         = "model/geo_io"
    geo_entity     = "model/geo_entity"
    cam_core       = "model/cam_core"
    cam_entity     = "model/cam_entity"
    cam_sim        = "model/cam_sim"
    job_runtime    = "model/job_runtime"
    job_domain     = "model/job_domain"
    converter      = "viewmodel/converter"
    graphics       = "viewmodel/graphics"
    render         = "view/render"
    stage          = "view/stage"
    app            = "view/app"
}

# Layer groups
$ARCH_LAYERS = @{
    Analysis  = @("analysis")
    Model     = @("geo_contracts", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_nurbs", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim", "job_runtime", "job_domain")
    ViewModel = @("converter", "graphics")
    View      = @("render", "stage", "app")
}

# Issue #413 design note:
# `cam_algorithms` 新設時は以下を同一変更セットで反映する。
# 1. ARCH_LAYER_MAPPING に `cam_algorithms = "model/cam_algorithms"` を追加
# 2. ARCH_LAYERS.Model に `cam_algorithms` を追加
# 3. ARCH_ALLOWED_DEPS に以下を追加
#    cam_algorithms = @("analysis", "cam_core", "geo_contracts", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_nurbs")
# 4. 既存クレート側の許可依存を更新
#    - cam_sim: `cam_algorithms` を限定的に許可
#    - converter: 必要になるまで `cam_algorithms` は許可しない
#    - cam_entity: `cam_algorithms` は許可しない
# 5. ARCH_FORBIDDEN_DEPS に以下を反映
#    - cam_core -> cam_algorithms を禁止
#    - cam_entity -> cam_algorithms を禁止
#    - cam_algorithms -> cam_sim を禁止
#    - geo_* -> cam_algorithms を禁止
# 6. クレート作成後に ARCH_REQUIRED_MODEL_CRATES へ追加する
# 注意: クレート未作成の段階で ARCH_REQUIRED_MODEL_CRATES へ追加すると、依存チェックが必須クレート不足で失敗する。

# Required model crates
$ARCH_REQUIRED_MODEL_CRATES = @("geo_contracts", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "geo_entity")

# Allowed dependency rules
# Last updated: 2026-03-20 (#318 Phase C: contracts-first dependency alignment)
# Pending update: Issue #413 で `cam_algorithms` 新設時に CAM 依存ルールを追加する。
$ARCH_ALLOWED_DEPS = @{
    analysis       = @()
    geo_contracts  = @("analysis", "geo_commons")
    geo_commons    = @("analysis")
    geo_core       = @("analysis", "geo_entity")
    geo_primitives = @("geo_contracts", "geo_commons", "geo_core", "analysis")
    geo_algorithms = @("geo_contracts", "geo_commons", "geo_core", "geo_primitives", "geo_nurbs", "analysis")
    geo_nurbs      = @("geo_contracts", "geo_commons", "geo_core", "geo_primitives", "analysis")
    geo_io         = @("geo_contracts", "geo_core", "geo_primitives", "geo_algorithms", "analysis")
    geo_entity     = @("geo_contracts", "geo_primitives")
    cam_core       = @("analysis", "geo_contracts", "geo_primitives", "geo_algorithms")
    cam_entity     = @("cam_core", "geo_entity")
    cam_sim        = @("analysis", "cam_core", "geo_algorithms", "job_runtime", "job_domain")
    job_runtime    = @("analysis")
    job_domain     = @("analysis", "job_runtime")
    converter      = @("geo_contracts", "geo_algorithms", "geo_io", "cam_core", "cam_entity", "cam_sim", "job_domain", "analysis")
    graphics       = @("geo_contracts", "geo_core", "geo_primitives", "analysis")
    render         = @("analysis")
    stage          = @("render", "analysis")
    app            = @("converter", "graphics", "render", "stage", "analysis")
}

# Forbidden dependency rules
# Pending update: `cam_algorithms` 新設時は CAM 内の逆依存禁止をここへ追加する。
$ARCH_FORBIDDEN_DEPS = @{
    geo_foundation = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")
    geo_contracts  = @("geo_foundation", "geo_core", "geo_primitives", "geo_nurbs", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim", "job_runtime", "graphics", "render", "stage", "app")
    geo_commons    = @("converter", "graphics", "render", "stage", "app", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "cam_core", "cam_entity", "cam_sim", "job_runtime")
    geo_core       = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")
    geo_primitives = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")
    geo_algorithms = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")
    geo_nurbs      = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")
    geo_io         = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")
    geo_entity     = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")
    cam_core       = @("converter", "graphics", "render", "stage", "app", "geo_entity", "cam_sim")
    cam_entity     = @("converter", "graphics", "render", "stage", "app", "cam_sim")
    cam_sim        = @("converter", "graphics", "render", "stage", "app", "geo_entity", "geo_foundation", "geo_core", "geo_primitives", "geo_nurbs", "geo_io", "cam_entity")
    job_runtime    = @("geo_foundation", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_nurbs", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim", "converter", "graphics", "render", "stage", "app")
    job_domain     = @("geo_foundation", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_nurbs", "geo_io", "geo_entity", "converter", "graphics", "render", "stage", "app")
    converter      = @("render", "stage", "app")
    graphics       = @("render", "stage", "app")
    render         = @("geo_core", "geo_primitives", "geo_algorithms", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim", "converter", "graphics")
    stage          = @("geo_foundation", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim", "converter", "graphics")
    app            = @("geo_foundation", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim")
    analysis       = @("geo_foundation", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_nurbs", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim", "job_runtime", "converter", "graphics", "render", "stage", "app")
}

# Shared helper: extract workspace dependencies from Cargo.toml
function Get-CrateDependenciesShared {
    param([string]$CratePath)

    $cargoToml = Join-Path $CratePath "Cargo.toml"
    if (-not (Test-Path $cargoToml)) {
        return @()
    }

    $knownCrates = $ARCH_ALLOWED_DEPS.Keys
    $dependencies = @()
    $content = Get-Content $cargoToml
    $inDepsSection = $false

    foreach ($line in $content) {
        if ($line -match '^\[dependencies\]') {
            $inDepsSection = $true
            continue
        }
        if ($line -match '^\[.*\]' -and $inDepsSection) {
            break
        }
        if ($inDepsSection -and $line -match '^(\w+)\s*=') {
            $depName = $matches[1]
            if ($knownCrates -contains $depName) {
                $dependencies += $depName
            }
        }
    }

    return $dependencies
}

# Shared helper: return workspace crate path map
function Get-WorkspaceCratesShared {
    $result = @{}
    foreach ($crateName in $ARCH_LAYER_MAPPING.Keys) {
        $cratePath = Join-Path (Get-Location) $ARCH_LAYER_MAPPING[$crateName]
        if (Test-Path $cratePath) {
            $result[$crateName] = $cratePath
        }
    }
    return $result
}
