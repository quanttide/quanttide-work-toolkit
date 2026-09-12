//! 任务：模型与流水、落点与占位、运行上下文。

use quanttide_work::executor::AGENT;
use quanttide_work::task::{JournalEvent, RunContext, Task, expand_placeholders};
use quanttide_work::workflow::{Step, Workflow};
use serde_json::{Value as Json, json};
use serde_yaml::Value as Yaml;

fn yaml(value: Json) -> Yaml {
    serde_yaml::to_value(value).expect("JSON 装成 YAML 值")
}

/// 只按步骤名搭一条工作流，流水判定够用。
fn workflow(steps: &[&str]) -> Workflow {
    Workflow {
        name: "w".into(),
        description: String::new(),
        steps: steps
            .iter()
            .map(|name| Step {
                name: (*name).into(),
                description: String::new(),
                executor: AGENT.into(),
                criteria: Vec::new(),
            })
            .collect(),
    }
}

/// 用一串流水事件装一件任务。
fn task_with_log(events: Json) -> Task {
    Task::of("甲", &yaml(json!({"log": events})))
}

// ---------------------------------------------------------------------------
// context
// ---------------------------------------------------------------------------

#[test]
fn run_context_reads_and_writes_three_places() {
    let payload = yaml(json!({"root": "/w", "data": "/w/data", "workflows": "/w/workflows"}));
    let context = RunContext::of(&payload);
    assert_eq!(context.root, "/w");
    assert_eq!(context.data, "/w/data");
    assert_eq!(context.workflows, "/w/workflows");
    assert_eq!(RunContext::of(&context.to_yaml()), context);
}

#[test]
fn run_context_defaults_empty() {
    let empty = RunContext::default();
    assert!(empty.root.is_empty() && empty.data.is_empty() && empty.workflows.is_empty());
    assert_eq!(RunContext::of(&yaml(json!({}))), empty);
}

// ---------------------------------------------------------------------------
// journal
// ---------------------------------------------------------------------------

#[test]
fn journal_event_of_defaults_ok_false() {
    let event = JournalEvent::of(&yaml(
        json!({"at": "t1", "step": "甲", "detail": "走了一步"}),
    ));
    assert_eq!(event.at, "t1");
    assert_eq!(event.step, "甲");
    assert_eq!(event.detail, "走了一步");
    assert!(!event.ok, "没写 ok 按 false 算");
    assert_eq!(JournalEvent::of(&event.to_yaml()), event);
}

#[test]
fn done_steps_votes_on_suffixed_events_and_reruns_from_scratch() {
    let task = task_with_log(json!([
        {"at": "t1", "step": "甲", "ok": true},
        {"at": "t2", "step": "甲·审", "ok": false},
        {"at": "t3", "step": "乙", "ok": false},
        {"at": "t4", "step": "乙", "ok": true},
        {"at": "t5", "step": "丙", "ok": true},
        {"at": "t6", "step": "丙·判", "ok": true},
        {"at": "t7", "step": "丁", "ok": true}
    ]));
    let workflow = workflow(&["甲", "乙", "丙"]);
    assert_eq!(
        task.done_steps(&workflow),
        vec!["乙", "丙"],
        "甲被附加判定否掉，乙重跑从头算通过，丙投票仍通过，丁不在工作流"
    );
}

#[test]
fn done_steps_keeps_the_journal_order() {
    let task = task_with_log(json!([
        {"at": "t1", "step": "丙", "ok": true},
        {"at": "t2", "step": "甲", "ok": true}
    ]));
    assert_eq!(task.done_steps(&workflow(&["甲", "丙"])), vec!["丙", "甲"]);
}

#[test]
fn next_step_is_the_first_not_done() {
    let task = task_with_log(json!([{"at": "t", "step": "甲", "ok": true}]));
    assert_eq!(task.next_step(&workflow(&["甲", "乙"])), Some("乙".into()));
    assert_eq!(task.next_step(&workflow(&["甲"])), None);
    assert_eq!(task.next_step(&workflow(&[])), None);
}

#[test]
fn state_line_names_the_next_or_the_finish() {
    let task = Task::of("甲", &yaml(json!({"workflow": "w"})));
    let empty = Workflow {
        name: "w".into(),
        description: String::new(),
        steps: Vec::new(),
    };
    assert_eq!(
        task.state_line(&empty),
        "这条工作流没有步骤——在 workflows/w.yaml 的 steps 里写步骤"
    );

    let workflow = workflow(&["甲", "乙"]);
    assert_eq!(task.state_line(&workflow), "下一步：甲");
    let finished = task_with_log(json!([
        {"at": "t1", "step": "甲", "ok": true},
        {"at": "t2", "step": "乙", "ok": true}
    ]));
    assert_eq!(finished.state_line(&workflow), "2 个步骤都走过了");
}

// ---------------------------------------------------------------------------
// task::model
// ---------------------------------------------------------------------------

