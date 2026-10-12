# RedRing Architecture Rules - Shared Data
# This file is dot-sourced by check_architecture_dependencies.ps1
# DO NOT execute directly.
#
# クレート名は Cargo.toml のパッケージ名で書く（ディレクトリ名ではない）。
# ワークスペースのクレートと依存は `cargo metadata` から取得する。
# ルールにないクレート・依存はエラーになるため、クレートを追加したら同じ変更でここを更新する。

# Layer groups
$ARCH_LAYERS = [ordered]@{
    Foundation  = @("analysis", "i18n_foundation", "logging_foundation")
    TestSupport = @("redring_test_support")
    Model       = @("geo_contracts", "geo_commons", "geo_core", "geo_primitives", "geo_nurbs", "geo_topology", "geo_algorithms", "geo_io", "geo_entity", "cam_core", "cam_algorithms", "cam_entity", "cam_sim", "job_runtime", "job_domain")
    Application = @("application")
    ViewModel   = @("viewmodel", "viewmodel-cam-demo", "viewmodel-graphics")
    View        = @("render", "stage", "redring")
}

# Required model crates
$ARCH_REQUIRED_MODEL_CRATES = @("geo_contracts", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "geo_entity", "cam_algorithms")

# Allowed dependency rules（通常の依存・dev-dependencies・build-dependencies を含む）
$ARCH_ALLOWED_DEPS = @{
    analysis             = @()
    i18n_foundation      = @()
    logging_foundation   = @()
    redring_test_support = @()

    geo_contracts  = @("analysis", "geo_core")
    geo_commons    = @("analysis")
    geo_core       = @("analysis")
    geo_primitives = @("analysis", "geo_contracts", "geo_commons", "geo_core")
    geo_nurbs      = @("analysis", "geo_contracts", "geo_commons", "geo_core")
    geo_topology   = @("analysis", "geo_contracts", "geo_commons", "geo_core", "geo_primitives", "geo_nurbs")
    geo_algorithms = @("analysis", "geo_contracts", "geo_commons", "geo_core", "geo_primitives", "geo_nurbs", "geo_topology")
    geo_io         = @("analysis", "geo_contracts", "geo_core", "geo_primitives", "geo_algorithms", "logging_foundation")
    geo_entity     = @("geo_contracts", "geo_primitives")
    cam_core       = @("analysis", "geo_contracts", "geo_primitives", "geo_algorithms")
    cam_algorithms = @("analysis", "cam_core", "geo_contracts", "geo_algorithms")
    cam_entity     = @("cam_core", "geo_entity")
    cam_sim        = @("analysis", "cam_core", "cam_algorithms", "geo_algorithms", "job_runtime", "job_domain", "redring_test_support")
    job_runtime    = @("analysis", "redring_test_support")
    job_domain     = @("analysis", "job_runtime")

    application    = @("analysis", "geo_contracts", "geo_algorithms", "geo_entity", "geo_primitives", "geo_topology", "cam_core", "cam_algorithms", "cam_sim", "job_runtime", "job_domain", "redring_test_support")

    viewmodel            = @("analysis", "i18n_foundation", "logging_foundation", "application", "geo_contracts", "geo_algorithms", "geo_io", "cam_core", "cam_entity", "cam_sim", "job_domain")
    "viewmodel-cam-demo" = @("application", "cam_core", "cam_sim", "geo_algorithms", "viewmodel")
    "viewmodel-graphics" = @("analysis", "geo_contracts", "geo_core", "geo_primitives")

    render  = @("analysis", "logging_foundation", "viewmodel", "viewmodel-graphics")
    stage   = @("analysis", "logging_foundation", "render", "viewmodel", "viewmodel-graphics")
    redring = @("analysis", "logging_foundation", "render", "stage", "viewmodel", "viewmodel-cam-demo", "viewmodel-graphics")
}

