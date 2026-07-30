# Objective

既存のCLI・GUI・計算結果・出力契約を変えずに、`pi-birthday-bench` の変更容易性と検証可能性を高める。対象は、現在残っている小規模な責務混在、公開境界の曖昧さ、CLI/GUI間の入力差、キャンセルと進捗のテスト不足、Windows GNU実行環境の再現性である。

全面書き換えや将来GPU実装の先取りは行わない。各変更は、根拠となる現行挙動をテストで固定してから、小さく戻しやすい単位で実施する。

# Project Understanding

## Product

- 指定された8桁の `YYYYMMDD` が、円周率の小数部に最初に現れる1始まり位置を探索するRustアプリケーション。
- 保存済みpiデータ、DB、Web APIは使わず、Chudnovsky法 + binary splittingで実行時に小数桁を生成する。
- CLIが主入口。`gui` featureでeframe/egui製GUIも提供し、サブプロセスではなく同じ `run_job` を直接呼ぶ。
- CPU singleとCPU multiは実装済み。CUDA/HIP/OpenCL/Vulkanは選択可能なstubで、通常ビルドにSDKを要求しない。

## Entry Points and Workflows

- CLI: `src/main.rs`
  - clapで入力を受け、`RunConfig`へ変換する。
  - `--list-backends` は計算せず一覧を表示する。
  - 通常時はstderrへ進捗、stdoutへtext結果を出す。
  - `--json` はstdoutへJSONだけを出し、進捗を抑制する。
- GUI: `src/bin/gui.rs` (`--features gui --bin gui`)
  - 入力検証後、worker threadで `run_job` を実行する。
  - channelで `ProgressEvent` を受信し、キャンセルflagを共有する。
  - 結果をtext/JSONでコピーできる。
- 共通ジョブ: `src/job.rs::run_job`
  - default thread数設定、検証、backend dispatch、pi生成、任意検証、検索、system info収集、結果構築、event発行を行う。

## Major Modules and Responsibilities

- `src/result.rs`: `RunConfig`、`BackendMode`、`BenchmarkResult`、進捗event/phase、text/JSON出力契約。
- `src/backend.rs`: backend metadata、availability、stub error、`PiBackend`、CPU backend実装。
- `src/pi.rs`: single/multiのpi小数桁生成。`rug`と`rayon`を使用する計算核。
- `src/search.rs`: chunk境界をまたぐpattern探索、benchmark-only、cancel、progress callback。
- `src/date.rs`: `YYYYMMDD` の暦日妥当性検証。
- `src/system_info.rs`: 環境変数と標準APIから任意のsystem infoを収集。
- `src/job.rs`: 上記を結ぶapplication orchestration。
- `tests/cli_*.rs`: backend一覧/stub、JSON契約、CPU backend同値性、進捗抑制をCLI境界で検証。

## Data Flow

1. CLIまたはGUI入力を `RunConfig` に変換する。
2. `run_job` がCPU multiの省略thread数を補完し、configを検証する。
3. `BackendMode` に応じた `PiBackend` が `max_digits` 桁を一括生成する。
4. `--verify` 時は既知prefixを照合し、CPU multiは先頭最大1000桁をsingle結果と比較する。
5. `search_pattern_with_options` が生成済み文字列をchunk単位で探索する。
6. system infoと計測値を `BenchmarkResult` にまとめ、textまたはJSONへ変換する。

注意: `chunk` は現在、piを分割生成する単位ではなく、生成済みdigitsを検索・進捗報告する単位である。計算中キャンセルはpi全桁生成完了まで反映されない。

## External Dependencies and Operational Boundaries

- `rug` / `gmp-mpfr-sys` とsystem GMP/MPFR/MPC。WindowsはMSYS2 MinGW + GNU Rust toolchainが前提。
- `rayon`: CPU multiの専用thread poolとbinary splitting並列化。
- `clap`: CLI契約。
- `serde_json`: JSON出力。
- optional `eframe` / `wgpu(dx12)`: Windows GUI。
- DB、migration、認証、課金、通知、外部API、queue、永続storageは存在しない。
- CIはUbuntu stable testと、MSYS2上のWindows GNU test + GUI release buildを実行する。

# Behaviors To Preserve

