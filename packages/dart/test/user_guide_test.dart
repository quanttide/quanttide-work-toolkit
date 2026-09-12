/// 文档测试：`docs/user-guide/` 里每一条 Dart 示例，都真跑一遍。
///
/// 示例里的调用**原样保留**，只补它需要的夹具（工作流定义、任务文件、目录基准）。
/// 每段前面的 `// 文档：<文件> #<第几个代码块>` 与 `scripts/doc-tests.sh` 对齐。
library;

import 'dart:io';

import 'package:quanttide_work/quanttide_work.dart';
import 'package:test/test.dart';

void main() {
  test('文档：criterion.md #2——翻成「要跑什么」', () {
    // 文档：criterion.md #2
    final step = Step.fromValue({
      'name': '核对',
      'executor': 'agent',
      'criteria': [
        {'executor': 'rule', 'path': 'docs/index.md'},
        {'executor': 'agent', 'description': '写干净了'},
      ],
    }, position: 1);

    final items = itemsOf(step.rules);

    expect(items, hasLength(1));
    expect(items.single.description, '存在：docs/index.md');
    expect(items.single.kind, RuleKind.pathExists);
    expect(items.single.args, ['docs/index.md']);
  });

  test('文档：executor.md #2——比较时用常量', () {
    // 文档：executor.md #2
    final step = Step.fromValue({
      'name': '实现',
    }, position: 1);

    if (step.executor == agent) { /* 这一段交给智能体 */ }

    expect(step.executor, agent);
    expect(executors, [agent, human]);
    expect(criterionTypes, [rule, agent, human]);
  });

  test('文档：outcome.md #2——起一个信封', () {
    // 文档：outcome.md #2
    const root = '/w';

    final result = Outcome(true).withFirst('工作区：$root');

    expect(result.ok, isTrue);
    expect(result.lines, ['工作区：/w']);
  });

  test('文档：outcome.md #3——读回来', () {
    // 文档：outcome.md #3
    const stdout = '{"ok":true,"lines":["走过 2 步"],'
        '"columns":["步骤","状态"],"rows":[["甲","走过"],["乙","没走"]]}';

    final result = Outcome.fromStdout(stdout);

    expect(result.ok, isTrue);
    expect(result.lines, ['走过 2 步']);
    expect(result.columns, ['步骤', '状态']);
    expect(result.rows, [
      ['甲', '走过'],
      ['乙', '没走'],
    ]);
  });

  test('文档：task.md #1——引用 Task / JournalEvent', () {
    // 文档：task.md #1
    final task = Task(name: '甲', workflowName: 'code-implement');
    final journal = task.journal;
    expect(journal, isEmpty);
  });

  test('文档：task.md #2——改动拿新值', () {
    // 文档：task.md #2
    final task = Task(name: '甲', workflowName: 'code-implement');

    final after = task.recorded(at: '2026-09-12', step: 'outline', detail: '走了一步', ok: true);

    expect(after.journal, hasLength(1));
    expect(after.journal.single.step, 'outline');
    expect(after.journal.single.ok, isTrue);
    expect(task.journal, isEmpty, reason: '模型不可变，原任务不动');
  });

  test('文档：workspace.md #1——判流水', () {
    // 文档：workspace.md #1
    final workflow = Workflow(
      name: 'code-implement',
      steps: [const Step(name: 'outline'), const Step(name: 'draft')],
    );
    final task = Task(
      name: '甲',
      workflowName: 'code-implement',
      journal: [
        const JournalEvent(at: 't1', step: 'outline', detail: '走了一步', ok: true),
      ],
    );
    final workspace = Workspace.of([workflow], [task]);

    final done = workspace.doneSteps(task);
    final next = workspace.nextStep(task);
    final line = workspace.stateLine(task);

    expect(done, ['outline']);
    expect(next, 'draft');
    expect(line, '下一步：draft');
  });

  test('文档：workspace.md #2——落点与占位', () {
    // 文档：workspace.md #2
    final task = Task(name: '甲', workflowName: 'code-implement');
    final workspace = Workspace.of([Workflow.fromValue({
      'name': 'code-implement',
      'steps': [
        {'name': '实现'},
      ],
    })], [task]);

    final place = workspace.artifact(task, 'report', '/d');
    final text = expandPlaceholders('{{report}}/清单.md', '/d');

    expect(place, '/d/artifacts/report/甲.md');
    expect(text, '/d/artifacts/report/清单.md');
  });

  test('文档：workspace.md #3——定义核对', () {
    // 文档：workspace.md #3
    final workflow = Workflow.fromValue({
      'name': 'demo',
      'steps': [
        {
          'name': '核对',
          'criteria': [
            {'executor': 'rule', 'path': 'docs/index.md'},
          ],
        },
      ],
    });
    final workspace = Workspace.of([workflow], [Task(name: '甲', workflowName: 'demo')]);

    final findings = workspace.check(workflow, '/d', (path) => File(path).existsSync());

    expect(findings, hasLength(1));
    expect(findings.single.where, '核对·docs/index.md');
    expect(findings.single.what, '判据里的路径在不在：docs/index.md');
    expect(findings.single.ok, isFalse, reason: 'docs/index.md 不在包根下');
  });

  test('文档：workflow.md #2——读定义顺带校验', () {
    // 文档：workflow.md #2
    final payload = {
      'name': 'code-implement',
      'steps': [
        {'name': '实现', 'executor': 'agent'},
      ],
    };

    final workflow = Workflow.fromValue(payload);

    expect(workflow.name, 'code-implement');
    expect(workflow.stepNames, ['实现']);
  });
}
