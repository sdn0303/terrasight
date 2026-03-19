# Development Workflow

## Feature Development Flow

```
/brainstorm → /write-plan → worktree → implement → review → merge
```

### 1. Design Phase
- `/brainstorm` で設計を練る（要件の曖昧さを解消）
- `/write-plan` で 2-5分単位のタスクに分解

### 2. Implementation Phase
- `using-git-worktrees` で隔離ブランチ作成
- `subagent-driven-development` でサブエージェントに委譲:
  - **Backend**: `rust-engineer` → Rust Axum 実装
  - **Frontend**: `frontend-developer` → Next.js 16 実装
  - **Database**: `database-admin` → スキーマ/マイグレーション
- `dispatching-parallel-agents` で独立タスクを並行実行

### 3. Quality Phase
- `code-reviewer` エージェント → 仕様準拠 + コード品質
- `security-auditor` エージェント → セキュリティ監査（Read-only）
- `test-automator` エージェント → テスト作成/改善
- `/verification-before-completion` → 動作確認

### 4. Delivery Phase
- `/finishing-a-development-branch` → クリーンアップ + マージ

## Bug Fix Flow

```
/systematic-debugging → 4段階根本原因分析 → 修正 → テスト → 検証
```

1. 症状の正確な記述
2. 仮説の列挙（最低3つ）
3. 仮説の体系的検証
4. 根本原因の特定と修正

## Code Review Flow

```
/requesting-code-review → reviewer の指摘対応 → /receiving-code-review
```

## Agent Delegation Rules

| タスク | Agent | Model | 権限 |
|--------|-------|-------|------|
| Rust 実装 | `rust-engineer` | sonnet | Read/Write/Edit/Bash |
| Frontend 実装 | `frontend-developer` | sonnet | Read/Write/Edit/Bash |
| DB 設計/最適化 | `database-admin` | sonnet | Read/Write/Edit/Bash |
| コードレビュー | `code-reviewer` | opus | Read/Grep/Glob/Bash |
| セキュリティ監査 | `security-auditor` | opus | Read/Grep/Glob (Read-only) |
| テスト自動化 | `test-automator` | sonnet | Read/Write/Edit/Bash |

## Parallel Execution Patterns

独立したタスクは `dispatching-parallel-agents` で並行実行:

```
# 例: 新しいAPIエンドポイント追加
並行1: rust-engineer → Axum handler + usecase + domain
並行2: frontend-developer → TanStack Query hook + component
並行3: database-admin → migration + index
→ 統合テスト
```