- targetは実在する `YYYYMMDD` 8桁のみ。2月29日を含む暦日判定を維持する。
- 探索対象はpiの小数部のみで、`first_position` は1始まり。整数部の `3` は数えない。
- 最初の一致を返し、chunk境界をまたぐ一致を取りこぼさない。
- 通常探索は発見後に停止し、`--benchmark-only` は `max_digits` まで検索を継続する。
- `chunks_processed`、progress range、phase順序、`digits_computed` の現行定義を維持する。
- CPU single/multiで同じdigitsと検索結果を返す。`--threads 1` もsingleと一致する。
- CPU multiで `--threads` 省略時は利用可能な論理並列数（取得失敗時1）を使用する。
- `--verify` 未指定は `skipped`、成功は `passed`。失敗を成功結果として返さない。
- `--json` のstdoutはJSONのみで、stderr進捗は空。`--no-progress` も進捗を抑制する。
- JSON field名、型、nullability、文字列値を維持する。特にbackend、algorithm、threads、gpu_role、verification_status。
- text outputのfield、順序、表現を意図せず変更しない。
- `--list-backends` はtarget不要で、CPU single/multiをavailable、GPU系をfeature依存のunavailableとして表示する。
- GPU stubはSDKなしで通常ビルドでき、選択時は計算せず明確な非ゼロ終了を返す。
- CLIとGUIは同じ `run_job` を使う。GUIは応答を止めず、結果のtext/JSONコピーを維持する。
- system infoが取得できない場合は失敗せず `null` を返す。

# Non-Negotiables

- 最初に `git status --short --branch` を確認する。
- 既存の未コミット変更と自分の変更を混ぜない。開始時点では `src/date.rs` にユーザー所有の変更、`refactor-instructions.md` に本指示書がある可能性がある。
- `src/date.rs` の `%` から `is_multiple_of` への既存差分を、明示的承認なしに変更・整形・revertしない。
- 編集前にbaselineコマンドと結果を記録する。
- 変更は小さく戻しやすい単位にする。各phaseの完了時に検証する。
- 無関係な整形、命名変更、依存更新、ついでのリファクタリングをしない。
- 既存挙動を勝手に変えない。テストがない契約はREADMEと現行コードからcharacterization testを先に作る。
- unsafe、async runtime、汎用plugin framework、大規模DIを追加しない。
- native依存やGPU SDKを増やさない。`Cargo.lock` は依存変更が承認された場合だけ変更する。
- PowerShell 5.1ではAGENTS.mdのUTF-8 wrapperを全コマンドに使う。ファイル書込時はUTF-8を明示する。
- 正しさが不明なら実装を止めて質問する。

# Stop And Ask Conditions

以下は承認なしに決めない。

- crateのpublic API (`pub` module/type/function/variant/field) を削除、rename、非公開化、または意味変更する必要がある。
- CLI option、既定値、help、exit status、stdout/stderr分離を変える必要がある。
- JSON/text schema、field順序、nullability、列挙文字列、時刻・速度・chunkの意味を変える必要がある。
- pi algorithm、precision、guard digits、丸め、全桁一括生成方針を変える。
- CPU multiの並列化戦略や結果決定性を変える。
- 計算中キャンセルを細粒度化するため、計算APIまたは性能特性を大きく変える。
- GPU backendを実装する、SDKを検出する、feature有効時のbuild/runtime契約を変える。
- GUIにthreads入力を追加するなど、CLI/GUIの機能差をどちらの仕様へ揃えるかプロダクト判断が必要になる。
- `is_multiple_of` を維持できる最低Rust版が不明なままMSRV互換性へ影響する変更を行う。
- 削除候補のpublic code、feature、script、docs項目が外部利用されていないと証明できない。
- README、テスト、実装の期待が矛盾し、正しい挙動を特定できない。

# Baseline Commands

PowerShellでは各コマンドをAGENTS.md記載のUTF-8 wrapper内で実行する。

```powershell
git status --short --branch
git diff -- src/date.rs refactor-instructions.md
git log --oneline -8
rustc --version
cargo --version
cargo +stable-x86_64-pc-windows-gnu fmt -- --check
cargo +stable-x86_64-pc-windows-gnu test
cargo +stable-x86_64-pc-windows-gnu clippy --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-gnu build --release --features gui --bin gui
git diff --check
```

既知の2026-07-11 baseline:

- rustc/cargo 1.96.0。
- fmt、clippy、GUI release build、`git diff --check` は成功。
- 通常PowerShellからのGNU test binary起動は `0xc0000135 STATUS_DLL_NOT_FOUND`。これはassertion failureではなく、GMP系DLLを含むMSYS2 runtime PATH不足。MSYS2 shellまたはCI同等PATHでtestを再実行すること。

# Debt Map

## D1. `run_job` の設定正規化とdispatchが列挙matchに固定されている

