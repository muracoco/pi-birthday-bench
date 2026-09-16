# ベンチマーク手順

比較するのは、実行時のpi計算と数字列の検索を合わせた処理です。既存のpiファイルを読み込む速度や、GPUの速度を測るツールではありません。

## 先に小さく確認する

[README](../README.md) の環境を用意し、1,000桁で両CPU backendを確認します。以下はWindowsのMINGW64ターミナル用です。Linuxでは `+stable-x86_64-pc-windows-gnu` を外してください。

```bash
cargo +stable-x86_64-pc-windows-gnu build --locked --release --bin pi-birthday-bench
./target/release/pi-birthday-bench --target 20000101 --max-digits 1000 --backend cpu-single --verify --json
./target/release/pi-birthday-bench --target 20000101 --max-digits 1000 --backend cpu-multi --threads 2 --verify --json
```

両方の `found` と `first_position` が一致し、`verification_status` が `passed` になることを確認します。速度の一致は求めません。1,000桁のような短い処理では、スレッドの起動などの負担でマルチスレッドの方が遅くなる場合もあります。

## 条件を揃えて測る

同じ実行ファイル、対象文字列、桁数、chunkサイズで実行します。途中で見つかっても同じ範囲を処理するため、`--benchmark-only` を付けます。スレッド数も明示します。

```bash
./target/release/pi-birthday-bench --target 20000101 --max-digits 1000000 --chunk 100000 --backend cpu-single --benchmark-only --json
./target/release/pi-birthday-bench --target 20000101 --max-digits 1000000 --chunk 100000 --backend cpu-multi --threads 2 --benchmark-only --json
```

初回の準備実行と測定を分け、同じ条件で複数回実行して個々の結果と中央値を残します。バックグラウンドの負荷、電源設定、温度によって結果は変わります。単発の最高値だけで高速化を主張しないでください。

結果を共有するときは、次の情報を添えると比較できます。

- `git rev-parse HEAD` のコミットと `rustc --version` の結果
- OS、CPU、スレッド数、ビルド種別（`--release`）、GMPのバージョン
- 実行したコマンドと、各回のJSON結果
- `--verify` の有無、準備実行と測定回数、同時に動いていた重い処理

対象の日付は任意の作例で構いません。実在の人の誕生日や、氏名・ユーザーフォルダを含む端末出力を公開する必要はありません。

## 値の意味

| 項目 | 読み方 |
|---|---|
| `elapsed_seconds` | pi計算、指定された検証、検索の経過時間。ビルド時間や結果の出力時間は含めない。 |
| `digits_per_second` | `digits_computed / elapsed_seconds`。検索だけの速度ではない。 |
| `digits_computed` | 先に計算した小数部の桁数。通常検索で早期終了しても計算済みの桁数を返す。 |
| `chunks_processed` | 検索で通過したchunk数。通常検索では途中で止まる場合がある。 |
| `first_position` | 小数部の1始まり位置。整数部の3は数えない。未発見なら `null`。 |
| `verification_status` | `--verify` なしは `skipped`。成功時は `passed`。検証に失敗するとCLIはエラー終了する。 |

`--verify` は既知の先頭100桁までを照合し、`cpu-multi` では最大1,000桁を単一CPU実装と比較します。全桁の独立検算ではありません。速度比較では両方に付けるか、事前検証の後で両方から外すかを揃えてください。

## 現在の限界

- メモリに全桁を保持してから検索します。`--chunk` を小さくしてもpi計算のメモリ消費は減りません。初回から極端に大きな桁数を指定しないでください。
- GPU backendは未実装です。対応featureを付けてもGPU計算ができるわけではありません。
- CPU数やメモリ量は取得できない環境では `null` です。ピークメモリなど、未実装の測定項目もあります。
- GUIのキャンセルはpi計算中に即時停止するとは限りません。
- ハードウェア間の性能順位や、マルチスレッドで必ず速くなることは保証していません。
