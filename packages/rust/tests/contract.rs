//! 契约：同一批用例向量（`../contract/*.json`），两侧各跑一遍，结论必须一样。
//!
//! 向量是两侧**共用**的同一批文件——契约写在向量里，不写在各自的测试代码里。

use quanttide_work::executor::AGENT;
use quanttide_work::workflow::Step;
use quanttide_work::{criterion, outcome, paths, task, workflow};
use serde_json::{Value, json};
use serde_yaml::{Mapping, Value as Yaml};
use std::fs;

/// 向量文件是 JSON（夹具），工具箱的模型吃 YAML 值——JSON 本来就是合法的 YAML。
fn as_yaml(value: &Value) -> Yaml {
    serde_yaml::to_value(value).unwrap_or(Yaml::Null)
}

fn vectors() -> Vec<(String, Value)> {
    let dir = std::path::Path::new("../../tests/contract");
    let mut found: Vec<(String, Value)> = fs::read_dir(dir)
        .expect("找不到 contract 目录")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().map(|e| e == "json").unwrap_or(false))
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let text = fs::read_to_string(&path).expect("向量读不了");
            (
                name,
                serde_json::from_str(&text).expect("向量不是合法 JSON"),
            )
        })
        .collect();
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found
}

/// 向量里的步骤名串成一条工作流（判出「走过哪几步」够用）。
fn workflow_of(steps: &Value) -> workflow::Workflow {
    workflow::Workflow {
        name: "v".to_string(),
        description: String::new(),
        steps: steps
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|value| value.as_str())
            .map(|name| Step {
                name: name.to_string(),
                description: String::new(),
                executor: AGENT.to_string(),
                criteria: Vec::new(),
            })
            .collect(),
    }
}

/// 向量里的流水装成一件任务。
fn task_of(events: &Value) -> task::Task {
    let mut payload = Mapping::new();
    payload.insert(
        Yaml::String("log".into()),
        Yaml::Sequence(
            events
                .as_array()
                .cloned()
                .unwrap_or_default()
                .iter()
                .map(as_yaml)
                .collect(),
        ),
    );
    task::Task::of("v", &Yaml::Mapping(payload))
}

#[test]
fn contract() {
    let vectors = vectors();
    for (name, vector) in vectors.iter() {
        let kind = vector["kind"].as_str().unwrap_or("");
        match kind {
            "validate" => {
                let file = vector["file"].as_str().unwrap_or("");
                let got = workflow::validate(&as_yaml(&vector["input"]));
                match vector["expect"].get("error") {
                    Some(Value::String(wanted)) => match got {
                        Ok(()) => panic!("{name}：期望报错，却通过了"),
                        Err(error) => {
                            assert_eq!(error.message(file), *wanted, "{name}：报错文字不一样")
                        }
                    },
                    _ => {
                        if let Err(error) = got {
                            panic!("{name}：期望通过，却报错：{}", error.message(file));
                        }
                    }
                }
            }
            "items" => {
                let criteria: Vec<Value> = vector["input"].as_array().cloned().unwrap_or_default();
                let criteria: Vec<criterion::Criterion> = criteria
                    .iter()
                    .map(|value| criterion::criterion_of(&as_yaml(value)))
                    .collect();
                let got: Vec<Value> = criterion::items_of(&criteria)
                    .into_iter()
                    .map(|item| {
                        json!({
                            "description": item.description,
                            "kind": item.kind.map(|kind| format!("{kind:?}").to_lowercase()),
                            "args": item.args,
                        })
                    })
                    .collect();
                assert_eq!(
                    Value::Array(got),
                    vector["expect"],
                    "{name}：判据翻出来的不一样"
                );
            }
            "done" => {
                let got = task_of(&vector["events"]).done_steps(&workflow_of(&vector["steps"]));
                assert_eq!(json!(got), vector["expect"], "{name}：走过哪几步不一样");
            }
            "section" => {
                for case in vector["cases"].as_array().cloned().unwrap_or_default() {
                    let input = case["input"].as_str().unwrap_or("");
                    let got = workflow::looks_like_section(input);
                    assert_eq!(json!(got), case["expect"], "{name}：{input} 算不算小节名");
                }
            }
            "artifact" => {
                // 每一格可以自带 root / data（换目录的那几格）；不写就按向量顶层的。
                let raw_root = vector["root"].as_str().unwrap_or("").to_string();
                let raw_data = vector["data"].as_str().unwrap_or("").to_string();
                for case in vector["cases"].as_array().cloned().unwrap_or_default() {
                    let context = task::RunContext {
                        root: case["root"].as_str().unwrap_or(&raw_root).to_string(),
                        data: case["data"].as_str().unwrap_or(&raw_data).to_string(),
                        workflows: String::new(),
                    };
                    let mut payload = Mapping::new();
                    payload.insert(
                        Yaml::String("artifacts".into()),
                        as_yaml(&case["artifacts"]),
                    );
                    let task = task::Task::of(
                        case["name"].as_str().unwrap_or(""),
                        &Yaml::Mapping(payload),
                    );
                    let kind = case["artifact"].as_str().unwrap_or("");
                    let note = case["note"].as_str().unwrap_or("");
                    assert_eq!(
                        task.artifact(kind, &context),
                        case["expect"].as_str().unwrap_or(""),
                        "{name}：{note} 落点算得不对"
                    );
                }
            }
            "outcome" => {
                for case in vector["cases"].as_array().cloned().unwrap_or_default() {
                    let got = outcome::Outcome::from_json(&case["input"]).to_json();
                    let note = case["note"].as_str().unwrap_or("");
                    assert_eq!(got, case["expect"], "{name}：{note} 编解码不一样");
                }
            }
            "expand" => {
                let fallback = vector["data"].as_str().unwrap_or("");
                for case in vector["cases"].as_array().cloned().unwrap_or_default() {
                    let input = case["input"].as_str().unwrap_or("");
                    // 每一格可以自带 data（换目录的那几格）；不写就按向量顶层的。
                    let data = case["data"].as_str().unwrap_or(fallback);
                    let got = paths::expand_placeholders(input, data);
                    assert_eq!(json!(got), case["expect"], "{name}：{input} 展开得不对");
                }
            }
            other => panic!("{name}：不认得的向量类型 {other}"),
        }
    }
    assert!(vectors.len() >= 11, "向量太少：{}", vectors.len());
    println!("契约：{} 份向量，两侧一致", vectors.len());
}
