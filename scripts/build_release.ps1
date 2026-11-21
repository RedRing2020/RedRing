<#
.SYNOPSIS
    リリースの配布物と同じ設定でビルドし、このビルドで使わなかった古い成果物を消す
.DESCRIPTION
    .github/workflows/release.yml と同じ対象（-p redring -p job_manager_app -p job_runner）と RUSTFLAGS で
    cargo build --release --locked を行います。cargo は、変わった部分だけを作り直します。

    cargo は、RUSTFLAGS・Rust の版・依存の版・ビルドする対象の組み合わせが変わると、成果物をもう一組作り、
    前の組を消しません。そのため、ビルドを繰り返すと target/release が数 GB 単位で増えます。

    このスクリプトは、ビルドが成功したら、target/release/build の成果物（クレートとハッシュごとのフォルダ）のうち、
    次のどちらでもないものを消します。
    - cargo が「このビルドで使った」と出力したもの（--message-format の JSON）
    - target/release の実行ファイルと同じ大きさ・更新時刻の実行ファイルを持つもの（実行ファイルの元）

    消しすぎても、次のビルドで cargo が作り直すだけで、ビルドの結果は変わりません。
    cargo check --release など、ほかの release のビルドの成果物も消えるため、次に回すときは作り直しになります。
    ほかの cargo が release でビルドしている最中には動かさないでください。
.EXAMPLE
    pwsh scripts/build_release.ps1
#>

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot
try {
    $metadata = cargo metadata --format-version 1 --no-deps | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw "cargo metadata に失敗しました" }
    $unitRoot = [IO.Path]::GetFullPath((Join-Path $metadata.build_directory "release/build"))
    $outputDir = [IO.Path]::GetFullPath((Join-Path $metadata.target_directory "release"))

    # このビルドで使った成果物のフォルダ（クレート\ハッシュ）
    $used = @{}
    $executables = @()
    function Add-UsedPath([string]$path) {
        $full = [IO.Path]::GetFullPath($path)
        if ($full.StartsWith($unitRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
            $parts = $full.Substring($unitRoot.Length + 1).Split([IO.Path]::DirectorySeparatorChar)
            if ($parts.Count -ge 2) { $used[$parts[0] + "\" + $parts[1]] = $true }
        }
    }

    $previousRustFlags = $env:RUSTFLAGS
    # release.yml と同じ（C のランタイムを静的につなぐ）
    $env:RUSTFLAGS = "-C target-feature=+crt-static"
    try {
        # 人が読む出力は標準エラーに、JSON は標準出力に出る
        cargo build --release --locked -p redring -p job_manager_app -p job_runner --message-format=json-render-diagnostics |
            ForEach-Object {
                $message = $_ | ConvertFrom-Json
                if ($message.reason -eq "compiler-artifact") {
                    foreach ($file in $message.filenames) { Add-UsedPath $file }
                    if ($message.executable) { $executables += $message.executable }
                } elseif ($message.reason -eq "build-script-executed") {
                    Add-UsedPath $message.out_dir
                }
            }
        if ($LASTEXITCODE -ne 0) { throw "ビルドに失敗しました。古い成果物は消していません" }
    } finally {
        $env:RUSTFLAGS = $previousRustFlags
    }

    if (-not (Test-Path $unitRoot)) { return }
    if ($used.Count -eq 0) { throw "このビルドで使った成果物がわかりませんでした。古い成果物は消していません" }

    # 実行ファイルは target/release に置いた場所だけが出力されるため、元の実行ファイルを持つフォルダを探す
    foreach ($executable in $executables) {
        $placed = Get-Item $executable
        Get-ChildItem (Join-Path $unitRoot "*/*/out/*$($placed.Extension)") -File |
            Where-Object {
                $_.BaseName -eq ($placed.BaseName -replace "-", "_") -and
                $_.Length -eq $placed.Length -and
                $_.LastWriteTimeUtc -eq $placed.LastWriteTimeUtc
            } |
            ForEach-Object { Add-UsedPath $_.FullName }
    }

    $removedBytes = 0
    $removedCount = 0
    foreach ($crateDir in Get-ChildItem $unitRoot -Directory) {
        foreach ($unitDir in Get-ChildItem $crateDir.FullName -Directory) {
            if ($used.ContainsKey($crateDir.Name + "\" + $unitDir.Name)) { continue }
            $removedBytes += (Get-ChildItem $unitDir.FullName -Recurse -File | Measure-Object Length -Sum).Sum
            Remove-Item $unitDir.FullName -Recurse -Force
            $removedCount++
        }
        if (-not (Get-ChildItem $crateDir.FullName -Force)) { Remove-Item $crateDir.FullName -Force }
    }

    $sizeText = "{0:N0} MB" -f ($removedBytes / 1MB)
    Write-Host "使わなかった成果物を $removedCount 個（$sizeText）消しました。実行ファイル: $outputDir" -ForegroundColor Green
} finally {
    Pop-Location
}
