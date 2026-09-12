//! 产物：名字是它的身份，规格是它凭以通过验收的那组判据。

use quanttide_work::artifact::Artifact;
use quanttide_work::criterion::Criterion;

#[test]
fn named_carries_the_name_and_has_no_spec_yet() {
    let artifact = Artifact::named("report");
    assert_eq!(artifact.name, "report");
    assert!(artifact.spec.is_empty(), "只知其名时还没有规格");
}

#[test]
fn of_carries_the_spec() {
    let criterion = Criterion::PathExists {
        path: "artifacts/report/甲.md".into(),
        description: String::new(),
    };
    let artifact = Artifact::of("report", vec![criterion.clone()]);
    assert_eq!(artifact.name, "report");
    assert_eq!(artifact.spec, vec![criterion], "规格就是验收它的那组判据");
}

#[test]
fn two_artifacts_are_the_same_when_the_name_and_the_spec_are() {
    assert_eq!(
        Artifact::named("report"),
        Artifact::of("report", Vec::new())
    );
}
