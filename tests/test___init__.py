import json
from pathlib import Path

from yarnlock import yarnlock_parse


def test_parse_empty():
    assert yarnlock_parse('') == {}


def test_parse():
    yarnlock = Path('tests/yarn.lock').read_text()
    yarnlock_expected = json.loads(Path('tests/yarn.lock.json').read_bytes())
    assert yarnlock_parse(yarnlock) == yarnlock_expected

def test_parse_same_package_name_with_different_semantic_versions_resolve_to_different_exact_versions():
    yarnlock = Path('tests/same_package_name_with_different_semantic_versions_resolve_to_different_exact_versions/yarn.lock').read_text()
    yarnlock_expected = json.loads(Path('tests/same_package_name_with_different_semantic_versions_resolve_to_different_exact_versions/yarn.lock.json').read_bytes())
    assert yarnlock_parse(yarnlock) == yarnlock_expected
