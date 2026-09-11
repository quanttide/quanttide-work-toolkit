# quanttide_work

知识工作工具箱（Dart 包）——知识工作领域的共享能力。

## 安装

```yaml
dependencies:
  quanttide_work: ^0.1.0
```

## 使用

```dart
import 'package:quanttide_work/quanttide_work.dart';

void main() {
  print('$domain v$version');
}
```

## 开发

```bash
dart pub get
dart analyze lib/ test/
dart test
```

## 发布

打 `dart/v0.1.0` 标签推送到远端，由 `release-dart.yml` 校验并发布到 pub.dev。

## 许可

[CC BY 4.0](../../LICENSE)
