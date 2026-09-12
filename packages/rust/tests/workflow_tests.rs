//! 工作流：模型读写、整体校验、定义核对（路径在不在、description 提到的小节有没有覆盖）。

use quanttide_work::error::DefinitionError;
use quanttide_work::executor::AGENT;
use quanttide_work::workflow::{Finding, Step, Workflow, looks_like_section, validate};
use serde_json::{Value as Json, json};
use serde_yaml::Value as Yaml;

fn yaml(value: Json) -> Yaml {
    serde_yaml::to_value(value).expect("JSON 装成 YAML 值")
}

/// 用 `validate` 读一份定义，取它该报的错。
fn validate_err(value: Json, file: &str) -> String {
    validate(&yaml(value)).expect_err("应当报错").message(file)
}

/// 核对结果里 description 提到的小节名有没有被覆盖。
fn section_ok(findings: &[Finding], name: &str) -> Option<bool> {
    findings
        .iter()
        .find(|finding| finding.what.ends_with(name))
        .map(|finding| finding.ok)
}

// ---------------------------------------------------------------------------
// 报错
// ---------------------------------------------------------------------------

#[test]
fn definition_error_shows_its_message() {
    use quanttide_work::error::{Fault, Position};
    let error = DefinitionError::new(Position::Top, Fault::MissingName);
    assert_eq!(error.message("demo.yaml"), "demo.yaml 少了 name");
    assert_eq!(error.to_string(), "少了 name");
    let as_std: &dyn std::error::Error = &error;
    assert_eq!(as_std.to_string(), "少了 name");
}

// ---------------------------------------------------------------------------
// validate / from_value
// ---------------------------------------------------------------------------

#[test]
fn validate_reads_a_well_formed_definition() {
    let workflow = Workflow::from_value(&yaml(json!({
        "name": "demo",
        "description": "走一遍给我看",
        "steps": [{"name": "甲", "executor": "agent"}, {"name": "乙", "executor": "human"}]
    })))
    .expect("合法定义");
    assert_eq!(workflow.name, "demo");
    assert_eq!(workflow.step_names(), vec!["甲", "乙"]);
}

#[test]
fn validate_rejects_a_non_mapping_top() {
    assert_eq!(
        validate_err(json!(["不是映射"]), "demo.yaml"),
        "demo.yaml 的顶层不是映射（name / steps）"
    );
}

#[test]
fn validate_rejects_a_missing_name() {
    assert_eq!(
        validate_err(json!({"steps": [{"name": "甲"}]}), "demo.yaml"),
        "demo.yaml 少了 name"
    );
}

#[test]
fn validate_rejects_missing_or_empty_steps() {
    assert_eq!(
        validate_err(json!({"name": "x"}), "demo.yaml"),
        "demo.yaml 少了 steps（至少一个步骤）"
    );
    assert_eq!(
        validate_err(json!({"name": "x", "steps": []}), "demo.yaml"),
        "demo.yaml 少了 steps（至少一个步骤）"
    );
    assert_eq!(
        validate_err(json!({"name": "x", "steps": "不是列表"}), "demo.yaml"),
        "demo.yaml 少了 steps（至少一个步骤）"
    );
}

#[test]
fn validate_rejects_unknown_top_fields() {
    assert_eq!(
        validate_err(
            json!({"name": "x", "version": 2, "steps": [{"name": "甲"}]}),
            "demo.yaml"
        ),
        "demo.yaml 顶层有不认识的字段：version（只认 name、description、steps）"
    );
}

#[test]
fn validate_rejects_a_step_that_is_not_a_mapping() {
    assert_eq!(
        validate_err(json!({"name": "x", "steps": ["甲"]}), "demo.yaml"),
        "demo.yaml 第 1 个步骤少了 name"
    );
}

#[test]
fn validate_rejects_a_step_without_a_name() {
    assert_eq!(
        validate_err(
            json!({"name": "x", "steps": [{"executor": "agent"}]}),
            "demo.yaml"
        ),
        "demo.yaml 第 1 个步骤少了 name"
    );
}

#[test]
fn validate_rejects_unknown_step_fields() {
    assert_eq!(
        validate_err(
            json!({"name": "x", "steps": [{"name": "甲", "foo": 1}]}),
            "demo.yaml"
        ),
        "demo.yaml 第 1 个步骤有不认识的字段：foo（只认 name、description、executor、criteria）"
    );
}

