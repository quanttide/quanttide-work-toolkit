//! 给智能体的两段话：走一步要它做什么、审一遍要它按什么判。
//!
//! 这两段话是产品的一部分——说什么、不说什么是定死的，所以抽出来两侧共用。

/// 这一步的判据清单：每条一行「谁判：说明」。
pub fn criteria_text(criteria: &[Json]) -> String {
    let lines: Vec<String> = criteria
        .iter()
        .map(|criterion| {
            let executor = criterion
                .get("executor")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            format!(
                "- {executor}：{}",
                crate::criteria::description_of(criterion)
            )
        })
        .collect();
    if lines.is_empty() {
        "（这一步没有判据）".to_string()
    } else {
        lines.join("\n")
    }
}

use serde_json::Value as Json;

/// 走一步那件事的现场：任务与这一步的已知事实（路径由各自包算好递进来）。
pub struct Facts {
    pub root: String,
    pub data: String,
    pub name: String,
    pub start: String,
    pub workflow_name: String,
    pub workflow_description: String,
    pub steps: String,
    pub step: String,
    pub what: String,
    pub report: String,
    pub journal: String,
    pub log: String,
    pub artifacts: String,
}

/// 交给 AI 的那一段话。
pub fn prompt_for(facts: &Facts, criteria: &[Json]) -> String {
    format!(
        "你在按一条工作流走一步。只做这一步，做完就停。\n\n\
         工作区：{root}\n\
         数据仓：{data}\n\
         任务：{name}（开工：{start}）\n\
         工作流：{flow}——{description}\n\
         步骤：{steps}\n\
         这一步：{step}\n\
         做什么：\n{what}\n\n\
         判据（程序随后自己核对，你不能改判据、也不许改判据文件）：\n{criteria}\n\n\
         本任务的三样东西（报告与日志是产物，流水是执行痕迹）：\n\
           产物：{report}（程序不碰产物内容，谁写谁定；闸门项记在任务文件里）\n\
           日志：{journal}\n\
           流水：{log}（就在任务文件里）\n\
         工作流里用 {{{{report}}}} / {{{{journal}}}} / {{{{log}}}} 指这三样；工作内容写进报告，别动程序那两节。\n\
         规矩：数据只写数据仓；工作区里只动「做什么」点名的东西。最后用一句话说明你做了什么。\n",
        root = facts.root,
        data = facts.data,
        name = facts.name,
        start = facts.start,
        flow = facts.workflow_name,
        description = facts.workflow_description,
        steps = facts.steps,
        step = facts.step,
        what = facts.what,
        criteria = criteria_text(criteria),
        report = facts.report,
        journal = facts.journal,
        log = facts.log,
    )
}

/// 交给智能体审的那一段话：产物 + 判准，逐条回答。
pub fn judge_prompt(facts: &Facts, criteria: &[Json]) -> String {
    let listed = criteria
        .iter()
        .enumerate()
        .map(|(index, criterion)| {
            format!(
                "{}. {}",
                index + 1,
                criterion
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "你是审查者，不是执行者。别改产物、别改判据文件。\n\n\
         工作区：{root}\n\
         要审的东西：这一步的产物在 {artifacts}（也可以看工作区里相关文件）\n\
         这一步做什么：{what}\n\n\
         判准（逐条判）：\n{listed}\n\n\
         对每条输出一行，格式只能是「序号. 通过 — 一句话理由」或「序号. 不通过 — 一句话理由」，最后不要写别的。\n",
        root = facts.root,
        artifacts = facts.artifacts,
        what = facts.what,
        listed = listed,
    )
}
