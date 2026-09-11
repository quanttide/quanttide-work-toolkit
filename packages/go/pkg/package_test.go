package quanttide_work_test

import (
	"testing"

	quanttide_work "github.com/quanttide/quanttide-work-toolkit/packages/go/pkg"
)

func TestDomain(t *testing.T) {
	if quanttide_work.Domain != "knowledge-work" {
		t.Fatalf("领域英文名不符：%s", quanttide_work.Domain)
	}
}

func TestVersion(t *testing.T) {
	if quanttide_work.Version != "0.1.0" {
		t.Fatalf("包版本不符：%s", quanttide_work.Version)
	}
}