- 根拠: `src/job.rs:24-57` はCPU multiのdefault threads補完と全backend matchを持つ。一方、metadata/availabilityは `src/backend.rs:82-151` にある。
- 理由: backend事実の所有者が二箇所に分かれ、新backend追加時にmetadataと実行dispatchがずれる。
- 影響: backend追加、threads validation、stub error。
- リスク: 中。core execution path。
- 改善案: まず現行dispatchのcharacterization testを追加する。その後、backend生成/実行準備をbackend moduleの最小関数へ寄せられるか検討する。trait objectやfactory frameworkは不要。`RunConfig`正規化の責務を明示し、同一値を再代入するだけの処理を整理する。
- 検証: CPU single/multi JSON同値、default/1/2 threads、GPU stub、event順序。
- 実装判断: 今実装可。ただし公開API変更なし、小さくできる場合のみ。差分が増えるなら提案止まり。

## D2. `PiBackend` の公開契約が実利用より広く曖昧

- 根拠: `src/backend.rs:16-21` のtraitはpublicで、`is_available` は `debug_assert!` のみ、`name`/`gpu_role` は結果構築に使う。`BackendInfo`にも同じmetadataがある。
- 理由: runtime backend objectとmetadata registryのどちらが正なのか曖昧で、`is_available` はrelease時の保証にならない。
- 影響: 外部crate利用者、backend拡張、結果metadata。
- リスク: 中〜高。public APIのため削除/非公開化は互換性判断が必要。
- 改善案: 現行利用をテストし、内部重複を減らせる最小案を提示する。public visibility変更は承認まで実装しない。release correctnessをdebug assertionに依存させない。
- 検証: backend list、CPU結果のbackend/gpu_role、全feature組合せのcheck。
- 実装判断: テストと内部整理のみ可。public contract変更は提案のみ。

## D3. 計算が全桁一括で、cancel/progressが計算中に効かない

- 根拠: `src/job.rs:73-89` は `compute_digits(max_digits)` 完了後に最初のcancel確認。GUIもその制約を表示する (`src/bin/gui.rs:225-228`)。
- 理由: 長時間計算ではGUI cancelが遅延し、進捗も計算完了まで実質更新できない。
- 影響: 大きい `max_digits` のUX、memory peak、worker終了。
- リスク: 高。piアルゴリズム、性能、結果正確性へ直結。
- 改善案: 今回は挙動をcharacterization test/文書で固定し、改善設計案（cooperative cancellation可能な計算境界、測定方法、性能回帰基準）だけを作る。計算核は変更しない。
- 検証: 計算前/計算後/検索中cancel event、成功時phase順序。
- 実装判断: テスト追加は可。計算方式変更は提案のみ。

## D4. event契約と失敗経路のテストが不足

- 根拠: `src/job.rs` はStarted/PhaseChanged/Progress/Completed/Cancelledを発行するが、unit testsは主にsearch/verifyを直接検証する。GUIがFailedへ変換する処理は `src/bin/gui.rs:269-277` にだけある。
- 理由: refactorでevent順序、二重通知、cancel時のerror扱いが壊れてもCLI testsでは検知しにくい。
- 影響: CLI進捗、GUI status、cancel UX。
- リスク: 低〜中。
- 改善案: 小桁の `run_job` event列を直接検証するテストを追加する。時刻値そのものは固定せず、phase順序、range、terminal eventの一意性を検証する。
- 検証: success、事前cancel、検索中cancel、invalid config、unsupported backend。
- 実装判断: 今実装可。production変更前の最優先安全網。

## D5. CLIとGUIの入力能力が非対称

- 根拠: CLIは `--threads` と `--no-progress` を持つ。GUIの `RunConfig` は `threads: None` 固定 (`src/bin/gui.rs:353-372`)。GUI backend一覧は共通metadataを使うが、CPU multi thread数を指定できない。
- 理由: 同じcoreを使う二入口で設定可能範囲が異なるが、それが意図的制約か未完成かコードだけでは決められない。
- 影響: GUI benchmark再現性、README、support。
- リスク: 中。UX判断が必要。
- 改善案: 現状差を明記し、共通の純粋なconfig validation helperで重複だけ減らせるか検討する。threads UI追加は承認までしない。
- 検証: GUI build、RunConfig validation unit tests、手動GUI smoke。
- 実装判断: validation整理のみ可。機能追加は提案のみ。

## D6. `BenchmarkResult` の手書きtext/JSON契約が変更に弱い

