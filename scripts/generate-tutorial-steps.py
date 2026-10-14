"""Copy the established Kotlin tutorial enum/text, never a Wiki task source."""
from pathlib import Path
import hashlib
import json
import re

root = Path(__file__).resolve().parents[1]
source = root / 'reference/kotlin/content/areas/tutorial-island/src/main/kotlin/org/rsmod/content/areas/tutorialisland/progression/TutorialSteps.kt'
text = source.read_text(encoding='utf-8')
entries = re.findall(r'^    (\w+)\((\d+)\)[,;]$', text, re.M)
assert len(entries) == 54 and [int(v) for _, v in entries] == list(range(54))
names = [name for name, _ in entries]
sections = text.split('    val guideTitle: String', 1)[1].split('    val guideBody: String', 1)
sections[1] = sections[1].split('    companion object', 1)[0]
maps = []
for section in sections:
    values = {}
    for labels, literal in re.findall(r'([A-Za-z]+(?:,\s*[A-Za-z]+)*)\s*->\s*("(?:\\.|[^"\\])*")', section):
        for label in re.split(r',\s*', labels):
            assert label not in values
            values[label] = json.loads(literal)
    assert set(values) == set(names), set(names) - set(values)
    maps.append(values)

lines = [
    '//! Original TutorialSteps.kt metadata; regenerate with scripts/generate-tutorial-steps.py.',
    f'//! Kotlin source SHA256: {hashlib.sha256(source.read_bytes()).hexdigest()}',
    '#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]',
    '#[repr(i32)]', 'pub enum Step {',
    *[f'    {name} = {value},' for name, value in entries], '}', 'impl Step {',
    '    pub const MAX_PROGRESS: i32 = 52;',
    '    pub const ALL: [Self; 54] = [',
    *[f'        Self::{name},' for name in names], '    ];',
    '    /// enumVarp defaults unknown stored values to the zero-valued member.',
    '    pub fn from_var(value: i32) -> Self {',
    '        usize::try_from(value).ok().and_then(|i| Self::ALL.get(i)).copied().unwrap_or(Self::CharCreate)',
    '    }',
    '    pub fn name(self) -> &\'static str { match self {',
    *[f'        Self::{name} => "{name}",' for name in names], '    }}',
]
for function, values in zip(('title', 'body'), maps):
    lines += [f'    pub fn {function}(self) -> &\'static str {{ match self {{']
    lines += [f'        Self::{name} => {json.dumps(values[name], ensure_ascii=False)},' for name in names]
    lines += ['    }}']
lines += [
    '    pub fn guide_html(self) -> String { format!("<col=0000ff>{}</col><br>{}", self.title(), self.body()) }',
    '    pub fn progress_percent(self) -> i32 { (self as i32) * 100 / Self::MAX_PROGRESS }',
    '}',
]
(root / 'crates/world/src/tutorial_steps.rs').write_text('\n'.join(lines) + '\n', encoding='utf-8')
