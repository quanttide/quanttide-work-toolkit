/// 任务：模型、流水与「走过」的判定、运行上下文、落点与占位。
///
/// 断言对着不变量：不可变（改动拿新值）、流水只增、闸门不重复、`doneSteps`
/// 附加判定投票与重跑从头算。
library;

import 'package:quanttide_work/quanttide_work.dart';
import 'package:test/test.dart';

void main() {
  group('RunContext', () {
    test('三处位置读出来、写回去', () {
      final context = RunContext.of({
        'root': ' /w ',
        'data': '/w/data',
        'workflows': '/w/workflows',
      });
      expect(context.root, '/w');
      expect(context.data, '/w/data');
      expect(context.workflows, '/w/workflows');
      expect(context.toMap(), {
        'root': '/w',
        'data': '/w/data',
        'workflows': '/w/workflows',
      });
    });

    test('缺样按空算', () {
      expect(RunContext.of(const {}).toMap(), {
        'root': '',
        'data': '',
        'workflows': '',
      });
    });
  });

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
    test('读任务文件：工作流、开工处、上下文、流水、闸门、落点', () {
      final task = Task.of('甲', {
        'workflow': 'code-implement',
        'start': 'outline',
        'root': '/w',
        'data': '/w/data',
        'workflows': '/w/workflows',
        'log': [
          {'at': 't1', 'step': 'outline', 'detail': '走了', 'ok': true},
          {'at': 't2', 'step': 'draft', 'detail': '没过', 'ok': false},
          {'at': 't3', 'step': 'review', 'detail': 'ok 不是布尔', 'ok': 'yes'},
        ],
        'gates': ['这版能不能发', 42],
        'artifacts': {'report': 'data/report/甲.md', 'bad': 7},
      });

      expect(task.name, '甲');
      expect(task.workflowName, 'code-implement');
      expect(task.start, 'outline');
      expect(task.context.root, '/w');
      expect(task.context.data, '/w/data');
      expect(task.context.workflows, '/w/workflows');
      expect(task.journal, hasLength(3));
      expect(task.journal[2].ok, isFalse);
      expect(task.gates, ['这版能不能发']);
      expect(task.artifacts, {'report': 'data/report/甲.md'});
    });

    test('缺样按空算', () {
      final task = Task.of('甲', const {});
      expect(task.workflowName, '');
      expect(task.start, '');
      expect(task.journal, isEmpty);
      expect(task.gates, isEmpty);
      expect(task.artifacts, isEmpty);
      expect(task.context.root, '');
    });
  });

  group('不可变改动', () {
    final task = Task.of('甲', {
      'workflow': 'w',
      'root': '/w',
      'data': '/w/data',
      'gates': ['这版能不能发'],
      'artifacts': {'report': 'data/report/甲.md'},
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
      expect(map['artifacts'], {'report': 'data/report/甲.md'});
      expect(map['root'], '/w');
      expect(map['data'], '/w/data');
      expect(map['workflows'], '');

      final back = Task.of('甲', map);
      expect(back.workflowName, task.workflowName);
      expect(back.context.toMap(), task.context.toMap());
      expect(back.artifacts, task.artifacts);
    });
  });

  group('判流水', () {
    final workflow = Workflow(
      name: 'v',
      steps: [const Step(name: '甲'), const Step(name: '乙'), const Step(name: '丙')],
    );

    test('nextStep：按工作流顺序给第一个没走到的', () {
      final task = Task(
        name: 'v',
        workflowName: 'v',
        journal: [const JournalEvent(at: 't1', step: '甲', detail: '', ok: true)],
      );
      expect(task.nextStep(workflow), '乙');
    });

    test('stateLine：下一步是谁，或者都走过了', () {
      final task = Task(
        name: 'v',
        workflowName: 'v',
        journal: [const JournalEvent(at: 't1', step: '甲', detail: '', ok: true)],
      );
      expect(task.stateLine(workflow), '下一步：乙');

      final all = task
          .recorded(at: 't2', step: '乙', detail: '', ok: true)
          .recorded(at: 't3', step: '丙', detail: '', ok: true);
      expect(all.nextStep(workflow), isNull);
      expect(all.stateLine(workflow), '3 个步骤都走过了');
    });

    test('stateLine：没有步骤的工作流给一句提示', () {
      expect(
        Task(name: 'e', workflowName: 'e').stateLine(const Workflow(name: 'e')),
        '这条工作流没有步骤——在 workflows/e.yaml 的 steps 里写步骤',
      );
    });

    test('附加判定投票：带后缀的给这一步投票，任一次不过就没过', () {
      final task = Task(
        name: 'v',
        workflowName: 'v',
        journal: [
          const JournalEvent(at: 't1', step: '甲', detail: '', ok: true),
          const JournalEvent(at: 't2', step: '甲·审', detail: '', ok: false),
        ],
      );
      expect(task.doneSteps(workflow), isEmpty);
    });

    test('重跑从头算：不带后缀的后一笔盖掉前一笔', () {
      final task = Task(
        name: 'v',
        workflowName: 'v',
        journal: [
          const JournalEvent(at: 't1', step: '乙', detail: '第一次', ok: false),
          const JournalEvent(at: 't2', step: '乙', detail: '重走一遍', ok: true),
        ],
      );
      expect(task.doneSteps(workflow), ['乙']);
    });

    test('工作流上没有的步骤名不算数', () {
      final task = Task(
        name: 'v',
        workflowName: 'v',
        journal: [
          const JournalEvent(at: 't1', step: '丁', detail: '', ok: true),
        ],
      );
      expect(task.doneSteps(workflow), isEmpty);
      expect(task.nextStep(workflow), '甲');
    });
  });

  group('落点与占位', () {
    test('declared：声明成空白等于没声明', () {
      final task = Task.of('甲', {
        'artifacts': {'report': '  ', 'journal': 'data/j.md'},
      });
      expect(task.declared('report'), isNull);
      expect(task.declared('journal'), 'data/j.md');
      expect(task.declared('missing'), isNull);
    });

    test('expandPlaceholders：四个占位各展开到数据仓', () {
      expect(
        expandPlaceholders(
          '{{artifacts}}/a {{report}}/b {{journal}} {{log}}/c',
          '/d',
        ),
        '/d/artifacts/a /d/artifacts/report/b /d/artifacts/journal /d/tasks/c',
      );
      expect(expandPlaceholders('没有占位', '/d'), '没有占位');
    });
  });
}
