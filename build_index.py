#!/usr/bin/env python3
"""
build_index.py — regenerates INDEX.md for the arma3-wiki repo.

Run from anywhere:
    python build_index.py
    python E:/Storage/Arma3Data/arma3-wiki/build_index.py

Outputs: INDEX.md next to this script.
"""

import os
import glob
import urllib.parse
from collections import defaultdict

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
COMMANDS_DIR = os.path.join(SCRIPT_DIR, "commands")
EVENTS_DIR = os.path.join(SCRIPT_DIR, "events")
OUTPUT = os.path.join(SCRIPT_DIR, "INDEX.md")


def decode_filename(name: str) -> str:
    """Turn a URL-encoded filename (minus .yml) into a display name."""
    return urllib.parse.unquote(name)


def parse_simple_yaml_field(lines: list[str], field: str) -> str | None:
    """Extract a simple single-line field value from raw YAML lines."""
    prefix = field + ":"
    for line in lines:
        if line.startswith(prefix):
            return line[len(prefix):].strip()
    return None


def parse_groups(lines: list[str]) -> list[str]:
    """Extract the groups list from raw YAML lines (stops at next top-level key)."""
    groups = []
    in_groups = False
    for line in lines:
        if line.startswith("groups:"):
            in_groups = True
            continue
        if in_groups:
            stripped = line.rstrip()
            if stripped.startswith("- ") and not stripped.startswith("- call"):
                groups.append(stripped[2:].strip())
            elif stripped and not stripped.startswith(" ") and not stripped.startswith("\t"):
                break  # hit next top-level key
    return groups


def parse_first_sentence(text: str) -> str:
    """Return the first sentence of a description, stripped of wiki markup."""
    if not text:
        return ""
    # Strip wiki template markup like {{Feature|...}}
    import re
    text = re.sub(r"\{\{[^}]*\}\}", "", text)
    # Strip wiki links [[foo|bar]] -> bar, [[foo]] -> foo
    text = re.sub(r"\[\[([^\]|]+)\|([^\]]+)\]\]", r"\2", text)
    text = re.sub(r"\[\[([^\]]+)\]\]", r"\1", text)
    # Strip HTML tags
    text = re.sub(r"<[^>]+>", "", text)
    # Take first sentence
    sentence = re.split(r"(?<=[.!?])\s", text.strip())[0]
    return sentence.strip()


# ── Commands ──────────────────────────────────────────────────────────────────

print(f"Scanning {COMMANDS_DIR} ...")
group_to_commands: dict[str, list[tuple[str, str]]] = defaultdict(list)
ungrouped: list[tuple[str, str]] = []

for path in sorted(glob.glob(os.path.join(COMMANDS_DIR, "*.yml"))):
    with open(path, encoding="utf-8", errors="ignore") as fh:
        lines = fh.readlines()

    raw_name = parse_simple_yaml_field(lines, "name")
    if not raw_name:
        raw_name = decode_filename(os.path.splitext(os.path.basename(path))[0])

    raw_desc = ""
    in_desc = False
    for line in lines:
        if line.startswith("description:"):
            rest = line[len("description:"):].strip()
            if rest and rest not in ("|-", ">-", "|", ">"):
                raw_desc = rest
                break
            in_desc = True
            continue
        if in_desc:
            stripped = line.rstrip()
            if stripped and not stripped.startswith(" ") and not stripped.startswith("\t"):
                break  # next top-level key
            raw_desc += " " + stripped.strip()

    desc = parse_first_sentence(raw_desc)
    groups = parse_groups(lines)

    if groups:
        for g in groups:
            group_to_commands[g].append((raw_name, desc))
    else:
        ungrouped.append((raw_name, desc))

# ── Events ────────────────────────────────────────────────────────────────────

print(f"Scanning {EVENTS_DIR} ...")
event_categories: dict[str, list[tuple[str, str]]] = defaultdict(list)

