//! 模型与判据的不变量：执行者取值、判据的读法与写法、翻成「要跑什么」、结果信封。
//!
//! 这一份只管 `executor` / `criterion` / `outcome` 三处公开 API——断言对着不变量与取值。

use quanttide_work::criterion::{
    Criterion, RuleItem, RuleKind, criterion_of, items_of, read_criterion,
};
use quanttide_work::executor::{AGENT, CRITERION_TYPES, EXECUTORS, HUMAN, RULE};
use quanttide_work::outcome::Outcome;
use serde_json::{Value as Json, json};
use serde_yaml::Value as Yaml;

/// JSON 本来就是合法的 YAML：夹具统一写成 JSON，再装成 YAML 值。
fn yaml(value: Json) -> Yaml {
    serde_yaml::to_value(value).expect("JSON 装成 YAML 值")
}

/// 读一条合法判据；位置固定成第 1 个步骤第 1 条判据。
fn read(value: Json) -> Criterion {
    read_criterion(&yaml(value), 1, 1).expect("合法判据")
}

/// 读一条判据，取它该报的错。
fn read_err(value: Json) -> String {
    read_criterion(&yaml(value), 1, 1)
        .expect_err("应当报错")
        .message("demo.yaml")
}

// ---------------------------------------------------------------------------
// executor
// ---------------------------------------------------------------------------

#[test]
fn executor_constants_are_the_three_fixed_values() {
    assert_eq!(AGENT, "agent");
    assert_eq!(RULE, "rule");
    assert_eq!(HUMAN, "human");
}

#[test]
fn step_executors_leave_rule_out_criteria_allow_it() {
    assert_eq!(EXECUTORS, [AGENT, HUMAN]);
    assert!(!EXECUTORS.contains(&RULE), "步骤只能 agent 或 human");
    assert_eq!(CRITERION_TYPES, [RULE, AGENT, HUMAN]);
}

// ---------------------------------------------------------------------------
// criterion::model
// ---------------------------------------------------------------------------

/// 四种 rule 判法 + 两种只能由智能体 / 人判的判据。
fn every_kind() -> Vec<Criterion> {
    vec![
        read(json!({"executor": "rule", "path": "docs/index.md"})),
        read(json!({"executor": "rule", "absent": "docs/gone.md"})),
        read(json!({"executor": "rule", "file": "docs/index.md", "contains": "第二大脑"})),
        read(json!({"executor": "rule", "run": "test -f docs/index.md"})),
        Criterion::AgentJudgement {
            description: "写干净了".into(),
        },
        Criterion::HumanGate {
            description: "人拍板".into(),
        },
    ]
}

#[test]
fn criterion_executor_follows_the_judge() {
    let got: Vec<&str> = every_kind().iter().map(Criterion::executor).collect();
    assert_eq!(got, vec![RULE, RULE, RULE, RULE, AGENT, HUMAN]);
}

#[test]
fn criterion_description_is_what_was_written() {
    let criterion = read(json!({"executor": "rule", "path": "x", "description": " 我写的 "}));
    assert_eq!(criterion.description(), "我写的", "读进来顺手去掉两侧空白");
}

#[test]
fn criterion_text_falls_back_to_the_judgement() {
    assert_eq!(
        read(json!({"executor": "rule", "path": "docs/index.md"})).text(),
        "存在：docs/index.md"
    );
    assert_eq!(
        read(json!({"executor": "rule", "absent": "docs/gone.md"})).text(),
        "不存在：docs/gone.md"
    );
    assert_eq!(
        read(json!({"executor": "rule", "file": "docs/index.md", "contains": "第二大脑"})).text(),
        "含「第二大脑」：docs/index.md"
    );
    assert_eq!(
        read(json!({"executor": "rule", "run": "test -f a"})).text(),
        "跑通：test -f a"
    );
    assert_eq!(
        Criterion::AgentJudgement {
            description: String::new()
        }
        .text(),
        "",
        "智能体没写说明就空着"
    );
    assert_eq!(
        Criterion::HumanGate {
            description: String::new()
        }
        .text(),
        ""
    );
}

#[test]
fn criterion_text_prefers_the_written_description() {
    let criterion = read(json!({"executor": "rule", "path": "x", "description": "我写的"}));
    assert_eq!(criterion.text(), "我写的");
}

