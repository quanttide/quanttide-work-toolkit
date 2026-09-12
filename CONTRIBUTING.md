# 贡献指南

参与 quanttide-work-toolkit 贡献的工作约定。

## 发布约定

- 一个语言包一个工作流：`release-{语言}.yml`，标签 `{语言}/vX.Y.Z`
- 工作流先跑质量门禁（标签 ↔ 清单版本 ↔ CHANGELOG 三方校验、测试、打包预检），再发布
- 发版前把包内 CHANGELOG 的 `[Unreleased]` 落成 `## [X.Y.Z]`，并同步清单版本，否则工作流在校验步失败
- 注册表发布成功后，工作流在仓库里补一条 GitHub Release（自动生成变更说明）；`-alpha.` / `-beta.` / `-rc.` 自动标为预发布——tag 是唯一事实源，无论 tag 怎么推上来，Releases 页都不缺版本

标签建议用 `qtcloud-devops release publish -v {语言}/vX.Y.Z` 起（它顺带建 Release，版本已建的再跑不报错）。

发布前还要过三条尺子（两条 `release-*.yml` 都挂了，跑完两侧才放行）：

```bash
sh scripts/contract.sh     # 契约：两侧跑同一批向量（13 份），结论必须一样
sh scripts/coverage.sh     # 覆盖率：两侧行覆盖都要 ≥ 90%
sh scripts/doc-tests.sh    # 文档示例：user-guide 里每条示例都要有测试兜
```

各语言检查命令（与工作流一致）：

```bash
cd packages/python     && uv sync --locked && uv run pytest && uv build
cd packages/rust       && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test --locked
cd packages/typescript && npm ci && npm run typecheck && npm test
cd packages/dart       && dart analyze lib/ test/ && dart test
cd packages/go         && go vet ./... && go test ./...
```

## 提交约定

- 提交信息用中文祈使句，前缀 `feat:`、`fix:`、`docs:`、`chore:`、`refactor:`
- 提交后默认推送到远端
- 语言包内改动在语言包范围提交，仓库级文档改动单独提交