- 根拠: `src/result.rs` がfieldを持ち、`as_text` と `as_json` で別々に列挙する。integration testsはJSONの一部fieldを検証するが、完全なkey集合やtext全体契約は固定していない。
- 理由: field追加/rename時に片方だけ更新される可能性がある。
- 影響: automation consumers、README例、GUI copy。
- リスク: 高。公開出力schema。
- 改善案: 実装を変える前に、JSON key集合・型/nullabilityとtext field集合/順序をcharacterization testで固定する。serde derive導入やfield順変更は必要性と互換性を確認してから。
- 検証: exact key set、代表値、parse可能性、stdout purity、text snapshot相当の明示assert。
- 実装判断: テスト追加は可。serialization方式変更は提案または承認後。

## D7. platform/system infoは環境依存で、テスト可能性が低い

- 根拠: `src/system_info.rs` は環境変数とOS APIを直接読む。結果はoptionalで、CLI testsはfield存在だけを見る。
- 理由: CI/端末差が大きく、parse edge caseや単位変換の回帰を検出しにくい。
- 影響: JSON metadataの信頼性。ただし探索結果には非影響。
- リスク: 低。
- 改善案: 純粋なparse helperが既にある範囲をunit testで補強し、OS adapterの抽象化は必要最小限にする。新crateは追加しない。
- 検証: missing/invalid/valid env、null fallback、単位。
- 実装判断: 今実装可。ただし大規模platform abstractionは不要。

## D8. Windows GNU検証がshell/PATHに依存する

- 根拠: READMEとCIはMSYS2を要求。2026-07-11にはcompile/link済みtest binaryが通常PowerShellで `STATUS_DLL_NOT_FOUND`。`scripts/bootstrap-github.ps1` はGitHub初期化用でbuild環境bootstrapではない。
- 理由: compile成功と実行可能性が分離し、担当者が環境エラーをcode failureと誤認しやすい。
- 影響: local verification、onboarding。
- リスク: 低。コード変更で解決しようとすると高。
- 改善案: READMEへCI同等のMSYS2 test手順と必要PATH確認を必要最小限追記する。native libraryのコピーや新bootstrap scriptは、再現性を検証できない限り追加しない。
- 検証: MSYS2 shellでfull test、`where.exe`/`which`でDLL/tool確認、CI。
- 実装判断: docs改善は可。dependency packaging変更は提案のみ。

## D9. 性能回帰を検知する自動基準がない

- 根拠: benchmark-onlyと速度fieldはあるが、CIはtest/buildのみ。pi unit testsは正確性中心。
- 理由: refactorがCPU single/multi性能やmemoryを悪化させても機能testは通る。
- 影響: 本製品のbenchmark用途。
- リスク: 中。CI時間・host varianceが大きい。
- 改善案: 今回は固定速度thresholdをCIへ入れない。代表入力、release build、複数回測定、中央値、許容差を含む手動比較手順を報告する。
- 検証: 同一host/toolchainでbefore/after比較。結果位置とdigitsも同時確認。
- 実装判断: 測定手順のみ。CI benchmark導入は提案のみ。

## D10. 明白な死コードは確認できないが、公開範囲が広い

- 根拠: `src/lib.rs` は全moduleをpublic公開し、主要型/関数もpublic。現行CLI/GUI内で使われないものでも外部利用の有無はrepoだけでは証明できない。
- 理由: dead-code lintで消せない公開項目を「未使用」と誤認しやすい。
- 影響: downstream compatibility。
- リスク: 高。
- 改善案: 削除しない。公開API inventoryを作り、内部利用・docs・testsを記録する。非公開化は利用方針確認後の別作業。
- 検証: `cargo doc --no-deps`、repo内参照、必要ならdownstream確認。
- 実装判断: inventoryのみ。削除は提案止まり。

# Implementation Phases

## Phase 0: Baseline and scope lock

- git状態と既存差分を記録する。
- baselineを実行する。DLL不足は正確なerrorとして記録し、MSYS2で再試行する。
- 本文のDebt Mapを現行HEADに照合し、既に解決済みなら再実装しない。

Verify: 編集前status、全command結果、環境制約が記録されている。

## Phase 1: Safety net first

- D4の `run_job` event/terminal behavior testsを追加する。
- D6のJSON exact key/type/nullabilityとtext field/orderを固定する。
- cancel、default threads、invalid config、GPU stubの不足ケースを最小限補う。

Verify: MSYS2/CI同等環境でfull test。追加testが現行挙動を表し、実装詳細へ過剰結合していない。

## Phase 2: Clearly safe cleanup

- D7の純粋parse testなど、production behaviorを変えない整理を行う。
- 自分の変更でのみ不要になったimport/helperを削除する。
- docsのMSYS2実行手順を事実に基づいて最小同期する。

