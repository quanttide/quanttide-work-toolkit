import '../fields.dart';

/// 任务聚合 / 流水：什么时候、哪一步、一句话、过没过。
///
/// 流水只增不改，一条的记录见 [JournalEvent]。
/// 「走过哪几步」是跨着任务与定义的操作，在 `workspace/`。
/// 出处：`docs/specification/process/task.md`·语法。

/// 流水里的一条：什么时候、哪一步、一句话、过没过。
class JournalEvent {
  const JournalEvent({
    required this.at,
    required this.step,
    required this.detail,
    required this.ok,
  });

  factory JournalEvent.of(Map value) => JournalEvent(
    at: textOf(value, 'at'),
    step: textOf(value, 'step'),
    detail: textOf(value, 'detail'),
    ok: value['ok'] == true,
  );

  final String at;
  final String step;
  final String detail;
  final bool ok;

  Map<String, Object?> toMap() => {
    'at': at,
    'step': step,
    'detail': detail,
    'ok': ok,
  };
}
