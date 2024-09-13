import json
from pathlib import Path

import yarnlock


def test_parse_empty():
    assert yarnlock.yarnlock_parse('') == {}


def test_parse():
    contents = Path('tests/yarn.lock').read_text()
    with Path('tests/yarn.lock.json').open('w') as f:
        json.dump(yarnlock.yarnlock_parse(contents), f, indent=2)
