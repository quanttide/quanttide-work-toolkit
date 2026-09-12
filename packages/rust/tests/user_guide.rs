//! 用户手册（`docs/user-guide/`）里的 Rust 示例，逐条真跑一遍。
//!
//! 示例里的调用原样保留；这里只补它需要的夹具（真工作流定义、任务文件、运行上下文）。
//! 编号按各文件里 ```rust 代码块的出现次序。

use quanttide_work::criterion::{RuleKind, criterion_of, items_of};
use quanttide_work::error::DefinitionError;
use quanttide_work::executor::AGENT;
use quanttide_work::outcome::Outcome;
use quanttide_work::paths::expand_placeholders;
use quanttide_work::task::{RunContext, Task};
use quanttide_work::workflow::{Step, Workflow};
use serde_json::{Value as Json, json};
use serde_yaml::Value as Yaml;

fn yaml(value: Json) -> Yaml {
    serde_yaml::to_value(value).expect("JSON 装成 YAML 值")
}

/// 手册里反复出现的工作流夹具。
fn code_implement() -> Workflow {
    Workflow::from_value(&yaml(json!({
        "name": "code-implement",
        "description": "实现一段代码",
        "steps": [
            {"name": "大纲", "executor": "agent"},
            {"name": "收尾", "executor": "human"}
        ]
    })))
    .expect("夹具工作流合法")
}

// ---------------------------------------------------------------------------
// criterion.md
// ---------------------------------------------------------------------------

// 文档：criterion.md #1
#[test]
fn doc_criterion_1() {
    use quanttide_work::criterion::{
        Criterion, RuleItem, RuleKind, criterion_of, items_of, read_criterion,
    };
    let parsed: Criterion =
        criterion_of(&yaml(json!({"executor": "rule", "path": "docs/index.md"})));
    assert_eq!(parsed.text(), "存在：docs/index.md");
    let items: Vec<RuleItem> = items_of(&[parsed]);
    assert_eq!(items[0].kind, Some(RuleKind::Path));
    let checked = read_criterion(
        &yaml(json!({"executor": "rule", "path": "docs/index.md"})),
        1,
        1,
    );
    assert!(checked.is_ok());
}

// 文档：criterion.md #2
#[test]
fn doc_criterion_2() {
    let step = Step {
        name: "大纲".into(),
        description: String::new(),
        executor: AGENT.into(),
        criteria: vec![
            criterion_of(&yaml(json!({"executor": "rule", "path": "outline.md"}))),
            criterion_of(&yaml(
                json!({"executor": "agent", "description": "写干净了"}),
            )),
        ],
    };
    let items = items_of(&step.rules()); // 工具箱：翻成「要跑什么」
    // 端侧：真的去查文件、起进程，把结果装进 Outcome
    assert_eq!(items.len(), 1, "只有 rule 那条要跑");
    assert_eq!(items[0].description, "存在：outline.md");
    assert_eq!(items[0].kind, Some(RuleKind::Path));
}

// ---------------------------------------------------------------------------
// executor.md
// ---------------------------------------------------------------------------

// 文档：executor.md #1
#[test]
fn doc_executor_1() {
    use quanttide_work::executor::{AGENT, CRITERION_TYPES, EXECUTORS, HUMAN, RULE};
    assert_eq!([AGENT, RULE, HUMAN], ["agent", "rule", "human"]);
    assert_eq!(EXECUTORS, [AGENT, HUMAN]);
    assert_eq!(CRITERION_TYPES, [RULE, AGENT, HUMAN]);
}

// 文档：executor.md #2
#[test]
fn doc_executor_2() {
    let step = Step {
        name: "甲".into(),
        description: String::new(),
        executor: AGENT.into(),
        criteria: Vec::new(),
    };
    let mut ran = false;
    if step.executor() == AGENT {
        /* 这一段交给智能体 */
        ran = true;
    }
    assert!(ran, "executor 为 agent 时走智能体那一段");
}

// ---------------------------------------------------------------------------
// outcome.md
// ---------------------------------------------------------------------------

// 文档：outcome.md #1
#[test]
fn doc_outcome_1() {
    use quanttide_work::outcome::Outcome;
    let outcome = Outcome::new(true);
    assert!(outcome.ok);
}

// 文档：outcome.md #2
#[test]
fn doc_outcome_2() {
    let root = "/w";
    let result = Outcome::new(true).with_first(format!("工作区：{}", root)); // 起一个信封
    let failed = Outcome::lines(false, vec!["未找到：案例".into()]); // 直接给几行
    assert_eq!(result.lines, vec!["工作区：/w"]);
    assert!(result.ok);
    assert!(!failed.ok);
    assert_eq!(failed.lines, vec!["未找到：案例"]);
}

// 文档：outcome.md #3
#[test]
fn doc_outcome_3() {
    let value = json!({"ok": true, "lines": ["走过 2 步"]});
    let result = Outcome::from_json(&value); // to_json() 是它的反操作
    assert!(result.ok);
    assert_eq!(result.lines, vec!["走过 2 步"]);
    assert_eq!(
        result.to_json(),
        json!({"ok": true, "lines": ["走过 2 步"], "columns": [], "rows": []})
    );
}