for category in sorted(os.listdir(EVENTS_DIR)):
    cat_path = os.path.join(EVENTS_DIR, category)
    if not os.path.isdir(cat_path):
        continue
    for path in sorted(glob.glob(os.path.join(cat_path, "*.yml"))):
        with open(path, encoding="utf-8", errors="ignore") as fh:
            lines = fh.readlines()

        raw_id = parse_simple_yaml_field(lines, "id")
        if not raw_id:
            raw_id = os.path.splitext(os.path.basename(path))[0]

        raw_desc = parse_simple_yaml_field(lines, "description") or ""
        desc = parse_first_sentence(raw_desc)

        event_categories[category].append((raw_id, desc))

# ── Write INDEX.md ────────────────────────────────────────────────────────────

version = ""
version_file = os.path.join(SCRIPT_DIR, "version.txt")
if os.path.exists(version_file):
    with open(version_file) as fh:
        version = fh.read().strip()

total_commands = sum(len(v) for v in group_to_commands.values()) + len(ungrouped)
total_events = sum(len(v) for v in event_categories.values())

lines_out: list[str] = []
lines_out.append("# Arma 3 Wiki — Command & Event Index\n")
if version:
    lines_out.append(f"_Generated from wiki data version `{version}`. Re-run `build_index.py` to refresh._\n")
lines_out.append(f"**{total_commands} commands** across {len(group_to_commands)} groups · **{total_events} event handlers** across {len(event_categories)} categories\n")
lines_out.append("\n---\n")

# Table of contents
lines_out.append("## Contents\n")
lines_out.append("- [SQF Commands by Group](#sqf-commands-by-group)\n")
for group in sorted(group_to_commands):
    anchor = group.lower().replace(" ", "-").replace("/", "").replace("(", "").replace(")", "")
    lines_out.append(f"  - [{group} ({len(group_to_commands[group])})](#sqf-{anchor})\n")
lines_out.append("- [Event Handlers by Category](#event-handlers-by-category)\n")
for cat in sorted(event_categories):
    anchor = cat.lower().replace(" ", "-")
    lines_out.append(f"  - [{cat} ({len(event_categories[cat])})](#events-{anchor})\n")
lines_out.append("\n---\n")

# Commands section
lines_out.append("## SQF Commands by Group\n\n")
lines_out.append("> **How to look up a command:** `commands/{CommandName}.yml`  \n")
lines_out.append("> Each file has: description, syntax, param types, return type, locality (`argument_loc`/`effect_loc`), version, examples.\n\n")

for group in sorted(group_to_commands):
    anchor = f"sqf-{group.lower().replace(' ', '-').replace('/', '').replace('(', '').replace(')', '')}"
    lines_out.append(f"### {group} <a id=\"{anchor}\"></a>\n\n")
    for name, desc in sorted(group_to_commands[group], key=lambda x: x[0].lower()):
        if desc:
            lines_out.append(f"- **`{name}`** — {desc}\n")
        else:
            lines_out.append(f"- **`{name}`**\n")
    lines_out.append("\n")

if ungrouped:
    lines_out.append("### (ungrouped)\n\n")
    for name, desc in sorted(ungrouped, key=lambda x: x[0].lower()):
        lines_out.append(f"- **`{name}`** — {desc}\n" if desc else f"- **`{name}`**\n")
    lines_out.append("\n")

# Events section
lines_out.append("---\n\n## Event Handlers by Category\n\n")
lines_out.append("> **How to look up an event:** `events/{category}/{EventName}.yml`  \n")
lines_out.append("> Each file has: description, params (name/type/description), locality, examples.\n\n")

for cat in sorted(event_categories):
    anchor = f"events-{cat.lower().replace(' ', '-')}"
    lines_out.append(f"### {cat} <a id=\"{anchor}\"></a>\n\n")
    for eid, desc in sorted(event_categories[cat], key=lambda x: x[0].lower()):
        if desc:
            lines_out.append(f"- **`{eid}`** — {desc}\n")
        else:
            lines_out.append(f"- **`{eid}`**\n")
    lines_out.append("\n")

with open(OUTPUT, "w", encoding="utf-8") as fh:
    fh.writelines(lines_out)

print(f"Written: {OUTPUT}")
print(f"  {total_commands} commands in {len(group_to_commands)} groups")
print(f"  {total_events} event handlers in {len(event_categories)} categories")
