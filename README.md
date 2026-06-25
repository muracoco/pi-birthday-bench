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

CPUマルチスレッドで実行する場合:

```bash
cargo +stable-x86_64-pc-windows-gnu run --release -- --target 19930628 --max-digits 1000000 --backend cpu-multi --threads 12
```

進捗を抑制する場合:

```bash
cargo +stable-x86_64-pc-windows-gnu run --release -- --target 20240628 --max-digits 1000000 --chunk 100000 --backend cpu-single --no-progress
```

通常実行では、phaseとchunk単位の進捗をstderrに出します。`backend`、`target`、現在の `range`、`digits_computed`、`elapsed_seconds`、`digits_per_second`、`chunk`、`threads` を含みます。`--json` 指定時と `--no-progress` 指定時はprogressを出しません。

targetが見つかっても `--max-digits` まで走り切る場合:

```bash
cargo +stable-x86_64-pc-windows-gnu run --release -- --target 19930628 --max-digits 1000000 --backend cpu-single --benchmark-only
```

JSONだけを標準出力に出す場合:

```bash
cargo +stable-x86_64-pc-windows-gnu run --release -- --target 19930628 --max-digits 1000000 --backend cpu-single --json
```

`--json` 指定時、標準出力にはJSONだけを出します。進捗やphase表示は混ぜません。

生成したpi digitsのprefixと、`cpu-multi` では短い範囲の `cpu-single` 比較も確認する場合:

```bash
cargo +stable-x86_64-pc-windows-gnu run --release -- --target 19930628 --max-digits 1000000 --backend cpu-multi --threads 12 --verify --json
```

出力例:

```json
{
  "algorithm": "chudnovsky_binary_splitting",
  "backend": "cpu-single",
  "chunks_processed": 1,
  "cpu_model": "AMD Ryzen ...",
  "digits_computed": 1000000,
  "digits_per_second": 610604.1,
  "elapsed_seconds": 1.637722,
  "first_position": null,
  "found": false,
  "gpu_name": null,
  "gpu_role": "none",
  "logical_cpu_count": 16,
  "memory_total_mb": 32768,
  "memory_peak_mb": null,
  "physical_cpu_count": null,
  "target": "19930628",
  "threads": null,
  "verification_status": "skipped"
}
```

現時点では `threads` は `cpu-multi` の場合のみ数値になり、`cpu-single` では `null` です。`cpu_model`、`logical_cpu_count`、`physical_cpu_count`、`memory_total_mb`、`memory_peak_mb` は取得できない環境では `null` です。GPUは未実装のため `gpu_role` は `none` です。`--verify` 未指定時の `verification_status` は `skipped`、検証成功時は `passed` です。

利用可能なbackend一覧を確認する場合:

```bash
cargo +stable-x86_64-pc-windows-gnu run --release -- --list-backends
```

現時点で実行可能なのは `cpu-single` と `cpu-multi` です。GPU系backendは一覧に表示しますが、まだ実装されていません。

GPU系backendはstubとしてCLI上で選択できますが、現時点では明確なエラーを返します。通常ビルドではCUDA/HIP/OpenCL/Vulkan SDKを要求しません。

```bash
cargo +stable-x86_64-pc-windows-gnu run --release -- --target 19930628 --max-digits 1000000 --backend cuda-compute
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
- `cpu-single` / `cpu-multi` backendでのStart/Cancel
- GPU stub backendの選択と未実装エラー表示
- Benchmark only mode
- Verify
- status、phase、current range、elapsed seconds、digits/sec、progress barの表示
- resultの表示
- result text / JSON のコピー

GUIでまだできないこと:

- GUIでは `cpu-multi` のスレッド数を直接指定できず、論理CPU数が使われます
- 厳密なリアルタイム桁進捗は未対応
- `computing_pi` 中のキャンセルは、その計算フェーズ完了後に反映される場合があります

将来予定:

- GPU backend selector

## オプション

- `--target YYYYMMDD`: 探索対象。8桁の実在日付のみ有効です。
- `--max-digits N`: 最大探索桁数。必須です。
- `--chunk N`: 検索単位。既定値は `1000000` です。
- `--backend cpu-single|cpu-multi`: CPU backendを選択します。
- `--threads N`: `cpu-multi` 用のスレッド数です。未指定なら論理CPU数を使います。
- `--backend cuda-compute|cuda-search-only|hip|opencl|vulkan`: stubとして選択可能ですが、現時点では未実装エラーを返します。
- `--no-progress`: 進捗表示を抑制します。
- `--json`: 結果をJSONだけで標準出力に出します。
- `--benchmark-only`: targetが見つかっても `--max-digits` まで検索を続けます。
- `--verify`: 生成したpi digitsのprefixを検証します。`cpu-multi` では短い範囲を `cpu-single` と比較します。
- `--list-backends`: 利用可能なbackendと未実装backendの状態を表示します。`--target` と `--max-digits` は不要です。

## 注意

8桁の数字列は、かなり深い桁まで現れない場合があります。そのため `--max-digits` は必須です。指定した桁数まで見つからない場合、結果は `found: false` になります。