#[test]
fn to_yaml_writes_the_definition_shape() {
    let yaml = read(json!({"executor": "rule", "path": "docs/index.md"})).to_yaml();
    assert_eq!(yaml.get("executor").and_then(Yaml::as_str), Some(RULE));
    assert_eq!(
        yaml.get("path").and_then(Yaml::as_str),
        Some("docs/index.md")
    );
    assert!(
        yaml.get("description").is_none(),
        "没写说明就不落 description 键"
    );
}

#[test]
fn to_yaml_roundtrips_every_kind() {
    for criterion in every_kind() {
        assert_eq!(
            criterion_of(&criterion.to_yaml()),
            criterion,
            "{criterion:?} 写回再读不一致"
        );
    }
}

#[test]
fn to_yaml_keeps_written_descriptions_for_rule_kinds() {
    let cases = [
        json!({"executor": "rule", "path": "x", "description": "存在就行"}),
        json!({"executor": "rule", "absent": "x", "description": "别出现"}),
        json!({"executor": "rule", "run": "true", "description": "命令过"}),
    ];
    for value in cases {
        let criterion = read(value);
        let written = criterion.to_yaml();
        assert_eq!(
            written.get("description").and_then(Yaml::as_str),
            Some(criterion.description()),
            "{criterion:?} 写了说明就该落 description"
        );
        assert_eq!(criterion_of(&written), criterion);
    }
}

#[test]
fn expanded_only_calls_out_for_placeholders() {
    let plain = Criterion::PathExists {
        path: "docs/index.md".into(),
        description: String::new(),
    };
    let calls = std::cell::Cell::new(0);
    let same = plain.expanded(|_| {
        calls.set(calls.get() + 1);
        String::new()
    });
    assert_eq!(same, plain);
    assert_eq!(calls.get(), 0, "每条字段都没有占位，就不该调用展开");
}

#[test]
fn expanded_replaces_every_field_with_placeholders() {
    let expand = |value: &str| {
        value
            .replace("{{report}}", "/d/artifacts/report")
            .replace("{{name}}", "甲")
    };
    let cases = [
        (
            Criterion::PathExists {
                path: "{{report}}/x.md".into(),
                description: "看{{name}}".into(),
            },
            Criterion::PathExists {
                path: "/d/artifacts/report/x.md".into(),
                description: "看甲".into(),
            },
        ),
        (
            Criterion::PathAbsent {
                absent: "{{report}}/gone.md".into(),
                description: String::new(),
            },
            Criterion::PathAbsent {
                absent: "/d/artifacts/report/gone.md".into(),
                description: String::new(),
            },
        ),
        (
            Criterion::FileContains {
                file: "{{report}}/x.md".into(),
                contains: "{{name}}".into(),
                description: String::new(),
            },
            Criterion::FileContains {
                file: "/d/artifacts/report/x.md".into(),
                contains: "甲".into(),
                description: String::new(),
            },
        ),
        (
            Criterion::CommandRun {
                run: "cat {{report}}".into(),
                description: String::new(),
            },
            Criterion::CommandRun {
                run: "cat /d/artifacts/report".into(),
                description: String::new(),
            },
        ),
        (
            Criterion::AgentJudgement {
                description: "{{name}}写干净了".into(),
            },
            Criterion::AgentJudgement {
                description: "甲写干净了".into(),
            },
        ),
        (
            Criterion::HumanGate {
                description: "{{name}}拍板".into(),
            },
            Criterion::HumanGate {
                description: "甲拍板".into(),
            },
        ),
    ];
    for (input, want) in cases {
        assert_eq!(input.expanded(expand), want);
    }
}

// ---------------------------------------------------------------------------
// criterion::read
// ---------------------------------------------------------------------------

#[test]
fn read_criterion_kinds_are_recognized() {
    assert_eq!(
        every_kind(),
        vec![
            Criterion::PathExists {
                path: "docs/index.md".into(),
                description: String::new(),
            },
            Criterion::PathAbsent {
                absent: "docs/gone.md".into(),
                description: String::new(),
            },
            Criterion::FileContains {
                file: "docs/index.md".into(),
                contains: "第二大脑".into(),
                description: String::new(),
            },
            Criterion::CommandRun {
                run: "test -f docs/index.md".into(),
                description: String::new(),
            },
            Criterion::AgentJudgement {
                description: "写干净了".into(),
            },
            Criterion::HumanGate {
                description: "人拍板".into(),
            },
        ]
    );
}