#[test]
fn task_of_reads_journal_gates_artifacts_and_context() {
    let payload = yaml(json!({
        "workflow": "code-implement",
        "start": "outline",
        "root": "/w",
        "data": "/w/data",
        "workflows": "/w/workflows",
        "log": [{"at": "t1", "step": "大纲", "detail": "走了", "ok": true}],
        "gates": ["等老板拍板", 3],
        "artifacts": {"report": "data/report/甲.md", "bad": 7}
    }));
    let task = Task::of("甲", &payload);
    assert_eq!(task.name, "甲");
    assert_eq!(task.workflow_name, "code-implement");
    assert_eq!(task.start, "outline");
    assert_eq!(task.context.data, "/w/data");
    assert_eq!(
        task.journal,
        vec![JournalEvent {
            at: "t1".into(),
            step: "大纲".into(),
            detail: "走了".into(),
            ok: true,
        }]
    );
    assert_eq!(task.gates, vec!["等老板拍板"], "非字符串的闸门项丢掉");
    assert_eq!(task.artifacts.len(), 1, "非字符串的落点丢掉");
}

#[test]
fn task_of_treats_missing_or_wrong_blocks_as_empty() {
    let task = Task::of(
        "甲",
        &yaml(json!({"log": "不是列表", "gates": "不是列表", "artifacts": "不是映射"})),
    );
    assert!(task.journal.is_empty());
    assert!(task.gates.is_empty());
    assert!(task.artifacts.is_empty());
    assert_eq!(task.workflow_name, "");
    assert_eq!(task.start, "");
}

#[test]
fn task_to_yaml_roundtrips() {
    let payload = yaml(json!({
        "workflow": "code-implement",
        "start": "outline",
        "root": "/w",
        "data": "/w/data",
        "workflows": "/w/workflows",
        "log": [{"at": "t1", "step": "大纲", "detail": "走了", "ok": true}],
        "gates": ["等老板拍板"],
        "artifacts": {"report": "data/report/甲.md"}
    }));
    let task = Task::of("甲", &payload);
    assert_eq!(Task::of("甲", &task.to_yaml()), task);
}

#[test]
fn declared_trims_and_ignores_blank() {
    let task = Task::of(
        "甲",
        &yaml(json!({"artifacts": {"report": "  data/甲.md  ", "blank": "   "}})),
    );
    assert_eq!(task.declared("report"), Some("data/甲.md".into()));
    assert_eq!(task.declared("blank"), None, "声明成空白等于没声明");
    assert_eq!(task.declared("missing"), None);
}

#[test]
fn artifact_trims_trailing_slashes_and_follows_the_rules() {
    let context = RunContext {
        root: "/w/".into(),
        data: "/d/".into(),
        workflows: "/w/workflows".into(),
    };
    let task = Task::of("甲", &yaml(json!({})));
    assert_eq!(
        task.artifact("report", &context),
        "/d/artifacts/report/甲.md"
    );
    assert_eq!(task.artifact("log", &context), "/d/tasks/甲.yaml");

    let declared = Task::of(
        "甲",
        &yaml(json!({"artifacts": {"report": "drafts/甲.md", "logs": "/elsewhere/甲.md"}})),
    );
    assert_eq!(
        declared.artifact("report", &context),
        "/w/drafts/甲.md",
        "相对工作区根"
    );
    assert_eq!(
        declared.artifact("logs", &context),
        "/elsewhere/甲.md",
        "绝对路径原样"
    );
}

#[test]
fn recorded_hands_back_a_new_task_and_leaves_the_origin_alone() {
    let task = Task::of("甲", &yaml(json!({})));
    let after = task.recorded("2026-09-12", "outline", "走了一步", true);
    assert!(task.journal.is_empty(), "原标题不变");
    assert_eq!(
        after.journal,
        vec![JournalEvent {
            at: "2026-09-12".into(),
            step: "outline".into(),
            detail: "走了一步".into(),
            ok: true,
        }]
    );
}

#[test]
fn with_gates_skips_duplicates() {
    let task = Task::of("甲", &yaml(json!({"gates": ["甲项"]})));
    let after = task.with_gates(&["甲项".into(), "乙项".into()]);
    assert_eq!(after.gates, vec!["甲项", "乙项"]);
    assert_eq!(task.gates, vec!["甲项"], "原标题不变");
}

#[test]
fn expand_placeholders_knows_four_and_leaves_the_rest() {
    assert_eq!(
        expand_placeholders("{{report}}/清单.md", "/d"),
        "/d/artifacts/report/清单.md"
    );
    assert_eq!(
        expand_placeholders("{{journal}}", "/d"),
        "/d/artifacts/journal"
    );
    assert_eq!(expand_placeholders("{{log}}", "/d"), "/d/tasks");
    assert_eq!(
        expand_placeholders("{{artifacts}}/y", "/d"),
        "/d/artifacts/y"
    );
    assert_eq!(expand_placeholders("没有占位", "/d"), "没有占位");
    assert_eq!(expand_placeholders("{{unknown}}", "/d"), "{{unknown}}");
}

/// 三处位置随任务写回顶层——`to_yaml` 之后仍然读得到。
#[test]
fn to_yaml_writes_the_run_context_back_at_top_level() {
    let payload = yaml(json!({
        "workflow": "w",
        "root": "/r",
        "data": "/d",
        "workflows": "/wf"
    }));
    let written = Task::of("甲", &payload).to_yaml();
    let map = written.as_mapping().expect("任务写成映射");
    for (key, want) in [("root", "/r"), ("data", "/d"), ("workflows", "/wf")] {
        assert_eq!(
            map.get(Yaml::String(key.into())).and_then(Yaml::as_str),
            Some(want),
            "三处位置要原样写回：{key}"
        );
    }
}
