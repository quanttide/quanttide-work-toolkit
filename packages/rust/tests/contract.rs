//! 契约：同一批用例向量（`../contract/*.json`），两侧各跑一遍，结论必须一样。
//!
//! 向量是两侧**共用**的同一批文件——契约写在向量里，不写在各自的测试代码里。

use quanttide_work::{criteria, definition, envelope, tasklog};
use serde_json::{Value, json};
use std::fs;

/// 向量文件是 JSON（夹具），工具箱的模型吃 YAML 值——JSON 本来就是合法的 YAML。
fn as_yaml(value: &Value) -> serde_yaml::Value {
    serde_yaml::to_value(value).unwrap_or(serde_yaml::Value::Null)
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

#[test]
fn contract() {
    let vectors = vectors();
    for (name, vector) in vectors.iter() {
        let kind = vector["kind"].as_str().unwrap_or("");
        match kind {
            "validate" => {
                let file = vector["file"].as_str().unwrap_or("");
                let got = definition::validate(&as_yaml(&vector["input"]), file);
                match vector["expect"].get("error") {
                    Some(Value::String(wanted)) => match got {
                        Ok(()) => panic!("{name}：期望报错，却通过了"),
                        Err(error) => assert_eq!(&error.0, wanted, "{name}：报错文字不一样"),
                    },
                    _ => {
                        if let Err(error) = got {
                            panic!("{name}：期望通过，却报错：{}", error.0);
                        }
                    }
                }
            }
            "items" => {
                let criteria: Vec<Value> = vector["input"].as_array().cloned().unwrap_or_default();
                let criteria: Vec<serde_yaml::Value> = criteria.iter().map(as_yaml).collect();
                let got: Vec<Value> = criteria::items_of(&criteria)
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
                let steps: Vec<String> = vector["steps"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default()
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                let events: Vec<Value> = vector["events"].as_array().cloned().unwrap_or_default();
                let events: Vec<serde_yaml::Value> = events.iter().map(as_yaml).collect();
                let got = json!(tasklog::done(&steps, &events));
                assert_eq!(got, vector["expect"], "{name}：走过哪几步不一样");
            }
            "section" => {
                for case in vector["cases"].as_array().cloned().unwrap_or_default() {
                    let input = case["input"].as_str().unwrap_or("");
                    let got = definition::looks_like_section(input);
                    assert_eq!(json!(got), case["expect"], "{name}：{input} 算不算小节名");
                }
            }
            "expand" => {
                let data = vector["data"].as_str().unwrap_or("");
                for case in vector["cases"].as_array().cloned().unwrap_or_default() {
                    let input = case["input"].as_str().unwrap_or("");
                    let got = definition::expand_placeholders(input, data);
                    assert_eq!(json!(got), case["expect"], "{name}：{input} 展开得不对");
                }
            }
            "envelope" => {
                let input = &vector["input"];
                let outcome = envelope::Outcome {
                    ok: input["ok"].as_bool().unwrap_or(false),
                    lines: string_list(&input["lines"]),
                    columns: string_list(&input["columns"]),
                    rows: input["rows"]
                        .as_array()
                        .cloned()
                        .unwrap_or_default()
                        .iter()
                        .map(string_list)
                        .collect(),
                    payload: None,
                };
                assert_eq!(
                    outcome.to_json(),
                    vector["expect"],
                    "{name}：信封的 JSON 不一样"
                );
            }
            other => panic!("{name}：不认得的向量类型 {other}"),
        }
    }
    assert!(vectors.len() >= 10, "向量太少：{}", vectors.len());
    println!("契约：{} 份向量，两侧一致", vectors.len());
}

fn string_list(value: &Value) -> Vec<String> {
    value
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|item| item.as_str().unwrap_or("").to_string())
        .collect()
}