#[test]
fn criterion_of_falls_back_to_a_command_run() {
    // 没有 executor 的字段、也没有 path / absent / file，只能当成一条命令
    let bare = criterion_of(&yaml(json!({"run": "true"})));
    assert_eq!(
        bare,
        Criterion::CommandRun {
            run: "true".into(),
            description: String::new(),
        }
    );
    let nothing = criterion_of(&yaml(json!({})));
    assert_eq!(
        nothing,
        Criterion::CommandRun {
            run: String::new(),
            description: String::new(),
        }
    );
}

#[test]
fn read_rejects_an_unknown_executor() {
    assert_eq!(
        read_err(json!({"executor": "auto", "path": "x"})),
        "demo.yaml 第 1 个步骤第 1 条判据的 executor 只能是 rule / agent / human（谁判：规则引擎 / 智能体 / 人）"
    );
}

/// 判据不是映射：报「不是映射」，不绕去说 executor 该怎么写（那样会指错方向）。
#[test]
fn read_rejects_a_value_that_is_not_a_mapping() {
    assert_eq!(
        read_err(json!("裸字符串")),
        "demo.yaml 第 1 个步骤第 1 条判据不是映射"
    );
}

#[test]
fn read_rejects_unknown_fields() {
    assert_eq!(
        read_err(json!({"executor": "rule", "path": "x", "extra": 1})),
        "demo.yaml 第 1 个步骤第 1 条判据有不认识的字段：extra（只认 executor、description、path、absent、file、contains、run）"
    );
}

#[test]
fn read_rule_needs_a_way_to_judge() {
    assert_eq!(
        read_err(json!({"executor": "rule"})),
        "demo.yaml 第 1 个步骤第 1 条判据是 rule，得写一条判法（path / absent / file+contains / run）"
    );
}

#[test]
fn read_contains_needs_file_and_file_needs_contains() {
    assert_eq!(
        read_err(json!({"executor": "rule", "contains": "第二大脑"})),
        "demo.yaml 第 1 个步骤第 1 条判据写了 contains，还得写 file"
    );
    assert_eq!(
        read_err(json!({"executor": "rule", "file": "docs/index.md"})),
        "demo.yaml 第 1 个步骤第 1 条判据写了 file，还得写 contains"
    );
}

#[test]
fn read_rule_takes_only_one_way_to_judge() {
    assert_eq!(
        read_err(json!({"executor": "rule", "path": "a", "run": "b"})),
        "demo.yaml 第 1 个步骤第 1 条判据的判法只能一种：path / absent / file+contains / run"
    );
    assert_eq!(
        read_err(json!({"executor": "rule", "path": "a", "file": "b", "contains": "c"})),
        "demo.yaml 第 1 个步骤第 1 条判据的判法只能一种：path / absent / file+contains / run"
    );
}

#[test]
fn read_agent_needs_description_and_rejects_rule_fields() {
    assert_eq!(
        read_err(json!({"executor": "agent"})),
        "demo.yaml 第 1 个步骤第 1 条判据是 agent，必须写 description（判准 / 要人拍板的事）"
    );
    assert_eq!(
        read_err(json!({"executor": "agent", "description": "写干净了", "run": "true"})),
        "demo.yaml 第 1 个步骤第 1 条判据是 agent，不该带 run（那是 rule 的字段）"
    );
}

#[test]
fn read_human_needs_description_and_rejects_rule_fields() {
    assert_eq!(
        read_err(json!({"executor": "human"})),
        "demo.yaml 第 1 个步骤第 1 条判据是 human，必须写 description（判准 / 要人拍板的事）"
    );
    assert_eq!(
        read_err(json!({"executor": "human", "description": "人拍板", "path": "a"})),
        "demo.yaml 第 1 个步骤第 1 条判据是 human，不该带 path（那是 rule 的字段）"
    );
}

// ---------------------------------------------------------------------------
// criterion::items
// ---------------------------------------------------------------------------

#[test]
fn items_of_rule_carries_kind_and_args() {
    let items = items_of(&[read(
        json!({"executor": "rule", "file": "a.md", "contains": "第二大脑"}),
    )]);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].description, "含「第二大脑」：a.md");
    assert_eq!(items[0].kind, Some(RuleKind::Contains));
    assert_eq!(items[0].args, vec!["a.md", "第二大脑"]);
    assert!(items[0].machine());
}

#[test]
fn items_of_agent_and_human_are_not_run_here() {
    for criterion in [
        Criterion::AgentJudgement {
            description: "写干净了".into(),
        },
        Criterion::HumanGate {
            description: "人拍板".into(),
        },
    ] {
        let items = items_of(&[criterion]);
        assert!(items[0].kind.is_none(), "kind 为空就是不用跑");
        assert!(items[0].args.is_empty());
        assert!(!items[0].machine());
    }
}

