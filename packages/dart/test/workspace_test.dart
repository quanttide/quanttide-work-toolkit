/// 工作区：装载内容、定义核对、落点、流水判定。
///
/// 断言对着不变量：工作区按名字取定义与任务；核对把判据路径与 description 提到的
/// 小节翻成回执；落点按声明或默认；`doneSteps` 附加判定投票与重跑从头算。
library;

import 'package:quanttide_work/quanttide_work.dart';
import 'package:test/test.dart';

Task taskOf(String name, Map payload) => Task.of({...payload, 'name': name});

Workflow workflow(List<String> steps) =>
    Workflow(name: 'w', steps: [for (final step in steps) Step(name: step)]);

Task taskWithLog(List<Map> events) =>
    taskOf('甲', {'workflow': 'w', 'log': events});

Workspace workspace(List<String> steps) =>
    Workspace.of([workflow(steps)], const []);

bool? sectionOk(List<Finding> findings, String name) =>
    findings.firstWhere((finding) => finding.what.endsWith(name)).ok;

void main() {
  group('工作区模型', () {
    test('按名字取定义与任务', () {
      final task = taskOf('甲', const {});
      final space = Workspace.of([workflow(['甲', '乙'])], [task]);
      expect(space.workflow('w')!.stepNames, ['甲', '乙']);
      expect(space.task('甲'), same(task));
      expect(space.workflow('没有'), isNull);
      expect(space.task('没有'), isNull);
      expect(const Workspace().workflows, isEmpty);
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
    test('判据路径与 description 提到的小节都出回执', () {
      final flow = Workflow.fromValue({
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
      });

      final findings = const Workspace().check(
        flow,
        (path) => path == '/w/data/artifacts/report/甲.md',
      );

      // 五条路径回执（四条运行时占位「未核」+ 一条核过）+ 四个小节回执。
      expect(findings, hasLength(9));

      final pathFindings =
          findings.where((f) => f.where.startsWith('甲·')).toList();
      expect(pathFindings, hasLength(5));
      for (var i = 0; i < 4; i++) {
        expect(pathFindings[i].ok, isNull, reason: '运行时占位未核');
        expect(pathFindings[i].what, contains('未核'));
      }
      expect(pathFindings[0].where, '甲·{{artifacts}}/report/甲.md');
      expect(pathFindings[4].where, '甲·docs/index.md');
      expect(pathFindings[4].what, '判据里的路径在不在：docs/index.md');
      expect(pathFindings[4].ok, isFalse);
      expect(sectionOk(findings, '案例'), isFalse);
      expect(sectionOk(findings, '收尾'), isTrue);
    });

    test('command 判据与没有描述的工作流不产生回执', () {
      final flow = Workflow.fromValue({
        'name': 'demo',
        'steps': [
          {
            'name': '甲',
            'criteria': [
              {'executor': 'rule', 'run': 'true'},
            ],
          },
        ],
      });
      expect(const Workspace().check(flow, (path) => true), isEmpty);
    });
  });

  group('落点', () {
    test('声明了按声明的，没声明落默认处', () {
      final task = taskOf('甲', const {});
      expect(
        const Workspace().artifact(task, 'report', '/d/'),
        '/d/artifacts/report/甲.md',
        reason: '目录尾斜杠忽略',
      );
      expect(const Workspace().artifact(task, 'log', '/d/'), '/d/tasks/甲.yaml');

      final declared = taskOf('甲', {
        'artifacts': {'report': 'report/甲.md', 'logs': '/elsewhere/甲.md'},
      });
      expect(const Workspace().artifact(declared, 'report', '/d'), '/d/report/甲.md');
      expect(
        const Workspace().artifact(declared, 'logs', '/d'),
        '/elsewhere/甲.md',
        reason: '绝对路径原样',
      );
    });

    test('占位表与落点同源：四个占位各换成哪条路径', () {
      final task = taskOf('甲', const {});
      final tab = const Workspace().placeholders(task, '/d');
      expect(tab.pathOf('report'), '/d/artifacts/report/甲.md');
      expect(tab.pathOf('journal'), '/d/artifacts/journal/甲.md');
      expect(tab.pathOf('log'), '/d/tasks/甲.yaml');
      expect(tab.pathOf('artifacts'), '/d/artifacts');
      expect(tab.pathOf('foo'), isNull, reason: '不认识的占位不给路径');
      expect(
        tab.expand('{{report}} 里写 {{artifacts}} 的清单'),
        '/d/artifacts/report/甲.md 里写 /d/artifacts 的清单',
      );
      expect(tab.expand('没有占位'), '没有占位');
      expect(tab.expand('{{foo}}'), '{{foo}}', reason: '不认识的占位原样留着');
      expect(tab.expand('没闭合的 {{report'), '没闭合的 {{report');
    });

    test('占位表跟着声明走', () {
      final task = taskOf('甲', {
        'artifacts': {'report': 'report/甲.md'},
      });
      final tab = const Workspace().placeholders(task, '/d');
      expect(tab.expand('{{report}}'), '/d/report/甲.md');
      expect(tab.expand('{{journal}}'), '/d/artifacts/journal/甲.md');
    });
  });

  group('流水判定', () {
    test('附加判定投票：带后缀的给这一步投票，任一次不过就没过', () {
      final task = taskWithLog([
        {'at': 't1', 'step': '甲', 'detail': '', 'ok': true},
        {'at': 't2', 'step': '甲·审', 'detail': '', 'ok': false},
      ]);
      expect(workspace(['甲', '乙']).doneSteps(task), isEmpty);
    });

    test('重跑从头算：不带后缀的后一笔盖掉前一笔', () {
      final task = taskWithLog([
        {'at': 't1', 'step': '乙', 'detail': '第一次', 'ok': false},
        {'at': 't2', 'step': '乙', 'detail': '重走一遍', 'ok': true},
      ]);
      expect(workspace(['甲', '乙']).doneSteps(task), ['乙']);
    });

    test('工作流上没有的步骤名不算数', () {
      final task = taskWithLog([
        {'at': 't1', 'step': '丁', 'detail': '', 'ok': true},
      ]);
      expect(workspace(['甲']).doneSteps(task), isEmpty);
      expect(workspace(['甲']).nextStep(task), '甲');
    });

    test('nextStep / stateLine：下一步是谁，或者都走过了', () {
      final task = taskWithLog([
        {'at': 't1', 'step': '甲', 'detail': '', 'ok': true},
      ]);
      expect(workspace(['甲', '乙']).nextStep(task), '乙');
      expect(workspace(['甲', '乙']).stateLine(task), '下一步：乙');

      final all = taskWithLog([
        {'at': 't1', 'step': '甲', 'detail': '', 'ok': true},
        {'at': 't2', 'step': '乙', 'detail': '', 'ok': true},
      ]);
      expect(workspace(['甲', '乙']).nextStep(all), isNull);
      expect(workspace(['甲', '乙']).stateLine(all), '2 个步骤都走过了');
    });

    test('没有步骤的工作流与没有定义的工作流各给一句', () {
      final task = taskOf('甲', {'workflow': 'w'});
      final empty = Workspace.of([Workflow(name: 'w')], const []);
      expect(empty.stateLine(task), '这条工作流没有步骤：w');
      expect(
        const Workspace().stateLine(task),
        '工作区里没有这条工作流：w',
      );
      expect(const Workspace().nextStep(task), isNull);
      expect(const Workspace().doneSteps(task), isEmpty);
    });
  });
}
