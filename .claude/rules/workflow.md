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

## Subagent Anti-patterns (MUST avoid)

以下は実際のインシデントから抽出した失敗パターン。

### 1. Cross-system タスクを1サブエージェントに渡さない
- **Bad**: 「Rust の spatial_index.rs を変更し、JS の spatial-engine.ts も変更して、テストも更新して」→ 70分間 Read 44回で停滞
- **Good**: Rust 変更は `rust-engineer`、JS 変更は `frontend-developer` に分離。各サブエージェントは1言語・2-3ファイルに限定

### 2. サブエージェントのファイル読み込みを最小化する
- プロンプトに必要なコードスニペットと行番号を含める
- 「ファイルを読んで理解して」ではなく「L108-126 の `get_features_geojson` メソッドの後に追加」と指示
- 大きなファイル (300行+) は該当箇所のみ抜粋してプロンプトに埋め込む

### 3. 1タスクの変更ファイル数を制限する
- **上限**: 3ファイル（ソース2 + テスト1）
- 4ファイル以上触る場合はタスク分割を検討
- background agent で長時間実行させる場合は特に厳守（キャンセル不可）

### 4. background agent は小タスク専用
- `run_in_background: true` は 5分以内で完了するタスクのみ
- 10分超えそうならフォアグラウンドで実行し、進捗を監視可能にする
- 並行 background agent は max 3（git 競合リスク）

### 5. モデル選択の実績ベース基準

| 条件 | Model | 理由 |
|------|-------|------|
| 1ファイル作成、明確なコード提示 | haiku | 150秒で完了 |
| 2-3ファイル変更、TS のみ | sonnet | 5-8分で完了 |
| Rust + TS cross-system | **分割必須** | 1エージェントに混ぜない |
| 設計判断・レビュー | opus | 判断力が必要 |
