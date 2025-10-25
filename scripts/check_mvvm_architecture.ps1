# MVVM Architecture Dependency Check Script
# CI/CD Architecture Validation

param(
    [string]$WorkspaceRoot = (Get-Location).Path
)

Write-Host "🏗️  MVVM Architecture Dependency Check Started" -ForegroundColor Green
Write-Host "📁 Workspace: $WorkspaceRoot" -ForegroundColor Gray

$ErrorCount = 0
$Violations = @()

# Architecture Rules Definition
$ArchitectureRules = @{
    # View layer (render, stage) should NOT depend on Model layer (geo_*) directly
    "render" = @{
        "ForbiddenDeps" = @("geo_primitives", "geo_foundation", "geo_core", "geo_algorithms", "geo_io")
        "AllowedDeps" = @("viewmodel", "wgpu", "winit", "bytemuck", "tracing", "pollster")
    }
    "stage" = @{
        "ForbiddenDeps" = @("geo_primitives", "geo_foundation", "geo_core", "geo_algorithms", "geo_io")
        "AllowedDeps" = @("render", "viewmodel", "wgpu", "bytemuck", "tracing")
    }
    # ViewModel layer can depend on Model layer
    "viewmodel" = @{
        "AllowedDeps" = @("model", "geo_primitives", "geo_foundation", "tracing")
        "ForbiddenDeps" = @()  # No specific restrictions
    }
    # Model layer should NOT depend on other layers (pure domain logic)
    "model" = @{
        "ForbiddenDeps" = @("render", "stage", "viewmodel", "redring")
        "AllowedDeps" = @()  # External libraries only
    }
}

function Test-Dependencies {
    param(
        [string]$CrateName,
        [hashtable]$Rules
    )
    
    $CargoTomlPath = Join-Path $WorkspaceRoot "$CrateName/Cargo.toml"
    
    if (-not (Test-Path $CargoTomlPath)) {
        Write-Host "⚠️  $CrateName Cargo.toml not found: $CargoTomlPath" -ForegroundColor Yellow
        return @()
    }
    
    Write-Host "🔍 Checking $CrateName dependencies..." -ForegroundColor Cyan
    
    $CargoContent = Get-Content $CargoTomlPath -Raw
    
    # Check forbidden dependencies
    if ($Rules.ContainsKey("ForbiddenDeps")) {
        foreach ($ForbiddenDep in $Rules["ForbiddenDeps"]) {
            if ($CargoContent -match "^\s*$ForbiddenDep\s*=") {
                $script:ErrorCount++
                $Violation = "❌ $CrateName → $ForbiddenDep (MVVM Violation: View layer directly depends on Model layer)"
                $script:Violations += $Violation
                Write-Host $Violation -ForegroundColor Red
            }
        }
    }
    
    # Extract actual dependencies
    $ActualDeps = @()
    $Lines = $CargoContent -split "`n"
    $InDependencies = $false
    
    foreach ($Line in $Lines) {
        if ($Line -match '^\[dependencies\]') {
            $InDependencies = $true
            continue
        }
        if ($Line -match '^\[') {
            $InDependencies = $false
            continue
        }
        if ($InDependencies -and $Line -match '^\s*(\w+)\s*=') {
            $DepName = $Matches[1]
            $ActualDeps += $DepName
        }
    }
    
    Write-Host "   Dependencies: $($ActualDeps -join ', ')" -ForegroundColor Gray
    
    return $ActualDeps
}

# Check each crate dependencies
foreach ($CrateName in $ArchitectureRules.Keys) {
    $Dependencies = Test-Dependencies -CrateName $CrateName -Rules $ArchitectureRules[$CrateName]
}

# Cross-check with cargo tree
Write-Host "`n🌳 Cross-checking with cargo tree..." -ForegroundColor Cyan
try {
    $TreeOutput = & cargo tree --depth 2 2>$null
    
    # Check render for geo_* dependencies
    $RenderLines = $TreeOutput | Where-Object { $_ -match "render v" }
    foreach ($Line in $RenderLines) {
        if ($Line -match "geo_(primitives|foundation|core|algorithms|io)") {
            $script:ErrorCount++
            $Violation = "❌ Detected by cargo tree: render depends on geo_* - $Line"
            $script:Violations += $Violation
            Write-Host $Violation -ForegroundColor Red
        }
    }
    
    # Check stage for geo_* dependencies
    $StageLines = $TreeOutput | Where-Object { $_ -match "stage v" }
    foreach ($Line in $StageLines) {
        if ($Line -match "geo_(primitives|foundation|core|algorithms|io)") {
            $script:ErrorCount++
            $Violation = "❌ Detected by cargo tree: stage depends on geo_* - $Line"
            $script:Violations += $Violation
            Write-Host $Violation -ForegroundColor Red
        }
    }
} catch {
    Write-Host "⚠️  cargo tree execution failed: $($_.Exception.Message)" -ForegroundColor Yellow
}

# Results Report
Write-Host "`n📊 MVVM Architecture Check Results" -ForegroundColor Green
Write-Host "=" * 50 -ForegroundColor Gray

if ($ErrorCount -eq 0) {
    Write-Host "✅ MVVM Architecture Compliant: No violations detected" -ForegroundColor Green
    Write-Host "   - View layer (render, stage) accesses Model layer only through ViewModel" -ForegroundColor Gray
    Write-Host "   - Dependency separation properly implemented" -ForegroundColor Gray
} else {
    Write-Host "❌ MVVM Architecture Violations: $ErrorCount issues detected" -ForegroundColor Red
    Write-Host "`nViolation Details:" -ForegroundColor Yellow
    foreach ($Violation in $Violations) {
        Write-Host "  $Violation" -ForegroundColor Red
    }
    Write-Host "`nHow to Fix:" -ForegroundColor Yellow
    Write-Host "  1. Remove direct dependencies from View layer (render, stage) to geo_*" -ForegroundColor Gray
    Write-Host "  2. Access Model data only through viewmodel" -ForegroundColor Gray
    Write-Host "  3. Remove forbidden dependencies from Cargo.toml" -ForegroundColor Gray
}

Write-Host "`n🔧 Recommended Architecture:" -ForegroundColor Blue
Write-Host "  redring → viewmodel → geo_* (Model)" -ForegroundColor Gray
Write-Host "         ↘ render → viewmodel (View)" -ForegroundColor Gray
Write-Host "         ↘ stage → render" -ForegroundColor Gray

# Exit code for CI/CD
exit $ErrorCount