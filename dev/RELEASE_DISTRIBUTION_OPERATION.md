# リリースの配布物の運用

最終更新: 2026-11-21

## 目的

利用者が受け取った実行ファイルが、RedRing のリポジトリのタグのソースから CI でビルドされたもので、途中で置き換えられていないことを確かめられるようにする。

## 配布物

リリースを公開すると、`.github/workflows/release.yml` が次のファイルを作り、リリースに添える。

| ファイル | 内容 |
|---|---|
| `redring-<タグ>-windows-x86_64.zip` | CAD/CAM のアプリ（`redring.exe`）・ジョブマネージャー（`redring-job-manager.exe`）・ランナー（`redring-batch-runner.exe`）の実行ファイル、ライセンス、README。3 つの実行ファイルは同じフォルダに置く（アプリは同じフォルダのジョブマネージャーを、ジョブマネージャーは同じフォルダのランナーを起動する） |
| `redring-<タグ>-windows-x86_64.cdx.json` | SBOM（CycloneDX 1.5。アプリが使う依存のクレートの一覧。Windows の対象だけ。ランナーの依存は、アプリの依存に含まれる） |
| `redring-<タグ>-windows-x86_64-job-manager.cdx.json` | ジョブマネージャー（画面）の SBOM |
| `SHA256SUMS.txt` | 上の 3 つの SHA-256（小文字の 16 進数、LF の改行。`sha256sum -c` でも確かめられる） |

- zip と SBOM には、GitHub のビルドの出所の証明（artifact attestation。Sigstore で署名する）を付ける。どのリポジトリ・ワークフロー・タグのコミットでビルドしたかを、利用者が確かめられる
- チェックサムは、取ってくる途中で壊れていないことを確かめるためのもの。リリースの資産ごと置き換えられた場合はチェックサムも置き換えられるため、置き換えられていないことは出所の証明で確かめる
- ビルドは、ほかのワークフローのキャッシュを使わずに、`Cargo.lock` のとおり（`--locked`）に、固定した版のコンパイラ（`rust-toolchain.toml`。CI と同じ版）で行う。コンパイラの版を上げる手順は `dev/DEPENDENCY_AUDIT_OPERATION.md`
- C のランタイムを静的につなぐ（`+crt-static`）。Visual C++ の再頒布可能パッケージのない PC でも起動できる
- タグと、アプリのクレート（`view/app/Cargo.toml` の `redring` の `version`）の版が違うときは、ビルドの前に失敗にする
- 実行ファイルのコード署名（Authenticode）は、まだ行わない。そのため Windows の SmartScreen の警告が出る（マニュアルの既知の制約に書く）

## リリースの作り方

1. `main` にリリースの変更（`view/app/Cargo.toml` の `version`・`CHANGELOG.md`・リリースノート）をマージする
2. `main` にタグを付けて push する（例: `git tag v0.1.3` → `git push origin v0.1.3`）
3. GitHub のリリースを、そのタグでプレリリースとして公開する（本文はリリースノート）
4. 公開すると `Release` のワークフローが回り、配布物を添える。終わったら、リリースに 3 つのファイルがあることを確かめる
5. 下の「確かめ方」で、zip のチェックサムと出所の証明を確かめる

ワークフローが失敗した場合:

- 原因を直してから、Actions の `Release` を手で回す（`workflow_dispatch`）。そのとき、ブランチではなくリリースのタグを選ぶ（出所の証明がタグのコミットを指すように、ブランチから回したときは何もしない）
- 同じ名前のファイルがリリースにあると、置き換えずに失敗にする（公開した配布物を黙って置き換えないため）。途中まで添えたファイルは、先にリリースから消す
- ワークフローやビルドの直しがタグのソースに要る場合は、版を上げて新しいタグで出し直す

## 手元でリリースビルドを作る

配布物と同じ設定の実行ファイルを手元で作るときは、`cargo build --release` ではなく次のスクリプトを使う。

```powershell
pwsh scripts/build_release.ps1
```

- `release.yml` と同じ対象と RUSTFLAGS（`+crt-static`）でビルドする。cargo は変わった部分だけを作り直す。実行ファイルは `target/release` にできる
- ビルドが成功したら、`target/release/build` のうち、このビルドで使わなかった成果物（クレートとハッシュごとのフォルダ）を消す
- cargo は、RUSTFLAGS・Rust の版・依存の版・ビルドする対象の組み合わせが変わると、成果物をもう一組作り、前の組を消さない。そのままビルドを繰り返すと、`target/release` が 1 組あたり 1 GB ほどずつ増える
- `cargo check --release` など、ほかの release のビルドの成果物も消すため、次に回すときは作り直しになる
- ほかの cargo が release でビルドしている最中には動かさない
- 外部の道具（`cargo-sweep`）は使わない。保守されておらず、cargo の成果物の置き方（クレートとハッシュごとのフォルダ）に合うかを確かめられないため

## 確かめ方

利用者向けの手順は、マニュアルの「動かす」にも書く。

チェックサム（PowerShell。`True` と出れば同じ。`-eq` は大文字・小文字を区別しない）:

```powershell
(Get-FileHash .\redring-v0.1.3-windows-x86_64.zip -Algorithm SHA256).Hash -eq '<SHA256SUMS.txt の値>'
```

出所の証明（GitHub CLI）:

```powershell
gh attestation verify .\redring-v0.1.3-windows-x86_64.zip --repo RedRing2020/RedRing
```

## ワークフローの決まり

- 書き込みの権限（`contents: write`・`id-token: write`・`attestations: write`）は、配布物を作るジョブだけに与える
- Action はコミットの SHA で固定し、ツール（`cargo-cyclonedx`）は版を決めて `--locked` で入れる（`DEPENDENCY_AUDIT_OPERATION.md` の GitHub Actions の決まり）
- タグなどの外からの値は、`run` の中に `${{ }}` で埋め込まず、環境変数で渡す
