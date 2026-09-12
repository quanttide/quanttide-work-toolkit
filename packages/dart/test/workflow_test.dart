/// 工作流的模型、整体语法校验与定义核对。
///
/// 断言对着不变量：缺 name / 缺 steps / 不认识的字段 / executor 越界 / criteria 不是
/// 列表，报错文字都带文件名与第几个步骤；`check` 把判据里的路径与 description 提到的
/// 小节都翻成一条条回执。
library;

import 'package:quanttide_work/quanttide_work.dart';
import 'package:test/test.dart';

void main() {
  group('工作流模型', () {
    test('Workflow.of：读步骤，按顺序；step 按名取', () {
      final workflow = Workflow.of('w', {
        'description': '走一遍',
        'steps': [
          {'name': '甲'},
          {'name': '乙', 'executor': 'human'},
        ],
      });
      expect(workflow.name, 'w');
      expect(workflow.description, '走一遍');
      expect(workflow.stepNames, ['甲', '乙']);
      expect(workflow.step('甲')!.name, '甲');
      expect(workflow.step('乙')!.executor, human);
      expect(workflow.step('没这步'), isNull);
    });

    test('Workflow.toMap：与进定义时同一形状，空说明不写', () {
      final workflow = Workflow.of('w', {
        'description': '走一遍',
        'steps': [
          {'name': '甲'},
          {'name': '乙', 'executor': 'human'},
        ],
      });
      expect(workflow.toMap(), {
        'name': 'w',
        'description': '走一遍',
        'steps': [
          {'name': '甲', 'executor': 'agent'},
          {'name': '乙', 'executor': 'human'},
        ],
      });

      final bare = Workflow.of('w', {
        'steps': [
          {'name': '甲'},
        ],
      });
      expect(bare.description, '');
      expect(bare.toMap(), {
        'name': 'w',
        'steps': [
          {'name': '甲', 'executor': 'agent'},
        ],
      });
    });

    test('Step.of：executor 没写默认 agent', () {
      final step = Step.of({'name': '甲'});
      expect(step.executor, agent);
      expect(step.isHuman, isFalse);
      expect(step.description, '');
      expect(step.criteria, isEmpty);
    });

    test('Step.of：从定义字段把判据读出来（不校验）', () {
      final step = Step.of({
        'name': '甲',
        'criteria': [
          {'executor': 'rule', 'path': '/a'},
          {'executor': 'agent', 'description': '审一下'},
        ],
      });
      expect(step.criteria, hasLength(2));
      expect(step.rules.single, isA<PathExists>());
      expect(step.agents.single, isA<AgentJudgement>());
    });

    test('Step.fromValue：读进来顺带校验', () {
      final step = Step.fromValue(
        {'name': '乙', 'executor': 'human'},
        file: 'demo.yaml',
        position: 2,
      );
      expect(step.name, '乙');
      expect(step.isHuman, isTrue);
    });

    test('Step 按判据的执行者分组：rules / agents / gates', () {
      const step = Step(
        name: '甲',
        criteria: [
          PathExists('/a'),
          AgentJudgement('审一下'),
          HumanGate('拍板'),
        ],
      );
      expect(step.rules.single, isA<PathExists>());
      expect(step.agents.single, isA<AgentJudgement>());
      expect(step.gates.single, isA<HumanGate>());
    });

    test('Step.toMap：有判据才写 criteria，有说明才写 description', () {
      const withAll = Step(
        name: '甲',
        description: '做甲',
        criteria: [PathExists('/a'), AgentJudgement('审一下')],
      );
      expect(withAll.toMap(), {
        'name': '甲',
        'description': '做甲',
        'executor': 'agent',
        'criteria': [
          {'executor': 'rule', 'path': '/a'},
          {'executor': 'agent', 'description': '审一下'},
        ],
      });

      expect(const Step(name: '乙', executor: human).toMap(), {
        'name': '乙',
        'executor': 'human',
      });
    });
  });

  group('定义校验', () {
    Matcher defError(String message) =>
        throwsA(isA<DefinitionError>().having((e) => e.message, 'message', message));

    test('DefinitionError.toString 交原文', () {
      expect(DefinitionError('少了 name').toString(), '少了 name');
    });

    test('顶层不是映射', () {
      expect(
        () => Workflow.fromValue('不是映射', file: 'demo.yaml'),
        defError('demo.yaml 的顶层不是映射（name / steps）'),
      );
    });

    test('少了 name', () {
      expect(
        () => Workflow.fromValue({
          'steps': [
            {'name': '甲'},
          ],
        }, file: 'demo.yaml'),
        defError('demo.yaml 少了 name'),
      );
    });

    test('少了 steps 或 steps 是空的', () {
      expect(
        () => Workflow.fromValue({'name': 'x'}, file: 'demo.yaml'),
        defError('demo.yaml 少了 steps（至少一个步骤）'),
      );
      expect(
        () => Workflow.fromValue({'name': 'x', 'steps': const []}, file: 'demo.yaml'),
        defError('demo.yaml 少了 steps（至少一个步骤）'),
      );
    });

    test('顶层有不认识的字段：报错列全只认哪些', () {
      expect(
        () => Workflow.fromValue({
          'name': 'x',
          'version': 2,
          'steps': [
            {'name': '甲'},
          ],
        }, file: 'demo.yaml'),
        defError('demo.yaml 顶层有不认识的字段：version（只认 name、description、steps）'),
      );
    });

    test('步骤不是映射：报错带第几个步骤', () {
      expect(
        () => Workflow.fromValue({
          'name': 'x',
          'steps': ['甲'],
        }, file: 'demo.yaml'),
        defError('demo.yaml 第 1 个步骤少了 name'),
      );
    });

    test('步骤少了 name：报错带第几个步骤', () {
      expect(
        () => Workflow.fromValue({
          'name': 'x',
          'steps': [
            {'executor': 'agent'},
          ],
        }, file: 'demo.yaml'),
        defError('demo.yaml 第 1 个步骤少了 name'),
      );
    });

    test('步骤有不认识的字段：报错带第几个步骤与只认哪些', () {
      expect(
        () => Workflow.fromValue({
          'name': 'x',
          'steps': [
            {'name': '甲', 'foo': 1},
          ],
        }, file: 'demo.yaml'),
        defError('demo.yaml 第 1 个步骤有不认识的字段：foo（只认 name、description、executor、criteria）'),
      );
    });

    test('步骤 executor 越界：只能 agent 或 human', () {
      expect(
        () => Workflow.fromValue({
          'name': 'x',
          'steps': [
            {'name': '甲', 'executor': 'auto'},
          ],
        }, file: 'demo.yaml'),
        defError('demo.yaml 第 1 个步骤的 executor 只能是 agent 或 human，实得 auto'),
      );
    });

    test('criteria 不是列表', () {
      expect(
        () => Workflow.fromValue({
          'name': 'x',
          'steps': [
            {'name': '甲', 'criteria': 'path: a'},
          ],
        }, file: 'demo.yaml'),
        defError('demo.yaml 第 1 个步骤的 criteria 应当是列表'),
      );
    });
  });

  group('小节名', () {
    test('looksLikeSection：空名与超长不算', () {
      expect(looksLikeSection(''), isFalse);
      expect(looksLikeSection('一二三四五六七八九十壹贰叁'), isFalse);
      expect(looksLikeSection('收尾'), isTrue);
    });
  });

  group('定义核对', () {
    test('looksLikeSection 先看：判据路径与 description 提到的小节都出回执', () {
      final workflow = Workflow.fromValue({
        'name': 'demo',
        'steps': [
          {
            'name': '甲',
            'description': '看 ## 案例 一节，再「收尾」一节、「小注」节、'
                '「两处」两节、「[X]」节，最后 「没闭合',
            'criteria': [
              {'executor': 'rule', 'path': '{{artifacts}}/report/甲.md'},
              {'executor': 'rule', 'path': '{{report}}/x.md'},
              {'executor': 'rule', 'path': '{{journal}}/y.md'},
              {'executor': 'rule', 'path': '{{log}}/z.md'},
              {'executor': 'rule', 'file': 'docs/index.md', 'contains': '收尾'},
              {'executor': 'rule', 'run': 'true'},
            ],
          },
        ],
      }, file: 'demo.yaml');

      final findings = workflow.check(
        '/w/data',
        (path) => path == '/w/data/artifacts/report/甲.md',
      );

      // 两处路径回执 + 四个小节回执。
      expect(findings, hasLength(6));

      final pathFindings = findings.where((f) => f.where.startsWith('甲·')).toList();
      expect(pathFindings, hasLength(2));
      expect(pathFindings[0].where, '甲·{{artifacts}}/report/甲.md');
      expect(pathFindings[0].what, '判据里的路径在不在：/w/data/artifacts/report/甲.md');
      expect(pathFindings[0].ok, isTrue);
      expect(pathFindings[1].where, '甲·docs/index.md');
      expect(pathFindings[1].what, '判据里的路径在不在：docs/index.md');
      expect(pathFindings[1].ok, isFalse);

      final sectionFindings =
          findings.where((f) => f.where == 'description').toList();
      expect(
        sectionFindings.map((f) => f.what).toList(),
        [
          'description 提到的报告小节有没有判据覆盖：案例',
          'description 提到的报告小节有没有判据覆盖：收尾',
          'description 提到的报告小节有没有判据覆盖：小注',
          'description 提到的报告小节有没有判据覆盖：两处',
        ],
      );
      expect(sectionFindings.map((f) => f.ok).toList(), [false, true, false, false]);
    });

    test('判据路径带 {{report}} / {{journal}} / {{log}} 的不核对（那是运行时才落的）', () {
      final workflow = Workflow.fromValue({
        'name': 'demo',
        'steps': [
          {
            'name': '甲',
            'criteria': [
              {'executor': 'rule', 'path': '{{report}}/x.md'},
              {'executor': 'rule', 'path': '{{journal}}/y.md'},
              {'executor': 'rule', 'path': '{{log}}/z.md'},
            ],
          },
        ],
      }, file: 'demo.yaml');

      expect(workflow.check('/w/data', (path) => true), isEmpty);
    });

    test('command 判据与没有描述的工作流不产生回执', () {
      final workflow = Workflow.fromValue({
        'name': 'demo',
        'steps': [
          {
            'name': '甲',
            'criteria': [
              {'executor': 'rule', 'run': 'true'},
            ],
          },
        ],
      }, file: 'demo.yaml');

      expect(workflow.check('/w/data', (path) => true), isEmpty);
    });
  });
}
