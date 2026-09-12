/// 任务：模型与流水、占位展开。
///
/// 断言对着不变量：不可变（改动拿新值）、流水只增、闸门不重复。
/// 「走过哪几步」与落点在 workspace_test.dart。
library;

import 'package:quanttide_work/quanttide_work.dart';
import 'package:test/test.dart';

/// 装一件任务：把名字并进 payload（任务名是文件里的 `name` 字段）。
Task taskOf(String name, Map payload) => Task.of({...payload, 'name': name});

void main() {
  group('JournalEvent', () {
    test('读进来、写回去', () {
      final event = JournalEvent.of({
        'at': '2026-09-12',
        'step': 'outline',
        'detail': '走了一步',
        'ok': true,
      });
      expect(event.at, '2026-09-12');
      expect(event.step, 'outline');
      expect(event.detail, '走了一步');
      expect(event.ok, isTrue);
      expect(event.toMap(), {
        'at': '2026-09-12',
        'step': 'outline',
        'detail': '走了一步',
        'ok': true,
      });
    });

    test('ok 不是 true 就算没过', () {
      expect(JournalEvent.of({'step': '甲', 'ok': 'yes'}).ok, isFalse);
    });
  });

  group('Task.of', () {
    test('读任务文件：工作流、开工处、流水、闸门、落点', () {
      final task = taskOf('甲', {
        'workflow': 'code-implement',
        'start': 'outline',
        'log': [
          {'at': 't1', 'step': 'outline', 'detail': '走了', 'ok': true},
          {'at': 't2', 'step': 'draft', 'detail': '没过', 'ok': false},
          {'at': 't3', 'step': 'review', 'detail': 'ok 不是布尔', 'ok': 'yes'},
        ],
        'gates': ['这版能不能发', 42],
        'artifacts': {'report': 'report/甲.md', 'bad': 7},
      });

      expect(task.name, '甲');
      expect(task.workflowName, 'code-implement');
      expect(task.start, 'outline');
      expect(task.journal, hasLength(3));
      expect(task.journal[2].ok, isFalse);
      expect(task.gates, ['这版能不能发']);
      expect(task.artifacts, {'report': 'report/甲.md'});
    });

    test('缺样按空算', () {
      final task = taskOf('甲', const {});
      expect(task.workflowName, '');
      expect(task.start, '');
      expect(task.journal, isEmpty);
      expect(task.gates, isEmpty);
      expect(task.artifacts, isEmpty);
    });
  });

  group('不可变改动', () {
    final task = taskOf('甲', {
      'workflow': 'w',
      'gates': ['这版能不能发'],
      'artifacts': {'report': 'report/甲.md'},
    });

    test('recorded：流水只增，原任务不动', () {
      final after = task.recorded(
        at: '2026-09-12',
        step: 'outline',
        detail: '走了一步',
        ok: true,
      );
      expect(after.journal, hasLength(task.journal.length + 1));
      expect(after.journal.last.toMap(), {
        'at': '2026-09-12',
        'step': 'outline',
        'detail': '走了一步',
        'ok': true,
      });
      expect(after.gates, task.gates);
      expect(task.journal, isEmpty);
    });

    test('withGates：已有的不重复记，原任务不动', () {
      final after = task.withGates(['这版能不能发', '换个名字']);
      expect(after.gates, ['这版能不能发', '换个名字']);
      expect(task.gates, ['这版能不能发']);
    });

    test('toMap：读回去还是同一件任务', () {
      final map = task.toMap();
      expect(map['name'], '甲');
      expect(map['workflow'], 'w');
      expect(map['start'], '');
      expect(map['gates'], ['这版能不能发']);
      expect(map['artifacts'], {'report': 'report/甲.md'});

      final back = taskOf('甲', map);
      expect(back.workflowName, task.workflowName);
      expect(back.artifacts, task.artifacts);
    });
  });

  group('声明与占位', () {
    test('declared：声明成空白等于没声明', () {
      final task = taskOf('甲', {
        'artifacts': {'report': '  ', 'journal': 'report/j.md'},
      });
      expect(task.declared('report'), isNull);
      expect(task.declared('journal'), 'report/j.md');
      expect(task.declared('missing'), isNull);
    });
  });
}
