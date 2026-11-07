# リリースの配布物の運用

最終更新: 2026-11-07

## 目的

利用者が受け取った実行ファイルが、RedRing のリポジトリのタグのソースから CI でビルドされたもので、途中で置き換えられていないことを確かめられるようにする。

## 配布物

リリースを公開すると、`.github/workflows/release.yml` が次のファイルを作り、リリースに添える。

| ファイル | 内容 |
|---|---|
| `redring-<タグ>-windows-x86_64.zip` | CAD/CAM のアプリの実行ファイル（`redring.exe`）、ライセンス、README |
| `redring-<タグ>-windows-x86_64.cdx.json` | SBOM（CycloneDX 1.5。アプリが使う依存のクレートの一覧。Windows の対象だけ） |
| `SHA256SUMS.txt` | 上の 2 つの SHA-256（小文字の 16 進数、LF の改行。`sha256sum -c` でも確かめられる） |

- zip と SBOM には、GitHub のビルドの出所の証明（artifact attestation。Sigstore で署名する）を付ける。どのリポジトリ・ワークフロー・タグのコミットでビルドしたかを、利用者が確かめられる
- チェックサムは、取ってくる途中で壊れていないことを確かめるためのもの。リリースの資産ごと置き換えられた場合はチェックサムも置き換えられるため、置き換えられていないことは出所の証明で確かめる
- ビルドは、ほかのワークフローのキャッシュを使わずに、`Cargo.lock` のとおり（`--locked`）に、固定した版のコンパイラで行う。コンパイラの版を上げるときは、ワークフローの `toolchain` を手で直す
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
