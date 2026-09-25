# RedRing Architecture Rules - Shared Data
# This file is dot-sourced by check_architecture_dependencies*.ps1
# DO NOT execute directly.

# Workspace crate path mapping
$ARCH_LAYER_MAPPING = @{
    analysis       = "foundation/analysis"
    application    = "model/application"
    geo_contracts  = "model/geo_contracts"
    geo_commons    = "model/geo_commons"
    geo_core       = "model/geo_core"
    geo_primitives = "model/geo_primitives"
    geo_topology   = "model/geo_topology"
    geo_algorithms = "model/geo_algorithms"
    geo_nurbs      = "model/geo_nurbs"
    geo_io         = "model/geo_io"
    geo_entity     = "model/geo_entity"
    cam_core       = "model/cam_core"
    cam_algorithms = "model/cam_algorithms"
    cam_entity     = "model/cam_entity"
    cam_sim        = "model/cam_sim"
    job_runtime    = "model/job_runtime"
    job_domain     = "model/job_domain"
    redring_test_support = "model/test_support"
    converter      = "viewmodel/converter"
    cam_demo       = "viewmodel/cam_demo"
    graphics       = "viewmodel/graphics"
    render         = "view/render"
    stage          = "view/stage"
    app            = "view/app"
}

# Layer groups
$ARCH_LAYERS = @{
    Analysis  = @("analysis")
    Application = @("application")
    Model     = @("geo_contracts", "geo_commons", "geo_core", "geo_primitives", "geo_topology", "geo_algorithms", "geo_nurbs", "geo_io", "geo_entity", "cam_core", "cam_algorithms", "cam_entity", "cam_sim", "job_runtime", "job_domain")
    TestSupport = @("redring_test_support")
    ViewModel = @("converter", "cam_demo", "graphics")
    View      = @("render", "stage", "app")
}

# Issue #413 design note: `cam_algorithms` was created in #684.
# Mapping, Model layer, allowed/forbidden dependency rules, and required crates were updated in one change set.

# Required model crates
$ARCH_REQUIRED_MODEL_CRATES = @("geo_contracts", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "geo_entity", "cam_algorithms")

# Allowed dependency rules
# Last updated: 2026-04-09 (#650: allow geo_topology -> geo_nurbs for NURBS curve edge topology, sync application allowed deps with current crate)
# 2026-09-25 (#684): add cam_algorithms (cam_sim/application -> cam_algorithms allowed)
$ARCH_ALLOWED_DEPS = @{
    analysis       = @()
    application    = @("analysis", "geo_contracts", "geo_algorithms", "geo_entity", "geo_primitives", "geo_topology", "cam_core", "cam_algorithms", "cam_sim", "job_runtime", "job_domain", "redring_test_support")
    geo_contracts  = @("analysis", "geo_commons")
    geo_commons    = @("analysis")
    geo_core       = @("analysis", "geo_contracts", "geo_entity")
    geo_primitives = @("geo_contracts", "geo_commons", "geo_core", "analysis")
    geo_topology   = @("geo_contracts", "geo_commons", "geo_core", "geo_primitives", "geo_nurbs", "analysis")
    geo_algorithms = @("geo_contracts", "geo_commons", "geo_core", "geo_primitives", "geo_topology", "geo_nurbs", "analysis")
    geo_nurbs      = @("geo_contracts", "geo_commons", "geo_core", "geo_primitives", "analysis")
    geo_io         = @("geo_contracts", "geo_core", "geo_primitives", "geo_algorithms", "analysis")
    geo_entity     = @("geo_contracts", "geo_primitives")
    cam_core       = @("analysis", "geo_contracts", "geo_primitives", "geo_algorithms")
    cam_algorithms = @("analysis", "cam_core", "geo_contracts", "geo_algorithms")
    cam_entity     = @("cam_core", "geo_entity")
    cam_sim        = @("analysis", "cam_core", "cam_algorithms", "geo_algorithms", "job_runtime", "job_domain", "redring_test_support")
    job_runtime    = @("analysis", "redring_test_support")
    job_domain     = @("analysis", "job_runtime")
    redring_test_support = @()
    converter      = @(
        "application",
        "geo_contracts",
        "geo_algorithms",
        "geo_io",
        "cam_core",
        "cam_entity",
        "cam_sim",
        "job_domain",
        "analysis"
    );
    cam_demo       = @("application", "cam_core", "cam_sim", "converter", "geo_algorithms")
    graphics       = @("geo_contracts", "geo_core", "geo_primitives", "analysis")
    render         = @("analysis")
    stage          = @("render", "analysis")
    app            = @("converter", "cam_demo", "graphics", "render", "stage", "analysis")
}

