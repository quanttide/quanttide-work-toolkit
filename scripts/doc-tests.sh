#!/usr/bin/env bash
# 文档测试：user-guide 里的每条示例，都要在测试里有对应、且真跑过。
#
# 查两件事：
#   1. 文档里每段 ```rust / ```dart 示例，都要在本语言的测试文件里找得到——文档改了、
#      测试没跟，这里就红。比法是「按词」：忽略注释、空白与花括号，因为 rustfmt 会把
#      `use {...}` 里的名字重排、示例的一行在测试里也可能展开成多行。
#      `use` / `import` 行不比整行，只要求里面出现的名字在测试文件里有。
#   2. 文档里每段 ```bash 示例，直接执行——它本身就是门禁命令。
#
# 用法（在工具箱根目录）：sh scripts/doc-tests.sh
root="$(cd "$(dirname "$0")/.." && pwd)"
python3 - "$root" <<'PY'
import os, re, subprocess, sys

root = sys.argv[1]
docs_dir = os.path.join(root, "docs/user-guide")
targets = {
    "rust": os.path.join(root, "packages/rust/tests/user_guide.rs"),
    "dart": os.path.join(root, "packages/dart/test/user_guide_test.dart"),
}

TOKEN = re.compile(r'"[^"]*"|\'[^\']*\'|[A-Za-z_][A-Za-z0-9_]*|\d+|[^\s]')

def tokens(line: str):
    """一行代码 → 词序列：去注释，丢掉花括号与分号（换行/嵌套差异不算差异）。"""
    line = re.sub(r"//.*$", "", line)
    return [t for t in TOKEN.findall(line) if t not in {"{", "}", ";"}]

def code_text(path: str):
    return open(path, encoding="utf-8").read()

missing = []
reported = set()
counted = {"rust": 0, "dart": 0, "bash": 0}

for fn in sorted(f for f in os.listdir(docs_dir) if f.endswith(".md")):
    text = code_text(os.path.join(docs_dir, fn))
    for lang, body in re.findall(r"```(rust|dart|bash)\n(.*?)```", text, re.S):
        counted[lang] += 1
        if lang == "bash":
            lines = [re.sub(r"//.*$", "", l).strip() for l in body.splitlines()]
            script = next((l for l in lines if l), "")
            # 只跑工具箱自己的脚本，别的（端侧的）不在这一层管
            if script.startswith("sh scripts/"):
                if subprocess.run(script, shell=True, cwd=root).returncode != 0:
                    missing.append(f"{fn}: bash 示例跑不过 → {script}")
            continue

        target = targets[lang]
        if not os.path.exists(target):
            if lang not in reported:          # 缺文件只报一次
                missing.append(f"缺 {os.path.relpath(target, root)}（{lang} 示例没有测试兜）")
                reported.add(lang)
            continue

        raw = code_text(target)
        parts = []
        for line in raw.splitlines():
            parts.extend(tokens(line))
        haystack = " ".join(parts)
        for line in body.splitlines():
            stripped = re.sub(r"//.*$", "", line).strip()
            if not stripped:
                continue
            if stripped.startswith(("use ", "import ")):
                # 只要求名字都在（顺序与写法由各语言的格式化定）
                for name in re.findall(r"\{([^}]*)\}|(?:::)([A-Za-z_][A-Za-z0-9_]*)", stripped):
                    for one in (name[0] or name[1]).split(","):
                        one = one.strip()
                        if one and one not in raw:
                            missing.append(f"{fn} [{lang}]: 名字没在测试里 → {one}")
                continue
            need = " ".join(tokens(line))
            if need and need not in haystack:
                missing.append(f"{fn} [{lang}]: 测试里找不到 → {stripped}")

total = sum(counted.values())
print(f"文档示例：rust {counted['rust']} 条、dart {counted['dart']} 条、bash {counted['bash']} 条，共 {total} 条")
if missing:
    print(f"\n没兜住的 {len(missing)} 处：")
    for m in missing:
        print("  ✗ " + m)
    sys.exit(1)
print("全部示例都有测试兜住（文档与测试对得上）")
PY
