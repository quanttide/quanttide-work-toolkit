# quanttide-work

知识工作工具箱（TypeScript 包）——知识工作领域的共享能力。

## 安装

```bash
npm install quanttide-work
```

## 使用

```ts
import { DOMAIN, VERSION } from "quanttide-work";

console.log(`${DOMAIN} v${VERSION}`);
```

## 开发

```bash
npm ci
npm run typecheck
npm test
```

## 发布

打 `typescript/v0.1.0` 标签推送到远端，由 `release-typescript.yml` 校验并发布到 npm。

## 许可

[CC BY 4.0](../../LICENSE)
