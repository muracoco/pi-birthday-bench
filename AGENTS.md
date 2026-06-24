# AGENTS.md

日本語で簡潔に回答する。

## Scope

このファイルがあるフォルダ配下すべてに適用する。

## Coding Guidelines

- 実装前に前提と不明点を明示する。
- 仕様が複数解釈できる場合は、黙って選ばず確認する。
- 要求されていない機能、抽象化、設定項目を追加しない。
- 変更は依頼に直結する範囲に絞る。
- 既存スタイルに合わせる。
- 自分の変更で不要になった import、変数、関数は削除する。
- 検証可能な成功条件を置き、テストまたはコマンドで確認する。

## PowerShell 5.1 UTF-8

PowerShell 5.1 / NoProfile 環境では、日本語と UTF-8 を保護するため、コマンドを次の形で実行する。

```powershell
[Console]::InputEncoding=[Text.UTF8Encoding]::new($false); [Console]::OutputEncoding=[Text.UTF8Encoding]::new($false); $OutputEncoding=[Text.UTF8Encoding]::new($false); chcp 65001 > $null; & { <COMMAND> }
```

PowerShell からファイルを書く場合は `-Encoding utf8` を明示する。
