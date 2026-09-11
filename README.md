# quanttide-work-toolkit

量潮知识工作工具箱 — 知识工作领域的共享库与工具集（独立仓库，挂载于 quanttide-work/packages）。

## 概述

承载知识工作领域的纯逻辑与共享能力，按语言拆分为独立包：

| 包 | 语言 | 说明 |
|---|---|---|
| [`packages/python`](packages/python) | Python | 知识工作 Python 包（`quanttide-work`） |
| [`packages/rust`](packages/rust) | Rust | 知识工作 Rust 库（`quanttide-work`） |

新增语言包时在 `packages/{语言}/` 下独立发布，互不依赖。

## 发布自动化

待补充——语言包就绪后，由 tag 触发的发布工作流承担。

## 许可

[CC BY 4.0](LICENSE)