// ---------------------------------------------------------------------------
// task.md
// ---------------------------------------------------------------------------

// 文档：task.md #1
#[test]
fn doc_task_1() {
    use quanttide_work::paths::expand_placeholders;
    use quanttide_work::task::{JournalEvent, RunContext, Task};
    let task = Task::of("甲", &yaml(json!({})));
    let _: Vec<JournalEvent> = task.journal.clone();
    let context = RunContext::default();
    assert_eq!(expand_placeholders("{{log}}", &context.data), "/tasks");
    assert_eq!(task.name, "甲");
}

// 文档：task.md #2
#[test]
fn doc_task_2() {
    let workflow = code_implement();
    let task = Task::of(
        "甲",
        &yaml(json!({
            "workflow": "code-implement",
            "log": [
                {"at": "t1", "step": "大纲", "detail": "走了", "ok": true},
                {"at": "t2", "step": "大纲·审", "detail": "审过", "ok": false}
            ]
        })),
    );
    let done = task.done_steps(&workflow); // 附加判定投票、重跑从头算
    let next = task.next_step(&workflow);
    let line = task.state_line(&workflow); // 给用户看的一句话
    assert!(done.is_empty(), "附加判定投票把大纲否掉了");
    assert_eq!(next, Some("大纲".to_string()));
    assert_eq!(line, "下一步：大纲");
}

// 文档：task.md #3
#[test]
fn doc_task_3() {
    let task = Task::of(
        "甲",
        &yaml(json!({"artifacts": {"report": "data/report/甲.md"}})),
    );
    let payload = yaml(json!({"root": "/w", "data": "/w/data", "workflows": "/w/workflows"}));
    let context = RunContext::of(&payload); // 三处位置
    let place = task.artifact("report", &context); // 落点：声明了按声明的，没声明落数据仓
    let text = expand_placeholders("{{report}}/清单.md", &context.data); // 占位展开
    assert_eq!(context.workflows, "/w/workflows");
    assert_eq!(place, "/w/data/report/甲.md");
    assert_eq!(text, "/w/data/artifacts/report/清单.md");
}

// 文档：task.md #4
#[test]
fn doc_task_4() {
    let task = Task::of("甲", &yaml(json!({})));
    let after = task.recorded("2026-09-12", "outline", "走了一步", true); // at / step / detail / ok
    // 端侧：把 after 写回任务文件
    assert!(task.journal.is_empty(), "recorded 拿新值，原任务不动");
    let event = &after.journal[0];
    assert_eq!(
        (
            event.at.as_str(),
            event.step.as_str(),
            event.detail.as_str(),
            event.ok
        ),
        ("2026-09-12", "outline", "走了一步", true)
    );
}

// ---------------------------------------------------------------------------
// workflow.md
// ---------------------------------------------------------------------------

// 文档：workflow.md #1
#[test]
fn doc_workflow_1() {
    use quanttide_work::error::{DefinitionError, Fault, Position};
    use quanttide_work::workflow::{Finding, Step, Workflow, looks_like_section, validate};
    assert!(looks_like_section("收尾"));
    assert!(validate(&yaml(json!({"name": "w", "steps": [{"name": "甲"}]}))).is_ok());
    let error = DefinitionError::new(Position::Top, Fault::MissingName);
    let _: &dyn std::error::Error = &error;
    let _: Option<Finding> = None;
    let step = Step {
        name: "甲".into(),
        description: String::new(),
        executor: AGENT.into(),
        criteria: Vec::new(),
    };
    let workflow = Workflow::of("w.yaml", &yaml(json!({"steps": []})));
    assert_eq!(workflow.steps().len(), 0);
    assert_eq!(step.name(), "甲");
}

// 文档：workflow.md #2
#[test]
fn doc_workflow_2() -> Result<(), DefinitionError> {
    let payload = yaml(json!({
        "name": "code-implement",
        "description": "实现一段代码",
        "steps": [{"name": "大纲", "executor": "agent"}, {"name": "收尾", "executor": "human"}]
    }));
    let workflow = Workflow::from_value(&payload)?; // 不合法当场 Err
    assert_eq!(workflow.step_names(), vec!["大纲", "收尾"]);
    let illegal = Workflow::from_value(&yaml(json!({"name": "w", "steps": []})));
    assert!(illegal.is_err(), "少了 steps 当场 Err");
    Ok(())
}

// 文档：workflow.md #3
#[test]
fn doc_workflow_3() {
    let workflow = Workflow::from_value(&yaml(json!({
        "name": "w",
        "steps": [{
            "name": "甲",
            "criteria": [{"executor": "rule", "path": "/nonexistent-quanttide-work-tests/x.md"}]
        }]
    })))
    .expect("合法定义");
    let context = RunContext::of(&yaml(json!({"data": "/w/data"})));
    let findings = workflow.check(&context.data, |path| std::path::Path::new(path).exists());
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].where_,
        "甲·/nonexistent-quanttide-work-tests/x.md"
    );
    assert!(!findings[0].ok, "这个路径在本机不存在");
}
