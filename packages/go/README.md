# quanttide-work Go 包

知识工作工具箱（Go 包）——知识工作领域的共享能力。

## 安装

```bash
go get github.com/quanttide/quanttide-work-toolkit/packages/go@v0.1.0
```

## 使用

```go
import quanttide_work "github.com/quanttide/quanttide-work-toolkit/packages/go/pkg"

fmt.Println(quanttide_work.Domain, quanttide_work.Version)
```

## 开发

```bash
go test ./...
go vet ./...
```

## 发布

打 `go/v0.1.0` 标签推送到远端，由 `release-go.yml` 校验并生成模块别名标签 `packages/go/v0.1.0`（Go 模块按目录取版本）。

## 许可

[CC BY 4.0](../../LICENSE)
