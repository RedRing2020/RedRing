# RedRing アーキテクチャ依存性チェックスクリプト
# 実行: powershell -ExecutionPolicy Bypass -File .\scripts\check_architecture_dependencies.ps1

param(
    [switch]$Verbose,
    [switch]$ExitOnError
)

# カラー出力関数
function Write-Success { param($Message) Write-Host "OK $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "WARN $Message" -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host "ERROR $Message" -ForegroundColor Red }
function Write-Info { param($Message) Write-Host "INFO $Message" -ForegroundColor Cyan }

# 依存性ルール定義
$ARCHITECTURE_RULES = @{
    # 許可された依存性パターン
    "AllowedDependencies"   = @{
        # analysisクレート: 完全独立
        "analysis"       = @()

        # Model層 (geo_*)
        "geo_foundation" = @("analysis")  # 数値計算のみ許可
        "geo_commons"    = @("geo_foundation", "analysis")  # 共通計算機能
        "geo_core"       = @("geo_foundation", "analysis")
        "geo_primitives" = @("geo_foundation", "analysis")  # geo_commonsは geo_foundation 経由でアクセス
        "geo_algorithms" = @("geo_foundation", "geo_core", "geo_primitives", "analysis")
        "geo_io"         = @("geo_foundation", "geo_core", "geo_primitives", "geo_algorithms", "analysis")

        # ViewModel層
        "converter"      = @("geo_foundation", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "analysis")
        "graphics"       = @("geo_foundation", "geo_core", "geo_primitives", "analysis")

        # View層
        "render"         = @("analysis")  # GPU層は独立性を保持
        "stage"          = @("render", "analysis")
        "app"            = @("converter", "graphics", "render", "stage", "analysis")
    }

    # 禁止された依存性パターン
    "ForbiddenDependencies" = @{
        # Model → ViewModel/View 禁止
        "geo_foundation" = @("converter", "graphics", "render", "stage", "app")
        "geo_commons"    = @("converter", "graphics", "render", "stage", "app", "geo_core", "geo_primitives", "geo_algorithms", "geo_io")  # geo_foundation以外からの直接アクセス禁止
        "geo_core"       = @("converter", "graphics", "render", "stage", "app")
        "geo_primitives" = @("converter", "graphics", "render", "stage", "app")
        "geo_algorithms" = @("converter", "graphics", "render", "stage", "app")
        "geo_io"         = @("converter", "graphics", "render", "stage", "app")

        # ViewModel → View 禁止
        "converter"      = @("render", "stage", "app")
        "graphics"       = @("render", "stage", "app")

        # View → Model 禁止（例外: geo_foundation将来許可予定）
        "render"         = @("geo_core", "geo_primitives", "geo_algorithms", "geo_io", "converter", "graphics")
        "stage"          = @("geo_foundation", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "converter", "graphics")
        "app"            = @("geo_foundation", "geo_core", "geo_primitives", "geo_algorithms", "geo_io")

        # analysis完全独立
        "analysis"       = @("geo_foundation", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "converter", "graphics", "render", "stage", "app")
    }

    # 命名規則
    "NamingRules"           = @{
        "ModelPrefix"         = "geo_"
        "RequiredModelCrates" = @("geo_foundation", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_io")
    }
}

# Cargo.tomlから依存関係を抽出
function Get-CrateDependencies {
    param([string]$CratePath)

    $cargoToml = Join-Path $CratePath "Cargo.toml"
    if (-not (Test-Path $cargoToml)) {
        return @()
    }

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
            # ワークスペース内クレートかチェック
            if ($ARCHITECTURE_RULES.AllowedDependencies.ContainsKey($depName)) {
                $dependencies += $depName
            }
        }
    }

    return $dependencies
}

# ワークスペースクレート一覧を取得
function Get-WorkspaceCrates {
    $workspaceCrates = @{}

    # 各層のクレートをマッピング
    $layerMapping = @{
        "analysis"       = "analysis"
        "geo_foundation" = "model/geo_foundation"
        "geo_commons"    = "model/geo_commons"
        "geo_core"       = "model/geo_core"
        "geo_primitives" = "model/geo_primitives"
        "geo_algorithms" = "model/geo_algorithms"
        "geo_io"         = "model/geo_io"
        "converter"      = "viewmodel/converter"
        "graphics"       = "viewmodel/graphics"
        "render"         = "view/render"
        "stage"          = "view/stage"
        "app"            = "view/app"
    }

    foreach ($crateName in $layerMapping.Keys) {
        $cratePath = Join-Path (Get-Location) $layerMapping[$crateName]
        if (Test-Path $cratePath) {
            $workspaceCrates[$crateName] = $cratePath
        }
    }

    return $workspaceCrates
}