#[test]
fn validate_rejects_an_out_of_range_step_executor() {
    assert_eq!(
        validate_err(
            json!({"name": "x", "steps": [{"name": "甲", "executor": "auto"}]}),
            "demo.yaml"
        ),
        "demo.yaml 第 1 个步骤的 executor 只能是 agent 或 human，实得 auto"
    );
}

#[test]
fn validate_rejects_criteria_that_are_not_a_list() {
    assert_eq!(
        validate_err(
            json!({"name": "x", "steps": [{"name": "甲", "criteria": "不是列表"}]}),
            "demo.yaml"
        ),
        "demo.yaml 第 1 个步骤的 criteria 应当是列表"
    );
}

#[test]
fn validate_points_at_the_offending_step_and_criterion() {
    assert_eq!(
        validate_err(
            json!({
                "name": "x",
                "steps": [
                    {"name": "甲"},
                    {"name": "乙", "criteria": [{"executor": "rule", "path": "a"}, {"executor": "rule"}]}
                ]
            }),
            "demo.yaml"
        ),
        "demo.yaml 第 2 个步骤第 2 条判据是 rule，得写一条判法（path / absent / file+contains / run）"
    );
}

#[test]
fn step_from_value_defaults_executor_and_empty_criteria() {
    let step = Step::from_value(&yaml(json!({"name": "甲"})), 1).expect("合法步骤");
    assert_eq!(step.executor(), AGENT);
    assert!(step.criteria().is_empty());

    let explicit_null = Step::from_value(
        &yaml(json!({"name": "甲", "executor": "human", "criteria": null})),
        1,
    )
    .expect("criteria 为 null 等于没写");
    assert!(explicit_null.human());
    assert!(explicit_null.criteria().is_empty());
}

// ---------------------------------------------------------------------------
// workflow::model
// ---------------------------------------------------------------------------

#[test]
fn workflow_of_reads_fields_without_checking() {
    let payload = yaml(json!({
        "name": "code-implement",
        "description": "实现一段代码",
        "steps": [
            {"name": "大纲", "description": "列大纲", "criteria": [{"executor": "rule", "path": "outline.md"}]},
            {"name": "收尾", "executor": "human"}
        ]
    }));
    let workflow = Workflow::of("code-implement.yaml", &payload);
    assert_eq!(workflow.name, "code-implement.yaml", "name 由调用方给");
    assert_eq!(workflow.description(), "实现一段代码");
    assert_eq!(workflow.step_names(), vec!["大纲", "收尾"]);
}

#[test]
fn workflow_step_and_steps_accessors() {
    let workflow = Workflow::from_value(&yaml(json!({
        "name": "w",
        "steps": [{"name": "甲", "description": "做甲"}, {"name": "乙", "executor": "human"}]
    })))
    .expect("合法定义");
    assert_eq!(workflow.steps().len(), 2);
    assert_eq!(
        workflow.step("甲").map(|step| step.description()),
        Some("做甲".into())
    );
    assert!(workflow.step("没有").is_none());
}

#[test]
fn step_groups_criteria_by_judge() {
    let step = Step::of(&yaml(json!({
        "name": "甲",
        "criteria": [
            {"executor": "rule", "path": "a"},
            {"executor": "agent", "description": "写干净了"},
            {"executor": "human", "description": "人拍板"}
        ]
    })));
    assert_eq!(step.executor(), AGENT, "没写 executor 默认 agent");
    assert!(!step.human());
    assert_eq!(step.rules().len(), 1);
    assert_eq!(step.agents().len(), 1);
    assert_eq!(step.gates().len(), 1);
    assert_eq!(step.name(), "甲");
}

#[test]
fn step_of_without_executor_or_criteria_defaults() {
    let step = Step::of(&yaml(json!({"name": "甲"})));
    assert_eq!(step.description(), "");
    assert_eq!(step.executor(), AGENT);
    assert!(step.criteria().is_empty());
}

#[test]
fn step_to_yaml_omits_empty_description_and_criteria() {
    let step = Step {
        name: "甲".into(),
        description: String::new(),
        executor: AGENT.into(),
        criteria: Vec::new(),
    };
    let written = step.to_yaml();
    assert!(written.get("description").is_none());
    assert!(written.get("criteria").is_none());
    assert_eq!(written.get("executor").and_then(Yaml::as_str), Some(AGENT));
    assert_eq!(written.get("name").and_then(Yaml::as_str), Some("甲"));
}

