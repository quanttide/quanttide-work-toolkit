//! 工作区：装载内容、定义核对、落点、流水判定。

use quanttide_work::executor::AGENT;
use quanttide_work::task::Task;
use quanttide_work::workflow::{Step, Workflow};
use quanttide_work::workspace::{Finding, Workspace, looks_like_section};
use serde_json::{Value as Json, json};
use serde_yaml::Value as Yaml;

fn yaml(value: Json) -> Yaml {
    serde_yaml::to_value(value).expect("JSON 装成 YAML 值")
}

/// 装一件任务：把名字并进 payload（任务名是文件里的 `name` 字段）。
fn task_of(name: &str, payload: Json) -> Task {
    let mut map = payload.as_object().cloned().unwrap_or_default();
    map.insert("name".to_string(), Json::String(name.to_string()));
    Task::of(&yaml(Json::Object(map)))
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

/// 用一串流水事件装一件任务（跑工作流 `w`）。
fn task_with_log(events: Json) -> Task {
    task_of("甲", json!({"workflow": "w", "log": events}))
}

/// 只装一条工作流 `w` 的工作区。
fn workspace(steps: &[&str]) -> Workspace {
    Workspace::of(vec![workflow(steps)], Vec::new())
}

/// 核对结果里 description 提到的小节名有没有被覆盖。
fn section_ok(findings: &[Finding], name: &str) -> Option<bool> {
    findings
        .iter()
        .find(|finding| finding.what.ends_with(name))
        .and_then(|finding| finding.ok)
}

// ---------------------------------------------------------------------------
// workspace::model
// ---------------------------------------------------------------------------

#[test]
fn workspace_holds_workflows_and_tasks() {
    let task = task_of("甲", json!({}));
    let workspace = Workspace::of(vec![workflow(&["甲", "乙"])], vec![task.clone()]);
    assert_eq!(
        workspace.workflow("w").map(|flow| flow.step_names()),
        Some(vec!["甲".to_string(), "乙".to_string()])
    );
    assert_eq!(workspace.task("甲"), Some(&task));
    assert!(workspace.workflow("没有").is_none());
    assert!(workspace.task("没有").is_none());
    assert_eq!(Workspace::default().workflows.len(), 0);
}

// ---------------------------------------------------------------------------
// workspace::check
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

    let findings = Workspace::default().check(&workflow, |path| path == "/w/seen.md");

    let path_finding = findings
        .iter()
        .find(|finding| finding.what.starts_with("判据里的路径在不在"))
        .expect("路径判据应当出一条核对");
    assert_eq!(path_finding.where_, "甲·/w/seen.md");
    assert_eq!(path_finding.what, "判据里的路径在不在：/w/seen.md");
    assert_eq!(path_finding.ok, Some(true));

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
fn check_marks_runtime_placeholders_unchecked() {
    let workflow = Workflow::from_value(&yaml(json!({
        "name": "w",
        "steps": [{
            "name": "甲",
            "criteria": [
                {"executor": "rule", "path": "{{report}}/x.md"},
                {"executor": "rule", "path": "{{journal}}/y.md"},
                {"executor": "rule", "path": "{{log}}/z.yaml"},
                {"executor": "rule", "path": "{{artifacts}}/w.md"},
                {"executor": "rule", "absent": "gone.md"},
                {"executor": "rule", "run": "true"},
                {"executor": "agent", "description": "写干净了"},
                {"executor": "human", "description": "人拍板"}
            ]
        }]
    })))
    .expect("合法定义");

    let findings = Workspace::default().check(&workflow, |_| true);
    assert_eq!(
        findings.len(),
        4,
        "四个运行时占位各出一条「未核」；不查的判法不出核对项：{findings:?}"
    );
    assert!(
        findings.iter().all(|finding| finding.ok.is_none()),
        "运行时占位一律「未核」"
    );
    assert!(
        findings.iter().all(|finding| finding.what.contains("未核")),
        "「未核」要写在回执里，不静默丢掉：{findings:?}"
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

    let findings = Workspace::default().check(&workflow, |_| true);
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

// ---------------------------------------------------------------------------
// workspace::artifact
// ---------------------------------------------------------------------------

#[test]
fn artifact_follows_the_declaration_and_the_default() {
    let workspace = Workspace::default();
    let task = task_of("甲", json!({}));
    assert_eq!(
        workspace.artifact(&task, "report", "/d/"),
        "/d/artifacts/report/甲.md",
        "没声明落默认处，目录尾斜杠忽略"
    );
    assert_eq!(
        workspace.artifact(&task, "log", "/d/"),
        "/d/tasks/甲.yaml",
        "流水是任务文件本身"
    );

    let declared = task_of(
        "甲",
        json!({"artifacts": {"report": "report/甲.md", "logs": "/elsewhere/甲.md"}}),
    );
    assert_eq!(
        workspace.artifact(&declared, "report", "/d"),
        "/d/report/甲.md",
        "声明了相对平台给的目录基准"
    );
    assert_eq!(
        workspace.artifact(&declared, "logs", "/d"),
        "/elsewhere/甲.md",
        "绝对路径原样"
    );
}

#[test]
fn placeholders_point_at_the_same_landings_as_artifact() {
    let workspace = Workspace::default();
    let task = task_of("甲", json!({}));
    let tab = workspace.placeholders(&task, "/d");

    assert_eq!(tab.path_of("report"), Some("/d/artifacts/report/甲.md"));
    assert_eq!(tab.path_of("journal"), Some("/d/artifacts/journal/甲.md"));
    assert_eq!(tab.path_of("log"), Some("/d/tasks/甲.yaml"));
    assert_eq!(tab.path_of("artifacts"), Some("/d/artifacts"));
    assert_eq!(tab.path_of("foo"), None, "不认识的占位不给路径");

    assert_eq!(
        tab.expand("{{report}} 里写 {{artifacts}} 的清单"),
        "/d/artifacts/report/甲.md 里写 /d/artifacts 的清单"
    );
    assert_eq!(tab.expand("没有占位"), "没有占位");
    assert_eq!(tab.expand("{{name}}"), "{{name}}", "不认识的占位原样留着");
    assert_eq!(
        tab.expand("没闭合的 {{report"),
        "没闭合的 {{report",
        "没闭合的占位不吞后面的字"
    );
}

#[test]
fn placeholders_follow_the_declaration() {
    let workspace = Workspace::default();
    let task = task_of("甲", json!({"artifacts": {"report": "report/甲.md"}}));
    let tab = workspace.placeholders(&task, "/d");
    assert_eq!(
        tab.expand("{{report}}"),
        "/d/report/甲.md",
        "声明了报告往哪写，占位就换成它"
    );
    assert_eq!(
        tab.expand("{{journal}}"),
        "/d/artifacts/journal/甲.md",
        "没声明的仍落默认处"
    );
}

// ---------------------------------------------------------------------------
// workspace::progress
// ---------------------------------------------------------------------------

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
    assert_eq!(
        workspace(&["甲", "乙", "丙"]).done_steps(&task),
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
    assert_eq!(workspace(&["甲", "丙"]).done_steps(&task), vec!["丙", "甲"]);
}

#[test]
fn next_step_is_the_first_not_done() {
    let task = task_with_log(json!([{"at": "t", "step": "甲", "ok": true}]));
    assert_eq!(workspace(&["甲", "乙"]).next_step(&task), Some("乙".into()));
    assert_eq!(workspace(&["甲"]).next_step(&task), None);
    assert_eq!(workspace(&[]).next_step(&task), None);
    assert_eq!(
        Workspace::default().next_step(&task),
        None,
        "工作区里没有这条工作流"
    );
}

#[test]
fn state_line_names_the_next_or_the_finish() {
    let task = task_of("甲", json!({"workflow": "w"}));
    let empty = Workspace::of(
        vec![Workflow {
            name: "w".into(),
            description: String::new(),
            steps: Vec::new(),
        }],
        Vec::new(),
    );
    assert_eq!(empty.state_line(&task), "这条工作流没有步骤：w");

    assert_eq!(workspace(&["甲", "乙"]).state_line(&task), "下一步：甲");
    let finished = task_with_log(json!([
        {"at": "t1", "step": "甲", "ok": true},
        {"at": "t2", "step": "乙", "ok": true}
    ]));
    assert_eq!(
        workspace(&["甲", "乙"]).state_line(&finished),
        "2 个步骤都走过了"
    );
    assert_eq!(
        Workspace::default().state_line(&task),
        "工作区里没有这条工作流：w"
    );
}
