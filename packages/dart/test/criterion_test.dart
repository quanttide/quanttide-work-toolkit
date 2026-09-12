/// 判据的模型、读法与「翻成要跑什么」。
///
/// 断言对着取值与不变量：默认说明按判法拼、写了说明就用写的、`toMap` 是进定义文件的
/// 形状、占位只换带 `{{` 的字段；读法的每条报错都对文字核对。
library;

import 'package:quanttide_work/quanttide_work.dart';
import 'package:test/test.dart';

void main() {
  group('判据模型', () {
    test('PathExists：执行者是 rule，说明按判法拼或照写的，写回是 path 字段', () {
      const bare = PathExists('docs/index.md');
      expect(bare.executor, rule);
      expect(bare.text, '存在：docs/index.md');
      expect(bare.toMap(), {'executor': rule, 'path': 'docs/index.md'});

      const named = PathExists('docs/index.md', description: '索引在');
      expect(named.text, '索引在');
      expect(named.toMap(), {
        'executor': rule,
        'path': 'docs/index.md',
        'description': '索引在',
      });
    });

    test('PathAbsent：写回 absent，缺说明按判法拼', () {
      const bare = PathAbsent('docs/gone.md');
      expect(bare.executor, rule);
      expect(bare.text, '不存在：docs/gone.md');
      expect(bare.toMap(), {'executor': rule, 'absent': 'docs/gone.md'});

      const named = PathAbsent('docs/gone.md', description: '清干净了');
      expect(named.text, '清干净了');
      expect(named.toMap(), {
        'executor': rule,
        'absent': 'docs/gone.md',
        'description': '清干净了',
      });
    });

    test('FileContains：写回 file + contains 两字段', () {
      const bare = FileContains('docs/index.md', '第二大脑');
      expect(bare.executor, rule);
      expect(bare.text, '含「第二大脑」：docs/index.md');
      expect(bare.toMap(), {
        'executor': rule,
        'file': 'docs/index.md',
        'contains': '第二大脑',
      });

      const named = FileContains('docs/index.md', '第二大脑', description: '点名要这句');
      expect(named.text, '点名要这句');
      expect(named.toMap(), {
        'executor': rule,
        'file': 'docs/index.md',
        'contains': '第二大脑',
        'description': '点名要这句',
      });
    });

    test('CommandRun：写回 run', () {
      const bare = CommandRun('test -f docs/index.md');
      expect(bare.executor, rule);
      expect(bare.text, '跑通：test -f docs/index.md');
      expect(bare.toMap(), {'executor': rule, 'run': 'test -f docs/index.md'});

      const named = CommandRun('test -f x', description: '命令得零');
      expect(named.text, '命令得零');
      expect(named.toMap(), {
        'executor': rule,
        'run': 'test -f x',
        'description': '命令得零',
      });
    });

    test('AgentJudgement：执行者是 agent，说明就是要审的判准', () {
      const c = AgentJudgement('写得像给人看的');
      expect(c.executor, agent);
      expect(c.text, '写得像给人看的');
      expect(c.toMap(), {'executor': agent, 'description': '写得像给人看的'});
    });

    test('HumanGate：执行者是 human，说明就是要人拍板的事', () {
      const c = HumanGate('这版能不能发');
      expect(c.executor, human);
      expect(c.text, '这版能不能发');
      expect(c.toMap(), {'executor': human, 'description': '这版能不能发'});
    });
  });

  group('占位展开', () {
    const tab = Placeholders(
      artifacts: '/d/artifacts',
      report: '/d/artifacts/report/甲.md',
      journal: '/d/artifacts/journal/甲.md',
      log: '/d/tasks/甲.yaml',
    );

    test('六种判据都换掉字段里的 {{…}}', () {
      final path = const PathExists('{{artifacts}}/index.md', description: '看 {{report}}')
          .expanded(tab);
      expect(path, isA<PathExists>());
      expect((path as PathExists).path, '/d/artifacts/index.md');
      expect(path.description, '看 /d/artifacts/report/甲.md');

      final absent = const PathAbsent('{{artifacts}}/gone.md', description: '清 {{report}}')
          .expanded(tab);
      expect(absent, isA<PathAbsent>());
      expect((absent as PathAbsent).absent, '/d/artifacts/gone.md');
      expect(absent.description, '清 /d/artifacts/report/甲.md');

      final file = const FileContains('{{report}}', '{{artifacts}}', description: '含 {{log}}')
          .expanded(tab);
      expect(file, isA<FileContains>());
      expect((file as FileContains).file, '/d/artifacts/report/甲.md');
      expect(file.contains, '/d/artifacts');
      expect(file.description, '含 /d/tasks/甲.yaml');

      final run = const CommandRun('cat {{report}}', description: '跑 {{artifacts}}')
          .expanded(tab);
      expect(run, isA<CommandRun>());
      expect((run as CommandRun).run, 'cat /d/artifacts/report/甲.md');
      expect(run.description, '跑 /d/artifacts');

      final judgement = const AgentJudgement('看 {{report}} 写完没').expanded(tab);
      expect(judgement, isA<AgentJudgement>());
      expect(judgement.description, '看 /d/artifacts/report/甲.md 写完没');

      final gate = const HumanGate('{{artifacts}} 里的要人拍板').expanded(tab);
      expect(gate, isA<HumanGate>());
      expect(gate.description, '/d/artifacts 里的要人拍板');
    });

    test('没有占位就原样，不认识的占位也原样', () {
      final plain = const PathExists('/w/docs', description: '就这点').expanded(tab);
      expect((plain as PathExists).path, '/w/docs');
      expect(plain.description, '就这点');

      final unknown = const PathExists('{{foo}}/x.md').expanded(tab);
      expect((unknown as PathExists).path, '{{foo}}/x.md');
    });
  });

  group('RuleItem', () {
    test('machine：有 kind 才要端侧跑', () {
      expect(RuleItem('存在：a', kind: RuleKind.pathExists).machine, isTrue);
      expect(RuleItem('写干净了').machine, isFalse);
      expect(RuleItem('写干净了').kind, isNull);
      expect(RuleItem('写干净了').args, isEmpty);
    });
  });

  group('读一条判据', () {
    Matcher defError(String message) =>
        throwsA(isA<DefinitionError>().having((e) => e.message('d.yaml'), 'message', message));

    test('rule 四种判法各自认出', () {
      expect(
        readCriterion({'executor': 'rule', 'path': 'a'}, 1, 1),
        isA<PathExists>(),
      );
      expect(
        readCriterion({'executor': 'rule', 'absent': 'a'}, 1, 1),
        isA<PathAbsent>(),
      );
      expect(
        readCriterion(
          {'executor': 'rule', 'file': 'a', 'contains': 'b'}, 1, 1
        ),
        isA<FileContains>(),
      );
      expect(
        readCriterion({'executor': 'rule', 'run': 'true'}, 1, 1),
        isA<CommandRun>(),
      );
    });

    test('executor 不在三选一里：报错点出三种取值', () {
      expect(
        () => readCriterion({'executor': 'auto', 'path': 'a'}, 1, 1),
        defError('d.yaml 第 1 个步骤第 1 条判据的 executor 只能是 rule / agent / human（谁判：规则引擎 / 智能体 / 人）'),
      );
    });

    test('不是映射：报「不是映射」，不绕去说 executor 该怎么写', () {
      expect(
        () => readCriterion('裸字符串', 1, 1),
        defError('d.yaml 第 1 个步骤第 1 条判据不是映射'),
      );
    });

    test('不认识的字段：报错列全只认哪些', () {
      expect(
        () => readCriterion(
          {'executor': 'rule', 'path': 'a', 'version': 2}, 1, 1
        ),
        defError('d.yaml 第 1 个步骤第 1 条判据有不认识的字段：version'
            '（只认 executor、description、path、absent、file、contains、run）'),
      );
    });

    test('rule 一条判法都没写：报错点出四种判法', () {
      expect(
        () => readCriterion({'executor': 'rule'}, 1, 1),
        defError('d.yaml 第 1 个步骤第 1 条判据是 rule，得写一条判法（path / absent / file+contains / run）'),
      );
    });

    test('写了 contains 没写 file', () {
      expect(
        () => readCriterion(
          {'executor': 'rule', 'contains': 'b'}, 1, 1
        ),
        defError('d.yaml 第 1 个步骤第 1 条判据写了 contains，还得写 file'),
      );
    });

    test('写了 file 没写 contains', () {
      expect(
        () => readCriterion({'executor': 'rule', 'file': 'a'}, 1, 1),
        defError('d.yaml 第 1 个步骤第 1 条判据写了 file，还得写 contains'),
      );
    });

    test('判法混着写：只准一种', () {
      expect(
        () => readCriterion(
          {'executor': 'rule', 'path': 'a', 'run': 'true'}, 1, 1
        ),
        defError('d.yaml 第 1 个步骤第 1 条判据的判法只能一种：path / absent / file+contains / run'),
      );
      expect(
        () => readCriterion(
          {'executor': 'rule', 'path': 'a', 'file': 'b', 'contains': 'c'}, 1, 1
        ),
        defError('d.yaml 第 1 个步骤第 1 条判据的判法只能一种：path / absent / file+contains / run'),
      );
    });

    test('agent / human 带了 rule 的字段：报错列出来', () {
      expect(
        () => readCriterion(
          {'executor': 'agent', 'description': '审一下', 'path': 'a'}, 1, 1
        ),
        defError('d.yaml 第 1 个步骤第 1 条判据是 agent，不该带 path（那是 rule 的字段）'),
      );
    });

    test('agent / human 没写 description：报错说清要写', () {
      expect(
        () => readCriterion({'executor': 'agent'}, 1, 1),
        defError('d.yaml 第 1 个步骤第 1 条判据是 agent，必须写 description（判准 / 要人拍板的事）'),
      );
    });

    test('路径里写了不认识的占位：报错列全四个', () {
      expect(
        () => readCriterion({'executor': 'rule', 'path': '{{foo}}/x.md'}, 1, 1),
        defError('d.yaml 第 1 个步骤第 1 条判据的路径里有不认识的占位：{{foo}}'
            '（只认 {{artifacts}} / {{report}} / {{journal}} / {{log}}）'),
      );
      // 只扫 path / absent / file；run 里的 `{{` 不当占位看。
      expect(
        readCriterion({'executor': 'rule', 'run': "echo '{{foo}}'"}, 1, 1),
        isA<CommandRun>(),
      );
      // 没闭合的 `{{` 不算占位。
      expect(
        readCriterion({'executor': 'rule', 'path': '{{report'}, 1, 1),
        isA<PathExists>(),
      );
    });
  });
}
