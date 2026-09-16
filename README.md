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

## ビルドと最初の実行

RustとGMPを使います。CLIだけならGUIやGPUのSDKは不要です。実行中に外部サイトへ通信することはなく、入力した日付を保存・送信しません。

### Windows

Windowsでは [MSYS2](https://www.msys2.org/) の **MINGW64** 環境とRust GNU toolchainを使います。PowerShellでRust GNU toolchainを用意します。

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu
```

次はMSYS2のMINGW64ターミナルで実行します。GMPの開発ファイルも必要です。MSYSやUCRT64用のパッケージを混ぜないでください。

```bash
pacman -S --needed mingw-w64-x86_64-gcc mingw-w64-x86_64-gmp mingw-w64-x86_64-pkgconf
```

リポジトリのフォルダへ移動して、環境を確認します。`gcc` と `pkg-config` は `/mingw64/bin/` のものを使います。`cargo` が見つからない場合は、WindowsでRustをインストールしたフォルダの `.cargo/bin` をこのターミナルのPATHへ追加してください。

```bash
echo "$MSYSTEM"
which gcc
which pkg-config
which cargo
cargo +stable-x86_64-pc-windows-gnu test --locked
cargo +stable-x86_64-pc-windows-gnu run --locked --release -- --target 20000101 --max-digits 1000 --verify --json
```

`MSYSTEM` は `MINGW64` になります。通常のPowerShellからGNU版の実行ファイルを起動すると、GMPなどのDLLがPATHにないため `STATUS_DLL_NOT_FOUND` になることがあります。ビルドと実行は同じMINGW64環境で行ってください。

### Linux

Ubuntuなどでは、RustとGMP開発パッケージを用意してCLIを使えます。

```bash
sudo apt-get install libgmp-dev
cargo test --locked
cargo run --locked --release -- --target 20000101 --max-digits 1000 --verify --json
```

`gmp-mpfr-sys` が要求するGMPとシステムのGMPが一致しない場合は、ビルドエラーに表示されるバージョンを確認してください。古いディストリビューションでの動作は保証していません。LinuxのGUIは現在の検証対象外です。

### 最初の結果

上の1,000桁の例では `verification_status: "passed"` を確認します。`found: false` と `first_position: null` は「指定した範囲では見つからなかった」という正常な結果です。`passed` は既知の先頭桁の照合などに通ったことを示し、全桁を独立した手法で証明した意味ではありません。

## CLI usage

100万桁まで検索する場合:

```bash
cargo +stable-x86_64-pc-windows-gnu run --release -- --target 20000101 --max-digits 1000000 --backend cpu-single
```

CPUマルチスレッドで実行する場合:

```bash
cargo +stable-x86_64-pc-windows-gnu run --release -- --target 20000101 --max-digits 1000000 --backend cpu-multi --threads 12
```

backend比較用に、targetが見つかっても `--max-digits` まで走り切る場合:

```bash
cargo +stable-x86_64-pc-windows-gnu run --release -- --target 20000101 --max-digits 1000000 --backend cpu-single --benchmark-only
cargo +stable-x86_64-pc-windows-gnu run --release -- --target 20000101 --max-digits 1000000 --backend cpu-multi --threads 12 --benchmark-only
```

進捗を抑制する場合:

```bash
cargo +stable-x86_64-pc-windows-gnu run --release -- --target 20000101 --max-digits 1000000 --chunk 100000 --backend cpu-single --no-progress
```

通常実行では、phaseとchunk単位の進捗をstderrに出します。`backend`、`target`、現在の `range`、`digits_computed`、`elapsed_seconds`、`digits_per_second`、`chunk`、`threads` を含みます。`--json` 指定時と `--no-progress` 指定時はprogressを出しません。

JSONだけを標準出力に出す場合:

```bash
cargo +stable-x86_64-pc-windows-gnu run --release -- --target 20000101 --max-digits 1000000 --backend cpu-single --json
```

`--json` 指定時、標準出力にはJSONだけを出します。進捗やphase表示は混ぜません。

比較時の条件、測定に含まれる処理、JSONの読み方は [ベンチマーク手順](docs/benchmarking.md) を参照してください。

生成したpi digitsのprefixと、`cpu-multi` では短い範囲の `cpu-single` 比較も確認する場合:

```bash
cargo +stable-x86_64-pc-windows-gnu run --release -- --target 20000101 --max-digits 1000000 --backend cpu-multi --threads 12 --verify --json
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
  "target": "20000101",
  "threads": null,
  "verification_status": "skipped"
}
```

現時点では `threads` は `cpu-multi` の場合のみ数値になり、`cpu-single` では `null` です。`cpu_model`、`logical_cpu_count`、`physical_cpu_count`、`memory_total_mb`、`memory_peak_mb` は取得できない環境では `null` です。GPUは未実装のため `gpu_role` は `none` です。`--verify` 未指定時の `verification_status` は `skipped`、検証成功時は `passed` です。

### Result fields

- `target`: CLIで指定した `YYYYMMDD` の探索対象です。
- `found`: `--max-digits` の範囲内でtargetが見つかったかどうかです。
- `first_position`: targetが最初に現れた小数部の1始まり位置です。整数部の `3` は数えません。未発見時は `null` です。
- `backend`: 実行に使ったbackendです。
- `algorithm`: pi計算アルゴリズムです。現時点では `chudnovsky_binary_splitting` です。
- `digits_computed`: 計算・検索対象にした小数部の桁数です。
- `elapsed_seconds`: job全体の実行秒数です。
- `digits_per_second`: `digits_computed / elapsed_seconds` で計算した処理速度です。benchmark-onlyでは最後まで走り切った全体速度になります。
- `chunks_processed`: 検索で処理したchunk数です。
- `threads`: `cpu-multi` で使ったスレッド数です。`cpu-single` では `null` です。
- `cpu_model` / `logical_cpu_count` / `physical_cpu_count`: 取得できたCPU情報です。取得できない値は `null` です。
- `gpu_name`: GPU backend用のフィールドです。現時点では `null` です。
- `gpu_role`: GPUの役割です。CPU backendでは `none` です。
- `memory_total_mb` / `memory_peak_mb`: 取得できたメモリ情報です。取得できない値は `null` です。
- `verification_status`: `skipped`、`passed`、`failed` のいずれかです。`--verify` 未指定時は `skipped` です。

利用可能なbackend一覧を確認する場合:

```bash
cargo +stable-x86_64-pc-windows-gnu run --release -- --list-backends
```

現時点で実行可能なのは `cpu-single` と `cpu-multi` です。GPU系backendは一覧に表示しますが、まだ実装されていません。

GPU系backendはstubとしてCLI上で選択できますが、現時点では明確なエラーを返します。通常ビルドではCUDA/HIP/OpenCL/Vulkan SDKを要求しません。

```bash
cargo +stable-x86_64-pc-windows-gnu run --release -- --target 20000101 --max-digits 1000000 --backend cuda-compute
```

## GPU acceleration scope

GPU accelerationは、どの処理をGPUに載せるかで意味が変わります。このプロジェクトでは結果の `gpu_role` で区別します。

| Mode | pi calculation | Pattern search | `gpu_role` | Status |
| --- | --- | --- | --- | --- |
| `cpu-single` | CPU single-thread | CPU | `none` | implemented |
| `cpu-multi` | CPU multi-thread | CPU | `none` | implemented |
| `cuda-search-only` | CPU | CUDA GPU | `search_only` | not implemented |
| `cuda-compute` | CUDA GPU | GPU or CPU | `pi_compute` | not implemented |
| `hip` / `opencl` / `vulkan` | undecided | undecided | `unavailable` | not implemented |

`cuda-search-only` は、pi digitsの計算自体をGPU化するmodeではありません。CPUでpiを計算し、検索だけをGPUへ渡す想定です。そのため、`cuda-search-only` の結果を `cpu-multi` と比較するときは、「pi計算を含む全体ベンチマーク」と「検索部分だけをGPU化した試作」を同じ意味の高速化として扱わないでください。

`cuda-compute` はpi計算本体のGPU化を表す予定ですが、現時点では未実装です。多倍長整数演算、binary splitting、メモリ転送、CPU/GMP実装との差分が大きいため、安定版の完了条件には含めません。

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

## ライセンス

本リポジトリのコードは [MIT License](LICENSE) で公開しています。GMPやRustクレートなどの依存ライブラリには、それぞれのライセンスが適用されます。
