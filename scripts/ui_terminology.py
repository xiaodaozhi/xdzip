"""Product-copy policy; archive filenames and technical documentation are outside this gate."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
POLICY = ROOT / 'docs/ux/chinese-terminology.json'


def inspect(messages):
    policy = json.loads(POLICY.read_text(encoding='utf-8'))
    errors = []
    for key, value in messages.items():
        for text in value.values() if isinstance(value, dict) else [value]:
            for noun in policy['forbidden_ui_nouns']:
                inspected = text
                if noun == '压缩文件':
                    for prefix in policy['allowed_compress_verb_prefixes']:
                        inspected = inspected.replace(prefix + noun, '')
                if noun in inspected:
                    errors.append(f'zh-CN/{key}: deprecated container/property term {noun}')
            if text in policy['forbidden_ui_labels']:
                errors.append(f'zh-CN/{key}: use 压缩大小 for the size property')
    for key, label in policy['message_labels'].items():
        if messages.get(key) != policy['labels'][label]:
            errors.append(f'zh-CN/{key}: expected terminology label {label}')
    return errors


def revised_source_label(value):
    """Apply explicit copy-only revisions without touching frozen screenshot measurements."""
    revisions = json.loads(POLICY.read_text(encoding='utf-8'))['historical_label_revisions']
    return [revisions.get(label, label) for label in value] if isinstance(value, list) else revisions.get(value, value)
