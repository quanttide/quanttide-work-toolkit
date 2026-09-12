import tomllib
from pathlib import Path

from quanttide_work import DOMAIN, __version__


# 版本常量与清单同源：这里对着 pyproject.toml 核一遍，错开就红。
def test_version():
    pyproject = Path(__file__).resolve().parents[1] / "pyproject.toml"
    declared = tomllib.loads(pyproject.read_text(encoding="utf-8"))["project"]["version"]
    assert __version__ == declared


def test_domain():
    assert DOMAIN == "knowledge-work"
