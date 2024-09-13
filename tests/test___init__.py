import json
from pathlib import Path

from yarnlock import yarnlock_parse


def test_parse_empty():
    assert yarnlock_parse('') == {}


def test_parse():
    yarnlock = Path('tests/yarn.lock').read_text()
    yarnlock_expected = json.loads(Path('tests/yarn.lock.json').read_bytes())
    assert yarnlock_parse(yarnlock) == yarnlock_expected
