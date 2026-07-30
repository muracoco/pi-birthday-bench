# Refactor follow-up proposals

この文書は、`refactor-instructions.md` で承認が必要とされた高リスク項目を記録する。ここに挙げた変更は今回実装していない。

## Public API inventory

`src/lib.rs` は次のmoduleをすべて公開している。

- `backend`: backend metadata、availability、`PiBackend`、CPU backend型
- `date`: `validate_yyyymmdd`
- `job`: `run_job`
- `pi`: single/multiのpi生成関数と既知prefix
- `result`: config、backend mode、result、progress event/phase、verification status
- `search`: search関数、options、outcome
- `system_info`: system info型と収集関数

repository内でCLI/GUIが利用していても、外部crateからの利用有無は確認できない。module、type、trait、field、enum variant、functionの非公開化・削除・renameは、互換性方針を確認するまで行わない。

## Backend contract ownership

`BackendInfo` と `PiBackend` は一部のmetadataを重複して持つ。将来整理する場合は次の二案がある。

1. `BackendInfo` を静的metadataの唯一の所有者とし、runtime backendは計算だけを担当する。
2. runtime backendを唯一の所有者とし、一覧表示用registryもbackend objectから生成する。

現在はGPU backendがstubでobjectを持たないため、案1の方が小さい。ただし `PiBackend` はpublic APIなので、method削除やvisibility変更の前にdownstream利用を確認する。

## Cooperative cancellation during pi calculation

現在は `compute_digits(max_digits)` が全桁を一括生成するため、計算中cancelは完了後まで反映されない。改善にはbinary splitting内部へcancel checkを渡すか、計算単位そのものを変える必要があり、性能・決定性・precisionへ影響する。

実装前に以下を決める。

- cancel latencyの目標
- check追加によるCPU single/multi性能低下の許容値
- cancel時に部分digitsを破棄するか
- progressの `digits_computed` を何として定義するか

同一machine/toolchainのrelease buildで最低3回測り、中央値をbefore/after比較する。正確性testとCPU single/multi同値testを必須にする。

## GUI thread selection

CLIは `--threads` を指定できるが、GUIはCPU multi選択時もlogical CPU数を自動利用する。GUIへ入力を追加するかはUX判断であり、今回変更していない。

追加する場合は、空欄をautoとして扱うか、既定値を表示するか、zero/parse errorの表示方法を先に決める。CLIの `RunConfig` validationと同じ規則を使い、GUI固有の別ルールを作らない。

## Result serialization

textとJSONは現在、別々の手書き変換で公開契約を維持している。今回追加したtestがJSON key集合とtext field順序を固定する。

serde deriveなどへ切り替える場合でも、field名、型、nullability、文字列値、text順序を変えない。出力変更が必要ならversioningまたは互換期間を別途設計する。

## Performance regression checks

host差が大きいため、CIへ絶対速度thresholdは追加しない。性能に関係する変更では同一host、同一toolchain、release build、同一target/max-digits/threadsで最低3回実行し、中央値と結果位置を報告する。安定したrunnerと十分な履歴が得られた場合にのみ相対thresholdを再検討する。
