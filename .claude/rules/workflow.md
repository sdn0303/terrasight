# Development Workflow

## Context Architecture

3層構造でコンテキスト消費を最適化:

- **Rules** (`paths:` スコープ付き): 該当ファイル編集時のみロード。プロジェクト規約のみ
- **Skills** (オンデマンド): 必要時にSKILL.md → 参照ファイルと段階的に読み込み
- **Agents**: ロール定義 + 「rules/skills を参照せよ」の委譲パターン

## Feature Development Flow

```text
/brainstorm → /write-plan → design doc → worktree → implement → review → merge
```

### 1. Design Phase

- `/brainstorm` で設計を練る（要件の曖昧さを解消）
- `/write-plan` で 2-5分単位のタスクに分解

### 2. Rust Design Doc Phase (Rust 実装の前に必須)

Rust は実装前に設計を完全に固める。`docs/designs/{date}-{feature}.md` を作成:

1. **Goals / Non-Goals / Out-of-Scope** — 何を変えて何を変えないか明示
2. **Design Rules Reference** — `rust.md` の `proj-*` ルールとの対応表
3. **影響範囲** — 変更対象クレート・モジュールの特定と既存コードの現状整理
4. **Module Tree** — ファイル・モジュール構成の before/after
5. **Layer 別設計** — Domain (型・トレイト) → Infra (実装) → Usecase (ロジック) → Handler (API)
6. **命名・インターフェース** — struct/enum/trait/method の名前と型シグネチャ
7. **Testing Strategy** — 単体/結合テストの方針
8. **Migration Plan** — コミット単位の段階的実装順序
9. **Implementation Stubs** — 主要関数のシグネチャと擬似実装

Design doc が完成し内容に納得してから実装に入る。TS/Frontend は軽量な変更が多いため任意。

### 3. Implementation Phase

- `using-git-worktrees` で隔離ブランチ作成
- `subagent-driven-development` でサブエージェントに委譲
- `dispatching-parallel-agents` で独立タスクを並行実行

### 4. Quality Phase

- `code-reviewer` → 仕様準拠 + コード品質
- `security-auditor` → セキュリティ監査（Read-only）
- `test-automator` → テスト作成/改善
- `/verification-before-completion` → 動作確認

### 5. Delivery Phase

- `/finishing-a-development-branch` → クリーンアップ + マージ

## Bug Fix Flow

```text
/systematic-debugging → 仮説3つ以上 → 体系的検証 → 根本原因特定 → 修正
```

## Agent Delegation Rules

| タスク | Agent | Rules | Skill |
| ------ | ----- | ----- | ----- |
| Rust 実装 | `rust-engineer` (sonnet) | `rust.md` | `rust-backend-rules` |
| Frontend 実装 | `frontend-developer` (sonnet) | `nextjs.md`, `typescript.md` | `frontend-nextjs-rules` |
| DB 設計/最適化 | `database-admin` (sonnet) | `postgresql.md` | `postgresql-patterns` |
| コードレビュー | `code-reviewer` (opus) | 全ルール参照 | 上記3スキル全て |
| セキュリティ監査 | `security-auditor` (opus) | `security.md` | Read-only |
| テスト自動化 | `test-automator` (sonnet) | 対象言語のルール | 対象言語のスキル |

## Parallel Execution Patterns

```text
並行1: rust-engineer   → Axum handler + usecase + domain
並行2: frontend-developer → TanStack Query hook + component
並行3: database-admin  → migration + index
→ 統合テスト
```

## Subagent Anti-patterns (MUST avoid)

### 1. Cross-system タスクを1サブエージェントに渡さない

- **Bad**: Rust + JS を1エージェントに → Read 44回で停滞
- **Good**: 言語ごとにエージェント分離。各サブエージェントは1言語・2-3ファイル

### 2. ファイル読み込みを最小化する

- プロンプトにコードスニペットと行番号を含める
- 「読んで理解して」ではなく「L108-126 の後に追加」と指示
- 300行+ のファイルは該当箇所のみ抜粋

### 3. 1タスクの変更ファイル数を制限する

- **上限**: 3ファイル（ソース2 + テスト1）
- 4ファイル以上はタスク分割。background agent は特に厳守

### 4. background agent は小タスク専用

- `run_in_background: true` は 5分以内のタスクのみ
- 並行 background agent は max 3（git 競合リスク）

### 5. モデル選択基準

| 条件 | Model | 理由 |
| ---- | ----- | ---- |
| 1ファイル、明確なコード提示 | haiku | 高速完了 |
| 2-3ファイル、単一言語 | sonnet | 5-8分で完了 |
| Rust + TS cross-system | **分割必須** | 1エージェントに混ぜない |
| 設計判断・レビュー | opus | 判断力が必要 |
