//! 产物聚合 / 模型：一件产出物的名字与规格。
//!
//! 名字是它的身份，产物不限种类——报告、日志只是两个名字。规格是一组验收判据，
//! 产物的质量由规格控制：判据核过了，产物才算通过验收。
//! 产物不带位置——位置不进模型，它落在哪由工作区按名字算（`Workspace::place`）。
//! 出处：`docs/specification/piece/artifact.md`。

use crate::criterion::Criterion;

/// 产物聚合。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    /// 产物的名字，也是它的身份。
    pub name: String,
    /// 规格：验收这组判据。
    pub spec: Vec<Criterion>,
}

impl Artifact {
    /// 只知其名：还没有规格的产物。
    pub fn named(name: &str) -> Artifact {
        Artifact {
            name: name.to_string(),
            spec: Vec::new(),
        }
    }

    /// 名字加规格。
    pub fn of(name: &str, spec: Vec<Criterion>) -> Artifact {
        Artifact {
            name: name.to_string(),
            spec,
        }
    }
}