#[test]
fn items_of_keeps_the_order_and_names_every_kind() {
    let items = items_of(&[
        read(json!({"executor": "rule", "path": "a"})),
        read(json!({"executor": "rule", "absent": "b"})),
        read(json!({"executor": "rule", "run": "c"})),
    ]);
    let kinds: Vec<Option<RuleKind>> = items.into_iter().map(|item: RuleItem| item.kind).collect();
    assert_eq!(
        kinds,
        vec![
            Some(RuleKind::Path),
            Some(RuleKind::Absent),
            Some(RuleKind::Run)
        ]
    );
}

// ---------------------------------------------------------------------------
// outcome
// ---------------------------------------------------------------------------

#[test]
fn outcome_new_and_lines_start_empty_apart_from_the_lines() {
    let blank = Outcome::new(true);
    assert!(blank.ok);
    assert!(blank.lines.is_empty());
    assert!(blank.columns.is_empty());
    assert!(blank.rows.is_empty());
    assert!(blank.data.is_none());

    let failure = Outcome::lines(false, vec!["未找到：案例".into()]);
    assert!(!failure.ok);
    assert_eq!(failure.lines, vec!["未找到：案例"]);
    assert!(failure.data.is_none());
}

#[test]
fn outcome_with_first_prepends_to_the_lines() {
    let outcome = Outcome::lines(true, vec!["第二句".into()]).with_first("第一句".into());
    assert_eq!(outcome.lines, vec!["第一句", "第二句"]);
}

#[test]
fn outcome_with_data_keeps_the_payload_and_hands_it_back_raw() {
    let payload = json!({"payload": {"name": "x"}});
    let outcome = Outcome::new(true).with_data(payload.clone());
    assert_eq!(outcome.data, Some(payload.clone()));
    assert_eq!(outcome.data_json(), payload, "托了原文就交原文");
}

#[test]
fn outcome_without_data_hands_back_the_envelope() {
    let outcome = Outcome::lines(true, vec!["走过 2 步".into()]);
    assert_eq!(outcome.data_json(), outcome.to_json());
    assert_eq!(
        outcome.to_json(),
        json!({"ok": true, "lines": ["走过 2 步"], "columns": [], "rows": []})
    );
    assert!(
        outcome.to_json().get("data").is_none(),
        "没托原文就不写 data"
    );
}

#[test]
fn outcome_results_carry_a_shared_table() {
    let outcome = Outcome {
        ok: true,
        lines: vec!["走过 2 步".into()],
        columns: vec!["步骤".into(), "状态".into()],
        rows: vec![vec!["甲".into(), "走过".into()]],
        data: None,
    };
    assert_eq!(
        outcome.to_json(),
        json!({
            "ok": true,
            "lines": ["走过 2 步"],
            "columns": ["步骤", "状态"],
            "rows": [["甲", "走过"]]
        })
    );
}

#[test]
fn outcome_from_json_fills_missing_parts_empty() {
    let outcome = Outcome::from_json(&json!({"ok": true}));
    assert!(outcome.ok);
    assert!(outcome.lines.is_empty());
    assert!(outcome.columns.is_empty());
    assert!(outcome.rows.is_empty());
    assert!(outcome.data.is_none());
    assert!(!Outcome::from_json(&json!({})).ok, "缺 ok 按 false 算");
}

#[test]
fn outcome_from_json_treats_non_arrays_as_empty() {
    let outcome = Outcome::from_json(&json!({"ok": false, "lines": "不是数组", "columns": 7}));
    assert!(outcome.lines.is_empty());
    assert!(outcome.columns.is_empty());
}

#[test]
fn outcome_from_json_stringifies_row_cells() {
    let outcome = Outcome::from_json(&json!({"rows": [[1, "b", null, true, {"k": 1}]]}));
    assert_eq!(
        outcome.rows,
        vec![vec!["1", "b", "null", "true", "{\"k\":1}"]]
    );
}

#[test]
fn outcome_roundtrips_through_json() {
    let envelope = json!({
        "ok": true,
        "lines": ["走过 2 步"],
        "columns": ["步骤"],
        "rows": [["甲"]],
        "data": {"payload": {"name": "x"}}
    });
    assert_eq!(Outcome::from_json(&envelope).to_json(), envelope);
}
