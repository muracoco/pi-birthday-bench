# pi-birthday-bench 仕様書・実装計画

## 1. プロジェクト概要

`pi-birthday-bench` は、指定した `YYYYMMDD` 形式の8桁数字列が、円周率 pi の小数部に最初に現れる桁位置を探索する CLI ベンチマークソフトである。

円周率の桁列は、既存ファイル、Web API、事前保存データを使わず、実行時に自力計算する。

将来的に比較対象とする実行モード:

- `cpu-single`: CPU シングルスレッド
- `cpu-multi`: CPU マルチスレッド
- `cuda-compute`: CUDA で pi 計算本体も含めて GPU 利用
- `cuda-search-only`: CPU で pi を計算し、GPU で検索のみ実行
- `hip`: AMD GPU 向け候補
- `opencl`: AMD/NVIDIA/Intel 横断候補
- `vulkan`: Vulkan Compute 候補
- `auto`: 利用可能な最適バックエンドを自動選択

初期バージョンでは CPU single / CPU multi を必須実装とし、GPU 系はモードセレクター、feature flag、未実装時エラー、README 上の拡張方針までを先に作る。

## 2. 重要な設計方針

### 2.1 桁位置の定義

検索対象は小数部のみ。整数部 `3` は検索対象に含めない。

```text
pi = 3.1415926535...
小数第1位 = 1桁目 = 1
小数第2位 = 2桁目 = 4
小数第3位 = 3桁目 = 1
```

### 2.2 target の形式

CLI で指定する探索対象は `YYYYMMDD` の8桁とする。

有効例:

- `20000229`
- `20240628`

無効例:

- `19930229`
- `19000229`
- `0628`
- `abc`

### 2.3 MMDD の既知位置テスト

4桁パターンは CLI 入力では無効だが、内部の汎用検索関数のテストに使う。

- `0628`: 小数第71桁
- `0812`: 小数第146桁
- `1027`: 小数第163桁
- `1117`: 小数第153桁
- `1105`: 小数第174桁

## 3. CLI 仕様

```bash
pi-birthday-bench --target 19930628 --max-digits 200000000 --chunk 1000000 --backend cpu-single
pi-birthday-bench --target 19930628 --max-digits 200000000 --chunk 1000000 --backend cpu-multi --threads 12
pi-birthday-bench --target 19930628 --max-digits 200000000 --chunk 1000000 --backend cuda-search-only
```

将来オプション:

- `--target YYYYMMDD`
- `--max-digits N`
- `--chunk N`
- `--backend MODE`
- `--threads N`
- `--json`
- `--no-progress`
- `--verify`
- `--benchmark-only`
- `--list-backends`

## 4. 出力仕様

通常出力:

```text
target: 19930628
found: true
first_position: 12345678
backend: cpu-multi
algorithm: chudnovsky_binary_splitting
digits_computed: 20000000
elapsed_seconds: 12.34
digits_per_second: 1620745.5
chunks_processed: 20
threads: 12
cpu_model: AMD Ryzen ...
gpu_name: null
gpu_role: none
memory_peak_mb: 512
verification_status: passed
```

GPU バックエンドは必ず `gpu_role` を表示する。

- `none`
- `search_only`
- `pi_compute`
- `unavailable`

## 5. バックエンド設計

Rust では次のような trait を想定する。

```rust
pub trait PiBackend {
    fn name(&self) -> &'static str;
    fn gpu_role(&self) -> GpuRole;
    fn is_available(&self) -> bool;
    fn compute_digits(&self, digits: usize) -> Result<String>;
}
```

初期段階では GPU 系は未実装でよい。ただし CLI 上は選択可能にし、明確な unavailable / not implemented エラーを返す。

## 6. pi 計算仕様

- Chudnovsky algorithm
- binary splitting
- `rug` / GMP 系の多倍長整数
- guard digits は20桁以上
- 検索には requested digits までを使う

chunk 境界をまたぐ一致を見逃さないため、直前 chunk の末尾 `target.len() - 1` 桁を保持する。

## 7. ベンチマーク仕様

通常探索モードでは target が見つかった時点で停止する。

`--benchmark-only` では target が見つかっても `--max-digits` まで計算・検索を続ける。

## GitHub Milestones

- v0.1: CPU single MVP
- v0.2: CPU multi backend
- v0.3: Benchmark framework
- v0.4: Backend selector and GPU stubs
- v0.5: CUDA search-only prototype
- v0.6: CUDA compute / AMD research branch

## v0.1 初回実装範囲

- Create Rust CLI project skeleton
- Implement YYYYMMDD date validation
- Implement CPU single pi calculation
- Implement pattern search with chunk overlap
- Connect CPU single backend to CLI search
- Add README basic documentation

対象外:

- GPU 対応
- CPU multi
- JSON 出力
- benchmark-only
