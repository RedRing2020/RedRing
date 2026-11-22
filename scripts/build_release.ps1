<#
.SYNOPSIS
    リリースの配布物と同じ設定でビルドし、このビルドで使わなかった古い成果物を消す
.DESCRIPTION
    .github/workflows/release.yml と同じ対象（-p redring -p job_manager_app -p job_runner）と RUSTFLAGS で
    cargo build --release --locked を行います。cargo は、変わった部分だけを作り直します。
    RUSTFLAGS より先に使われる CARGO_ENCODED_RUSTFLAGS は、ビルドの間だけ外します。

    cargo は、RUSTFLAGS・Rust の版・依存の版・ビルドする対象の組み合わせが変わると、成果物をもう一組作り、
    前の組を消しません。そのため、ビルドを繰り返すと target/release が数 GB 単位で増えます。

    このスクリプトは、ビルドが成功したら、成果物のフォルダ（target/release/build。ビルドの対象の環境を
    設定で決めているときは target/<対象>/release/build も）にある、クレートとハッシュごとのフォルダのうち、
    次のどちらでもないものを消します。
    - cargo が「このビルドで使った」と出力したもの（--message-format の JSON）
    - 置いた実行ファイルと同じ大きさ・更新時刻の実行ファイルを持つもの（実行ファイルの元）

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
    # このビルドで使った成果物のフォルダ（クレートとハッシュごと）と、それを置く成果物のフォルダ
    $used = @{}
    $unitRoots = @{}
    $executables = @()
    function Add-UsedPath([string]$path) {
        $full = [IO.Path]::GetFullPath($path)
        if ($full -match '^(?<root>.+\\release\\build)\\(?<crate>[^\\]+)\\(?<hash>[^\\]+)(\\|$)') {
            $unitRoots[$Matches.root] = $true
            $used[$Matches.root + "\" + $Matches.crate + "\" + $Matches.hash] = $true
        }
    }

    $previousRustFlags = $env:RUSTFLAGS
    $previousEncodedRustFlags = $env:CARGO_ENCODED_RUSTFLAGS
    # release.yml と同じ（C のランタイムを静的につなぐ）
    $env:RUSTFLAGS = "-C target-feature=+crt-static"
    $env:CARGO_ENCODED_RUSTFLAGS = $null
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
        $env:CARGO_ENCODED_RUSTFLAGS = $previousEncodedRustFlags
    }

    if ($used.Count -eq 0) { throw "このビルドで使った成果物がわかりませんでした。古い成果物は消していません" }

    # 実行ファイルは置いた場所だけが出力されるため、元の実行ファイルを持つフォルダを探す
    foreach ($executable in $executables) {
        $placed = Get-Item $executable
        foreach ($unitRoot in @($unitRoots.Keys)) {
            Get-ChildItem (Join-Path $unitRoot "*/*/out/*$($placed.Extension)") -File |
                Where-Object {
                    $_.BaseName -eq ($placed.BaseName -replace "-", "_") -and
                    $_.Length -eq $placed.Length -and
                    $_.LastWriteTimeUtc -eq $placed.LastWriteTimeUtc
                } |
                ForEach-Object { Add-UsedPath $_.FullName }
        }
    }

    $removedBytes = 0
    $removedCount = 0
    foreach ($unitRoot in $unitRoots.Keys) {
        foreach ($crateDir in Get-ChildItem $unitRoot -Directory) {
            foreach ($unitDir in Get-ChildItem $crateDir.FullName -Directory) {
                if ($used.ContainsKey($unitRoot + "\" + $crateDir.Name + "\" + $unitDir.Name)) { continue }
                $removedBytes += (Get-ChildItem $unitDir.FullName -Recurse -File | Measure-Object Length -Sum).Sum
                Remove-Item $unitDir.FullName -Recurse -Force
                $removedCount++
            }
            if (-not (Get-ChildItem $crateDir.FullName -Force)) { Remove-Item $crateDir.FullName -Force }
        }
    }

    $sizeText = "{0:N0} MB" -f ($removedBytes / 1MB)
    $outputDirs = ($executables | ForEach-Object { Split-Path -Parent $_ } | Sort-Object -Unique) -join ", "
    Write-Host "使わなかった成果物を $removedCount 個（$sizeText）消しました。実行ファイル: $outputDirs" -ForegroundColor Green
} finally {
    Pop-Location
}