#[test]
fn workflow_to_yaml_roundtrips() {
    let payload = yaml(json!({
        "name": "code-implement",
        "description": "实现一段代码",
        "steps": [
            {
                "name": "大纲",
                "description": "列大纲",
                "criteria": [
                    {"executor": "rule", "path": "outline.md"},
                    {"executor": "rule", "file": "report.md", "contains": "大纲", "description": "报告里有大纲"}
                ]
            },
            {"name": "收尾", "executor": "human", "criteria": [{"executor": "human", "description": "人拍板"}]}
        ]
    }));
    let workflow = Workflow::from_value(&payload).expect("合法定义");
    let back = Workflow::from_value(&workflow.to_yaml()).expect("写回仍合法");
    assert_eq!(back, workflow);
}

#[test]
fn workflow_to_yaml_without_description_still_lists_steps() {
    let workflow = Workflow {
        name: "w".into(),
        description: String::new(),
        steps: Vec::new(),
    };
    let written = workflow.to_yaml();
    assert!(written.get("description").is_none());
    assert_eq!(
        written
            .get("steps")
            .and_then(Yaml::as_sequence)
            .map(Vec::len),
        Some(0)
    );
}

// ---------------------------------------------------------------------------
// workflow::check
// ---------------------------------------------------------------------------

#[test]
fn looks_like_section_rejects_non_names() {
    assert!(looks_like_section("收尾"));
    assert!(!looks_like_section(""), "空名不算");
    assert!(
        !looks_like_section("这是一个超过十二个字的报告小节名字"),
        "太长不算"
    );
    for odd in ["<标题>", "a_b", "`代号`", "A>B"] {
        assert!(!looks_like_section(odd), "{odd} 不算小节名");
    }
}

#[test]
fn check_reports_paths_and_sections() {
    let workflow = Workflow::from_value(&yaml(json!({
        "name": "code-implement",
        "steps": [{
            "name": "甲",
            "description": "交一份带 ## 收尾 的报告，并按「结论」一节写清楚",
            "criteria": [
                {"executor": "rule", "path": "/w/seen.md"},
                {"executor": "rule", "file": "report.md", "contains": "收尾"}
            ]
        }]
    })))
    .expect("合法定义");

    let findings = workflow.check("/w", |path| path == "/w/seen.md");

    let path_finding = findings
        .iter()
        .find(|finding| finding.what.starts_with("判据里的路径在不在"))
        .expect("路径判据应当出一条核对");
    assert_eq!(path_finding.where_, "甲·/w/seen.md");
    assert_eq!(path_finding.what, "判据里的路径在不在：/w/seen.md");
    assert!(path_finding.ok);

    assert_eq!(
        section_ok(&findings, "收尾"),
        Some(true),
        "contains 覆盖了收尾"
    );
    assert_eq!(
        section_ok(&findings, "结论"),
        Some(false),
        "没人写结论的判据"
    );
}

#[test]
fn check_skips_placeholders_and_unchecked_kinds() {
    let workflow = Workflow::from_value(&yaml(json!({
        "name": "w",
        "steps": [{
            "name": "甲",
            "criteria": [
                {"executor": "rule", "path": "{{report}}/x.md"},
                {"executor": "rule", "path": "{{journal}}/y.md"},
                {"executor": "rule", "path": "{{log}}/z.yaml"},
                {"executor": "rule", "absent": "gone.md"},
                {"executor": "rule", "run": "true"},
                {"executor": "agent", "description": "写干净了"},
                {"executor": "human", "description": "人拍板"}
            ]
        }]
    })))
    .expect("合法定义");

    let findings = workflow.check("/d", |_| true);
    assert!(
        findings.is_empty(),
        "占位路径与不查的判法都不出核对项：{findings:?}"
    );
}

#[test]
fn check_dedupes_section_mentions_and_ignores_unclosed_ones() {
    let workflow = Workflow::from_value(
        &yaml(json!({
            "name": "w",
            "steps": [{
                "name": "甲",
                "description": "先写「结论」节，再写「结论」两节，最后「收尾」一节；未闭合的「半句和 ## 版本",
                "criteria": [
                    {"executor": "rule", "path": "a.md"},
                    {"executor": "rule", "file": "b.md", "contains": "结论"}
                ]
            }]
        })))
    .expect("合法定义");

    let findings = workflow.check("/d", |_| true);
    assert_eq!(section_ok(&findings, "结论"), Some(true));
    assert_eq!(section_ok(&findings, "收尾"), Some(false));
    assert_eq!(section_ok(&findings, "半句和"), None, "没闭合的「不往后认");
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.what.ends_with("结论"))
            .count(),
        1,
        "同一个小节只出一条"
    );
}
