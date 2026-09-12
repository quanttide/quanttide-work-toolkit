/// 文档测试：`docs/user-guide/` 里每一条 Dart 示例，都真跑一遍。
///
/// 示例里的调用**原样保留**，只补它需要的夹具（工作流定义、任务文件、运行上下文）。
/// 每段前面的 `// 文档：<文件> #<第几个代码块>` 与 `scripts/doc-tests.sh` 对齐。
library;

import 'dart:io';

// 文档：criterion.md #1
// 文档：executor.md #1
// 文档：outcome.md #1
// 文档：task.md #1
// 文档：workflow.md #1
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
    expect(items.single.kind, RuleKind.path);
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

  test('文档：task.md #2——判流水', () {
    // 文档：task.md #2
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

    final done = task.doneSteps(workflow);
    final next = task.nextStep(workflow);
    final line = task.stateLine(workflow);

    expect(done, ['outline']);
    expect(next, 'draft');
    expect(line, '下一步：draft');
  });

  test('文档：task.md #3——落点、占位与上下文', () {
    // 文档：task.md #3
    final payload = {'root': '/w', 'data': '/w/data', 'workflows': '/w/workflows'};
    final task = Task(
      name: '甲',
      workflowName: 'code-implement',
      artifacts: const {'report': 'data/report/甲.md'},
    );

    final context = RunContext.of(payload);
    final place = task.artifact('report', context);
    final text = expandPlaceholders('{{report}}/清单.md', context.data);

    expect(context.root, '/w');
    expect(context.data, '/w/data');
    expect(place, '/w/data/report/甲.md');
    expect(text, '/w/data/artifacts/report/清单.md');
  });

  test('文档：task.md #4——改动拿新值', () {
    // 文档：task.md #4
    final task = Task(name: '甲', workflowName: 'code-implement');

    final after = task.recorded(at: '2026-09-12', step: 'outline', detail: '走了一步', ok: true);

    expect(after.journal, hasLength(1));
    expect(after.journal.single.step, 'outline');
    expect(after.journal.single.ok, isTrue);
    expect(task.journal, isEmpty, reason: '模型不可变，原任务不动');
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

  test('文档：workflow.md #3——定义核对', () {
    // 文档：workflow.md #3
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
    const data = '/w/data';

    final findings = workflow.check(data, (path) => File(path).existsSync());

    expect(findings, hasLength(1));
    expect(findings.single.where, '核对·docs/index.md');
    expect(findings.single.what, '判据里的路径在不在：docs/index.md');
    expect(findings.single.ok, isFalse, reason: 'docs/index.md 不在包根下');
  });
}