# Forbidden dependency rules
# 許可されていない依存はすべてエラーになる。ここには、特に守る境界を明示する。
$ARCH_VIEW_AND_VIEWMODEL = @("viewmodel", "viewmodel-cam-demo", "viewmodel-graphics", "render", "stage", "redring")
$ARCH_CAM_CRATES = @("cam_core", "cam_algorithms", "cam_entity", "cam_sim")
$ARCH_GEO_CRATES = @("geo_contracts", "geo_commons", "geo_core", "geo_primitives", "geo_nurbs", "geo_topology", "geo_algorithms", "geo_io", "geo_entity")
$ARCH_FORBIDDEN_DEPS = @{
    # 基盤はワークスペースの他のクレートに依存しない
    analysis           = $ARCH_GEO_CRATES + $ARCH_CAM_CRATES + $ARCH_VIEW_AND_VIEWMODEL + @("application", "job_runtime", "job_domain")
    # trait定義の正本（抽象化）は形状のクレートに依存しない（値型の geo_core には依存する）
    geo_contracts      = @("geo_primitives", "geo_nurbs", "geo_topology", "geo_algorithms", "geo_io", "geo_entity") + $ARCH_CAM_CRATES + $ARCH_VIEW_AND_VIEWMODEL
    # 共通計算は形状のクレートに依存しない
    geo_commons        = @("geo_contracts", "geo_core", "geo_primitives", "geo_nurbs", "geo_topology", "geo_algorithms") + $ARCH_CAM_CRATES + $ARCH_VIEW_AND_VIEWMODEL
    # 幾何のクレートは CAM・ビューモデル・ビューに依存しない（CAD / CAM の境界）
    # 値型は抽象化・形状に依存しない
    geo_core           = @("geo_contracts", "geo_commons", "geo_primitives", "geo_nurbs", "geo_topology", "geo_algorithms", "geo_io", "geo_entity") + $ARCH_CAM_CRATES + $ARCH_VIEW_AND_VIEWMODEL
    geo_primitives     = @("geo_nurbs") + $ARCH_CAM_CRATES + $ARCH_VIEW_AND_VIEWMODEL
    geo_nurbs          = @("geo_primitives", "geo_topology") + $ARCH_CAM_CRATES + $ARCH_VIEW_AND_VIEWMODEL
    geo_topology       = $ARCH_CAM_CRATES + $ARCH_VIEW_AND_VIEWMODEL
    geo_algorithms     = $ARCH_CAM_CRATES + $ARCH_VIEW_AND_VIEWMODEL
    geo_io             = $ARCH_CAM_CRATES + $ARCH_VIEW_AND_VIEWMODEL
    geo_entity         = $ARCH_CAM_CRATES + $ARCH_VIEW_AND_VIEWMODEL
    cam_core           = @("cam_algorithms", "cam_entity", "cam_sim", "geo_entity") + $ARCH_VIEW_AND_VIEWMODEL
    cam_algorithms     = @("cam_entity", "cam_sim", "geo_entity", "application") + $ARCH_VIEW_AND_VIEWMODEL
    cam_entity         = @("cam_algorithms", "cam_sim") + $ARCH_VIEW_AND_VIEWMODEL
    cam_sim            = @("cam_entity", "geo_entity") + $ARCH_VIEW_AND_VIEWMODEL
    job_runtime        = $ARCH_GEO_CRATES + $ARCH_CAM_CRATES + $ARCH_VIEW_AND_VIEWMODEL
    job_domain         = $ARCH_GEO_CRATES + $ARCH_VIEW_AND_VIEWMODEL
    application        = $ARCH_VIEW_AND_VIEWMODEL
    # ビューはモデルのクレートに直接依存しない（ビューモデル経由）
    render             = $ARCH_GEO_CRATES + $ARCH_CAM_CRATES + @("application", "job_runtime", "job_domain")
    stage              = $ARCH_GEO_CRATES + $ARCH_CAM_CRATES + @("application", "job_runtime", "job_domain")
    redring            = $ARCH_GEO_CRATES + $ARCH_CAM_CRATES + @("application", "job_runtime", "job_domain")
}

# Shared helper: ワークスペースのクレートと、ワークスペース内の依存（path 依存）を返す
# 戻り値: パッケージ名 → @{ Path = <ディレクトリ>; Deps = <依存するパッケージ名の配列> }
function Get-WorkspaceCratesShared {
    $json = cargo metadata --no-deps --format-version 1 2>$null
    if ($LASTEXITCODE -ne 0 -or -not $json) {
        throw "cargo metadata failed"
    }
    $metadata = $json | ConvertFrom-Json
    $result = [ordered]@{}
    foreach ($package in ($metadata.packages | Sort-Object name)) {
        $deps = @($package.dependencies | Where-Object { $_.path } | ForEach-Object { $_.name } | Sort-Object -Unique)
        $result[$package.name] = @{
            Path = Split-Path -Parent $package.manifest_path
            Deps = $deps
        }
    }
    return $result
}