# Forbidden dependency rules
# Last updated: 2026-04-09 (#501/#650 sync application forbidden deps with current allowed deps)
# 2026-09-25 (#684): add cam_algorithms reverse dependency guards
$ARCH_FORBIDDEN_DEPS = @{
    application    = @("geo_foundation", "geo_commons", "geo_core", "geo_nurbs", "geo_io", "cam_entity", "converter", "cam_demo", "graphics", "render", "stage", "app")
    geo_foundation = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_algorithms", "cam_entity", "cam_sim", "job_runtime")
    geo_contracts  = @("geo_foundation", "geo_core", "geo_primitives", "geo_nurbs", "geo_io", "geo_entity", "cam_core", "cam_algorithms", "cam_entity", "cam_sim", "job_runtime", "graphics", "render", "stage", "app")
    geo_commons    = @("converter", "graphics", "render", "stage", "app", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "cam_core", "cam_algorithms", "cam_entity", "cam_sim", "job_runtime")
    geo_core       = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_algorithms", "cam_entity", "cam_sim", "job_runtime")
    geo_primitives = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_algorithms", "cam_entity", "cam_sim", "job_runtime")
    geo_topology   = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_algorithms", "cam_entity", "cam_sim", "job_runtime")
    geo_algorithms = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_algorithms", "cam_entity", "cam_sim", "job_runtime")
    geo_nurbs      = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_algorithms", "cam_entity", "cam_sim", "job_runtime")
    geo_io         = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_algorithms", "cam_entity", "cam_sim", "job_runtime")
    geo_entity     = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_algorithms", "cam_entity", "cam_sim", "job_runtime")
    cam_core       = @("converter", "graphics", "render", "stage", "app", "geo_entity", "cam_sim", "cam_algorithms")
    cam_entity     = @("converter", "graphics", "render", "stage", "app", "cam_sim", "cam_algorithms")
    cam_algorithms = @("converter", "graphics", "render", "stage", "app", "geo_entity", "cam_entity", "cam_sim", "job_runtime", "job_domain", "application", "cam_demo")
    cam_sim        = @("converter", "graphics", "render", "stage", "app", "geo_entity", "geo_foundation", "geo_core", "geo_primitives", "geo_nurbs", "geo_io", "cam_entity")
    job_runtime    = @("geo_foundation", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_nurbs", "geo_io", "geo_entity", "cam_core", "cam_algorithms", "cam_entity", "cam_sim", "converter", "graphics", "render", "stage", "app")
    job_domain     = @("geo_foundation", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_nurbs", "geo_io", "geo_entity", "converter", "graphics", "render", "stage", "app")
    redring_test_support = @("geo_foundation", "application", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_nurbs", "geo_io", "geo_entity", "cam_core", "cam_algorithms", "cam_entity", "cam_sim", "job_runtime", "job_domain", "converter", "cam_demo", "graphics", "render", "stage", "app", "analysis")
    converter      = @(
        "cam_demo",
        "render",
        "stage",
        "app"
    );
    cam_demo       = @("geo_foundation", "geo_commons", "geo_core", "geo_primitives", "geo_nurbs", "geo_io", "geo_entity", "cam_entity", "cam_algorithms", "job_runtime", "job_domain", "graphics", "render", "stage", "app")
    graphics       = @("render", "stage", "app")
    render         = @("geo_core", "geo_primitives", "geo_algorithms", "geo_io", "geo_entity", "cam_core", "cam_algorithms", "cam_entity", "cam_sim", "converter", "cam_demo", "graphics")
    stage          = @("geo_foundation", "application", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "geo_entity", "cam_core", "cam_algorithms", "cam_entity", "cam_sim", "converter", "cam_demo", "graphics")
    app            = @("geo_foundation", "application", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "geo_entity", "cam_core", "cam_algorithms", "cam_entity", "cam_sim")
    analysis       = @("geo_foundation", "application", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_nurbs", "geo_io", "geo_entity", "cam_core", "cam_algorithms", "cam_entity", "cam_sim", "job_runtime", "converter", "cam_demo", "graphics", "render", "stage", "app")
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
    $depsSections = @("dependencies", "dev-dependencies")
    $inDepsSection = $false

    foreach ($line in $content) {
        if ($line -match '^\[(.+)\]') {
            $sectionName = $matches[1].Trim().ToLowerInvariant()
            $inDepsSection = $depsSections -contains $sectionName
            continue
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
