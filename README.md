# pi-birthday-bench

`pi-birthday-bench` は、指定した `YYYYMMDD` 形式の8桁数字列が、円周率 pi の小数部に最初に現れる桁位置を探索する CLI です。

このツールは pi の既存テキストファイル、Web API、事前保存データを使いません。実行時に Chudnovsky algorithm と binary splitting で pi を計算します。

## 桁位置の定義

検索対象は小数部のみです。整数部の `3` は含めません。

```text
pi = 3.1415926535...
        ^ 小数第1位
```

つまり、小数第1位は `1`、小数第2位は `4`、小数第3位は `1` です。出力の `first_position` は、この小数部を1始まりで数えた位置です。

## CLI usage

Windows で `rug` / GMP 系依存を使うため、現時点では MSYS2 MinGW と Rust GNU toolchain でのビルドを前提にしています。

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu
```

```bash
cargo +stable-x86_64-pc-windows-gnu run --release -- --target 19930628 --max-digits 1000000 --backend cpu-single
```

進捗を抑制する場合:

```bash
cargo +stable-x86_64-pc-windows-gnu run --release -- --target 20240628 --max-digits 1000000 --chunk 100000 --backend cpu-single --no-progress
```

## GUI usage

GUIは `eframe` / `egui` を使うRust-native GUIです。CLIをサブプロセス起動せず、CLIと同じ中核ジョブ処理を呼びます。
GUI binaryは `gui` feature 有効時のみビルドされます。Windowsでは `wgpu` のDX12 backendを明示して、OpenGLが使えない環境でも起動できる構成にしています。

```bash
cargo +stable-x86_64-pc-windows-gnu run --release --features gui --bin gui
```

GUIでできること:

- `YYYYMMDD` targetの入力とvalidation
- `max_digits` と `chunk` の入力
- `cpu-single` backendでのStart/Cancel
- status、phase、elapsed seconds、digits/sec、progress barの表示
- resultの表示
- result text / JSON のコピー

GUIでまだできないこと:

- backendは `cpu-single` のみ
- 厳密なリアルタイム桁進捗は未対応
- `computing_pi` 中のキャンセルは、その計算フェーズ完了後に反映される場合があります
- `benchmark-only` と `verify` は未実装

将来予定:

- `cpu-multi`
- `benchmark-only`
- CLI `--json`
- GPU backend selector

## オプション

- `--target YYYYMMDD`: 探索対象。8桁の実在日付のみ有効です。
- `--max-digits N`: 最大探索桁数。必須です。
- `--chunk N`: 検索単位。既定値は `1000000` です。
- `--backend cpu-single`: v0.1 では `cpu-single` のみ対応します。
- `--no-progress`: 進捗表示を抑制します。

## 注意

8桁の数字列は、かなり深い桁まで現れない場合があります。そのため `--max-digits` は必須です。指定した桁数まで見つからない場合、結果は `found: false` になります。
