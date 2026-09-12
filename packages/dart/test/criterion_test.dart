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
    test('六种判据都换掉字段里的 {{…}}', () {
      String expand(String value) => value
          .replaceAll('{{artifacts}}', '/d/artifacts')
          .replaceAll('{{report}}', '/d/artifacts/report');

      final path = const PathExists('{{artifacts}}/index.md', description: '看 {{report}}')
          .expanded(expand);
      expect(path, isA<PathExists>());
      expect((path as PathExists).path, '/d/artifacts/index.md');
      expect(path.description, '看 /d/artifacts/report');

      final absent = const PathAbsent('{{artifacts}}/gone.md', description: '清 {{report}}')
          .expanded(expand);
      expect(absent, isA<PathAbsent>());
      expect((absent as PathAbsent).absent, '/d/artifacts/gone.md');
      expect(absent.description, '清 /d/artifacts/report');

      final file = const FileContains('{{report}}/x.md', '{{artifacts}}', description: '含 {{report}}')
          .expanded(expand);
      expect(file, isA<FileContains>());
      expect((file as FileContains).file, '/d/artifacts/report/x.md');
      expect(file.contains, '/d/artifacts');
      expect(file.description, '含 /d/artifacts/report');

      final run = const CommandRun('cat {{report}}', description: '跑 {{artifacts}}')
          .expanded(expand);
      expect(run, isA<CommandRun>());
      expect((run as CommandRun).run, 'cat /d/artifacts/report');
      expect(run.description, '跑 /d/artifacts');

      final judgement = const AgentJudgement('看 {{report}} 写完没').expanded(expand);
      expect(judgement, isA<AgentJudgement>());
      expect(judgement.description, '看 /d/artifacts/report 写完没');

      final gate = const HumanGate('{{artifacts}} 里的要人拍板').expanded(expand);
      expect(gate, isA<HumanGate>());
      expect(gate.description, '/d/artifacts 里的要人拍板');
    });

    test('字段里没有 {{ 时不调展开函数', () {
      var called = 0;
      final plain = const PathExists('/w/docs', description: '就这点')
          .expanded((value) {
        called++;
        return value;
      });
      expect((plain as PathExists).path, '/w/docs');
      expect(plain.description, '就这点');
      expect(called, 0);
    });
  });

  group('RuleItem', () {
    test('machine：有 kind 才要端侧跑', () {
      expect(RuleItem('存在：a', kind: RuleKind.path).machine, isTrue);
      expect(RuleItem('写干净了').machine, isFalse);
      expect(RuleItem('写干净了').kind, isNull);
      expect(RuleItem('写干净了').args, isEmpty);
    });
  });

  group('读一条判据', () {
    Matcher defError(String message) =>
        throwsA(isA<DefinitionError>().having((e) => e.message, 'message', message));

    const place = '第 1 个步骤第 1 条判据';

    test('rule 四种判法各自认出', () {
      expect(
        readCriterion({'executor': 'rule', 'path': 'a'}, file: 'd.yaml', place: place),
        isA<PathExists>(),
      );
      expect(
        readCriterion({'executor': 'rule', 'absent': 'a'}, file: 'd.yaml', place: place),
        isA<PathAbsent>(),
      );
      expect(
        readCriterion(
          {'executor': 'rule', 'file': 'a', 'contains': 'b'},
          file: 'd.yaml',
          place: place,
        ),
        isA<FileContains>(),
      );
      expect(
        readCriterion({'executor': 'rule', 'run': 'true'}, file: 'd.yaml', place: place),
        isA<CommandRun>(),
      );
    });

    test('executor 不在三选一里：报错点出三种取值', () {
      expect(
        () => readCriterion({'executor': 'auto', 'path': 'a'}, file: 'd.yaml', place: place),
        defError('d.yaml 第 1 个步骤第 1 条判据的 executor 只能是 rule / agent / human（谁判：规则引擎 / 智能体 / 人）'),
      );
    });

    test('不是映射：报「不是映射」，不绕去说 executor 该怎么写', () {
      expect(
        () => readCriterion('裸字符串', file: 'd.yaml', place: place),
        defError('d.yaml 第 1 个步骤第 1 条判据不是映射'),
      );
    });

    test('不认识的字段：报错列全只认哪些', () {
      expect(
        () => readCriterion(
          {'executor': 'rule', 'path': 'a', 'version': 2},
          file: 'd.yaml',
          place: place,
        ),
        defError('d.yaml 第 1 个步骤第 1 条判据有不认识的字段：version'
            '（只认 executor、description、path、absent、file、contains、run）'),
      );
    });

    test('rule 一条判法都没写：报错点出四种判法', () {
      expect(
        () => readCriterion({'executor': 'rule'}, file: 'd.yaml', place: place),
        defError('d.yaml 第 1 个步骤第 1 条判据是 rule，得写一条判法（path / absent / file+contains / run）'),
      );
    });

    test('写了 contains 没写 file', () {
      expect(
        () => readCriterion(
          {'executor': 'rule', 'contains': 'b'},
          file: 'd.yaml',
          place: place,
        ),
        defError('d.yaml 第 1 个步骤第 1 条判据写了 contains，还得写 file'),
      );
    });

    test('写了 file 没写 contains', () {
      expect(
        () => readCriterion({'executor': 'rule', 'file': 'a'}, file: 'd.yaml', place: place),
        defError('d.yaml 第 1 个步骤第 1 条判据写了 file，还得写 contains'),
      );
    });

    test('判法混着写：只准一种', () {
      expect(
        () => readCriterion(
          {'executor': 'rule', 'path': 'a', 'run': 'true'},
          file: 'd.yaml',
          place: place,
        ),
        defError('d.yaml 第 1 个步骤第 1 条判据的判法只能一种：path / absent / file+contains / run'),
      );
      expect(
        () => readCriterion(
          {'executor': 'rule', 'path': 'a', 'file': 'b', 'contains': 'c'},
          file: 'd.yaml',
          place: place,
        ),
        defError('d.yaml 第 1 个步骤第 1 条判据的判法只能一种：path / absent / file+contains / run'),
      );
    });

    test('agent / human 带了 rule 的字段：报错列出来', () {
      expect(
        () => readCriterion(
          {'executor': 'agent', 'description': '审一下', 'path': 'a'},
          file: 'd.yaml',
          place: place,
        ),
        defError('d.yaml 第 1 个步骤第 1 条判据是 agent，不该带 path（那是 rule 的字段）'),
      );
    });

    test('agent / human 没写 description：报错说清要写', () {
      expect(
        () => readCriterion({'executor': 'agent'}, file: 'd.yaml', place: place),
        defError('d.yaml 第 1 个步骤第 1 条判据是 agent，必须写 description（判准 / 要人拍板的事）'),
      );
    });
  });
}