Verify: fmt、test、clippy、diff review。

## Phase 3: Small responsibility separation

- D1について、backend metadata・availability・execution preparationの所有範囲を狭く明確化する。
- `run_job` はapplication orchestrationの入口として維持する。
- 新しい抽象化より、既存の `backend.rs` / `result.rs` / `job.rs` の境界を使う。

Verify: CPU single/multi、default threads、GPU stub、event sequence、CLI tests。

## Phase 4: Clarify validation boundaries

- CLI/GUI/coreのvalidation責務を棚卸しし、同一規則だけを純粋helperへ寄せる。
- GUIへのthreads UI追加やCLI option変更は行わない。

Verify: invalid date/zero values/backend-specific threads、GUI build、CLI help/output。

## Phase 5: Proposal-only high-risk work

- D2のpublic API縮小、D3の細粒度cancel/streaming calculation、D5のGUI threads UX、D6のserialization方式変更、D9のCI benchmarkを実装しない。
- それぞれ現状、選択肢、互換性、測定計画、推奨案を短い設計メモとして報告する。

Verify: production diffに高リスク変更が混入していない。

## Phase 6: Final verification and diff audit

- 全required commandとsmoke testを実行する。
- `git diff --check` とファイル単位diffを読み、既存 `src/date.rs` 差分を自分の成果として扱っていないことを確認する。
- docs/test/implementationの整合を確認する。

# Verification Requirements

必須:

```powershell
cargo +stable-x86_64-pc-windows-gnu fmt -- --check
cargo +stable-x86_64-pc-windows-gnu test
cargo +stable-x86_64-pc-windows-gnu clippy --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-gnu build --release --features gui --bin gui
git diff --check
```

CLI smoke:

```powershell
cargo +stable-x86_64-pc-windows-gnu run -- --list-backends
cargo +stable-x86_64-pc-windows-gnu run -- --target 20000101 --max-digits 100 --backend cpu-single --json
cargo +stable-x86_64-pc-windows-gnu run -- --target 20000101 --max-digits 100 --backend cpu-multi --threads 2 --verify --json
cargo +stable-x86_64-pc-windows-gnu run -- --target 20000101 --max-digits 100 --chunk 25 --backend cpu-single --benchmark-only --json
cargo +stable-x86_64-pc-windows-gnu run -- --target 20000101 --max-digits 100 --backend cuda-compute --no-progress
```

期待値:

- listは計算を開始せず、CPU二種のみavailable。
- JSON stdoutは単一のparse可能なJSONで、singleのthreadsはnull、multiは2、verifyはpassed。
- benchmark-onlyは100/25で4 chunks。
- CUDAはSDKなし通常buildを壊さず、実行は明確な非ゼロerror。
- test binaryがDLL不足で起動しない場合は「test failure」と報告せず、MSYS2/CI同等環境で再実行する。再実行不能なら未検証と明記し、完了扱いにしない。

性能に触れる変更では、同一machine/toolchain/release buildでbefore/afterを最低3回測り中央値を報告する。速度改善を目的化せず、明白な回帰がないことを確認する。

# Reporting Format

```text
Summary:
- 変更した責務と、変更しなかった公開挙動

Pre-existing changes preserved:
- 開始時点のファイルと差分概要

Changed files:
- file: 依頼に直結する変更理由

Behavior verification:
- CLI JSON/text/list/progress
- CPU single/multi/verify
- GPU stub
- GUI build/cancel contract

Commands:
- PASS/FAIL/NOT RUN: exact command
- 失敗時はexact errorと、code failureかenvironment failureか

Performance:
- 測定条件とbefore/after（該当時のみ）

Proposal-only items:
- 承認が必要な設計変更と理由

Remaining debt / questions:
- 未解決事項
```

最後に実行した全コマンドと結果を必ず列挙する。test未実行・起動不能をPASSと表現しない。

# Out-of-scope Items

- CUDA/HIP/OpenCL/Vulkanの実装、SDK統合、runtime device検出。
- pi algorithm、precision、guard digits、丸め方式の変更。
- streaming/chunked pi生成や細粒度cancelの実装。
- 公開crate API、CLI、JSON/text schemaの破壊的変更。
- GUIの再設計、threads UI追加、表示文言の全面変更。
- 新しいDB、migration、認証、課金、通知、外部API、job queue、storage。
- dependency upgrade、edition変更、MSRV決定。
- CIへ不安定な絶対性能thresholdを追加すること。
- GitHub Issueのclose/edit、release、commit、push。
- 無関係なdead code削除、全体rename、format-only diff。
