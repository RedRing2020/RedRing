# STEP の試験のファイル

STEP の読み込みの試験（設計: `dev/architecture/BREP_AND_STEP_IMPORT_DESIGN.md` 11 章）で読むファイルです。

## 適合の試験のファイル

形を式で決めた部品を、RedRing で書いた STEP（AP214）です。各ファイルの `FILE_DESCRIPTION` に、部品の形と寸法を書いています。単位は、`cone_inch.step` だけがインチと度で、ほかは mm とラジアンです。

| ファイル | 部品 | B-rep の面の数 |
|---|---|---|
| `block.step` | 直方体 40 × 30 × 20 mm | 6 |
| `cylinder.step` | 円柱 半径 10 mm・高さ 20 mm（側面は継ぎ目なし。2 つのループ） | 3 |
| `cylinder_seam.step` | 同じ円柱（側面は継ぎ目の稜線を行きと帰りに使う 1 つのループ） | 3 |
| `cylinder_far.step` | 同じ円柱を (1e6, -2e6, 5e5) mm に置いたもの | 3 |
| `cone.step` | 円錐 底の半径 10 mm・高さ 15 mm（側面は頂点まで） | 2 |
| `cone_inch.step` | 円錐 底の半径 1 インチ・半頂角 30 度 | 2 |
| `sphere.step` | 球 半径 10 mm（赤道で分けた 2 つの半球） | 2 |
| `sphere_seam.step` | 同じ球（継ぎ目の稜線だけで囲んだ 1 つの面） | 1 |
| `torus.step` | トーラス 中心の円の半径 30 mm・管の半径 8 mm（上と下の円で分けた外側と内側） | 2 |
| `torus_seam.step` | 同じトーラス（2 本の継ぎ目の稜線で囲んだ 1 つの面） | 1 |
| `plate_with_holes.step` | 板 100 × 60 × 10 mm に、半径 6 mm の穴を 6 つ | 12 |
| `block_with_void.step` | 直方体 40 × 40 × 40 mm に、半径 10 mm の球の空洞（`BREP_WITH_VOIDS`） | 8 |

## 大きなファイルの試験のファイル

公開されている試験用のファイルは、リポジトリに置きません。`large_files.txt` に、読むファイルの SHA-256 と、取得したフォルダからのパスを置きます。

1. NIST（米国国立標準技術研究所）の「MBE PMI Validation and Conformance Testing」の試験用の CAD モデルを取得する
   - 取得のページ: https://www.nist.gov/ctl/smart-connected-systems-division/smart-connected-manufacturing-systems-group/mbe-pmi-0
   - ファイル: `NIST-FTC-CTC-PMI-CAD-models.zip`（60,294,624 バイト）
   - zip の SHA-256: `17e14dbf207f447f16e0182a7eda5254ee19afff591dd409b8b510103dd05d10`
2. zip を展開する。`NIST-FTC-CTC-PMI-CAD-models` のフォルダの下に、`CTC Definitions` と `FTC Definitions` のフォルダができる
3. 展開した `NIST-FTC-CTC-PMI-CAD-models` のフォルダを、環境の変数 `REDRING_STEP_LARGE_FILES` に渡して試験を実行する

```powershell
$env:REDRING_STEP_LARGE_FILES = "C:\path\to\NIST-FTC-CTC-PMI-CAD-models"
cargo test --release -p application --lib large_files -- --nocapture
```

試験は、ファイルの SHA-256 が一覧と合うことを確かめてから読み、読めた件数・外した要素の数・三角形にできなかった面の数を出します。環境の変数がなければ、何も読まずに終わります（CI では読みません）。