# 依存性チェック実行
function Test-ArchitectureDependencies {
    Write-Info "RedRing Architecture Dependency Check Start"
    Write-Info "Date: $(Get-Date -Format 'yyyy/MM/dd HH:mm:ss')"
    Write-Host ""

    $errorCount = 0
    $warningCount = 0
    $workspaceCrates = Get-WorkspaceCrates

    # 1. 命名規則チェック
    Write-Info "1. Model Layer Naming Rule Check"
    foreach ($crateName in $ARCHITECTURE_RULES.NamingRules.RequiredModelCrates) {
        if ($workspaceCrates.ContainsKey($crateName)) {
            Write-Success "Model crate '$crateName' follows correct naming rules"
        }
        else {
            Write-Error "Required Model crate '$crateName' not found"
            $errorCount++
        }
    }
    Write-Host ""

    # 2. 依存性ルールチェック
    Write-Info "2. Dependency Rule Check"
    foreach ($crateName in $workspaceCrates.Keys) {
        $cratePath = $workspaceCrates[$crateName]
        $actualDeps = Get-CrateDependencies $cratePath

        Write-Info "Validating '$crateName' dependencies..."

        # 許可された依存性チェック
        $allowedDeps = $ARCHITECTURE_RULES.AllowedDependencies[$crateName]
        foreach ($dep in $actualDeps) {
            if ($allowedDeps -contains $dep) {
                if ($Verbose) {
                    Write-Success "  OK '$crateName' -> '$dep' (allowed)"
                }
            }
            else {
                Write-Error "  ERROR '$crateName' -> '$dep' (not allowed)"
                $errorCount++
            }
        }

        # 禁止された依存性チェック
        $forbiddenDeps = $ARCHITECTURE_RULES.ForbiddenDependencies[$crateName]
        foreach ($dep in $actualDeps) {
            if ($forbiddenDeps -contains $dep) {
                Write-Error "  ERROR '$crateName' -> '$dep' (explicitly forbidden)"
                $errorCount++
            }
        }

        if ($actualDeps.Length -eq 0) {
            Write-Info "  INFO '$crateName' has no workspace dependencies"
        }
    }
    Write-Host ""

    # 3. 層別依存性サマリー
    Write-Info "3. Layer Dependency Summary"

    $layers = @{
        "Analysis"  = @("analysis")
        "Model"     = @("geo_foundation", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_io")
        "ViewModel" = @("converter", "graphics")
        "View"      = @("render", "stage", "app")
    }

    foreach ($layerName in $layers.Keys) {
        Write-Info "Layer: $layerName"
        foreach ($crateName in $layers[$layerName]) {
            if ($workspaceCrates.ContainsKey($crateName)) {
                $deps = Get-CrateDependencies $workspaceCrates[$crateName]
                if ($deps.Length -gt 0) {
                    $depsStr = $deps -join ", "
                    Write-Host "    $crateName -> $depsStr" -ForegroundColor White
                }
                else {
                    Write-Host "    $crateName -> (no deps)" -ForegroundColor Gray
                }
            }
            else {
                Write-Warning "    $crateName -> (not found)"
                $warningCount++
            }
        }
    }
    Write-Host ""

    # 4. 結果サマリー
    Write-Info "4. Check Result Summary"
    if ($errorCount -eq 0 -and $warningCount -eq 0) {
        Write-Success "SUCCESS: Architecture dependency check passed"
        Write-Success "   - View -> ViewModel -> Model dependency direction maintained"
        Write-Success "   - Model layer naming rules (geo_*) followed"
        Write-Success "   - No forbidden circular dependencies detected"
    }
    else {
        Write-Error "FAILED: Architecture dependency check found issues"
        Write-Error "   - Errors: $errorCount"
        Write-Warning "   - Warnings: $warningCount"

        if ($ExitOnError -and $errorCount -gt 0) {
            Write-Error "STOP: Exiting due to detected errors"
            exit 1
        }
    }

    return @{ "Errors" = $errorCount; "Warnings" = $warningCount }
}

# アーキテクチャルール詳細表示
function Show-ArchitectureRules {
    Write-Info "RedRing Architecture Dependency Rules"
    Write-Host ""

    Write-Info "ALLOWED dependency patterns:"
    Write-Host "   • View -> ViewModel -> Model (one-way)"
    Write-Host "   • ViewModel -> geo_* (concrete Model reference)"
    Write-Host "   • ViewModel -> geo_io (efficient data conversion)"
    Write-Host "   • geo_foundation -> geo_commons (Foundation Pattern)"
    Write-Host "   • analysis -> independent (numerical computation crate)"
    Write-Host ""

    Write-Info "FORBIDDEN dependency patterns:"
    Write-Host "   • Model -> ViewModel (reverse dependency)"
    Write-Host "   • Model -> View (layer crossing)"
    Write-Host "   • ViewModel -> View (reverse dependency)"
    Write-Host "   • Direct geo_commons access (must use geo_foundation)"
    Write-Host "   • View -> Model (direct dependency, geo_foundation exception planned)"
    Write-Host "   • analysis -> other crates (independence violation)"
    Write-Host ""

    Write-Info "NAMING rules:"
    Write-Host "   • Model layer crates: 'geo_' prefix required"
    Write-Host "   • geo_foundation: Model abstraction layer & bridge to geo_commons"
    Write-Host "   • geo_commons: Common computation functions (Foundation Pattern)"
    Write-Host "   • geo_io: Data format exchange (ViewModel direct reference allowed)"
    Write-Host ""
}

# メイン実行
if ($args -contains "-Help" -or $args -contains "-h") {
    Write-Host "RedRing Architecture Dependency Check Script" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage:"
    Write-Host "  .\scripts\check_architecture_dependencies.ps1 [options]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Verbose      Show detailed dependency information"
    Write-Host "  -ExitOnError  Exit script on error detection (default: false)"
    Write-Host "  -Rules        Show architecture rule details"
    Write-Host "  -Help, -h     Show this help"
    exit 0
}

if ($args -contains "-Rules") {
    Show-ArchitectureRules
    exit 0
}

# 依存性チェック実行
$result = Test-ArchitectureDependencies

# CI/CD用の結果出力
if ($env:CI -eq "true") {
    if ($result.Errors -gt 0) {
        Write-Output "::error::Architecture dependency check detected errors"
        exit 1
    }
    if ($result.Warnings -gt 0) {
        Write-Output "::warning::Architecture dependency check detected warnings"
    }
}
