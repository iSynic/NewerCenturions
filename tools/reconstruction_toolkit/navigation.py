"""Join authored dependencies to current source owners and maintained backlog entries."""
from __future__ import annotations

from collections import Counter
import json
from pathlib import Path
import re

from .common import digest, identities, read_json, write_json
from .report import write_report


def observations(paths):
    opcodes, spells, items, evidence = Counter(), Counter(), Counter(), []
    for path in paths:
        text = path.read_text(encoding='utf-8-sig')
        try:
            value = json.loads(text)
            values = [value] if isinstance(value, dict) else value
        except json.JSONDecodeError:
            values = [json.loads(line) for line in text.splitlines() if line.strip()]
        if not isinstance(values, list):
            raise ValueError('observations must contain replay JSON/JSONL')
        events, trace, binding = [], [], None
        for value in values:
            if isinstance(value, list):
                events.extend(value)
            elif isinstance(value, dict):
                if value.get('source_stdout'):
                    if digest(Path(value['source_stdout'])) != value['source_stdout_sha256']:
                        raise ValueError('observation source output changed')
                if value.get('kind') == 'story':
                    trace.append(value)
                events.extend(value.get('events', []))
                trace.extend(value.get('trace', []))
                binding = value.get('build_sha256', binding)
        opcodes.update(row['opcode'] for row in trace if row.get('kind') == 'story')
        for event in events:
            if 'SpellResolved' in event:
                spells.update([event['SpellResolved']['spell']])
            # Item uses are only observed when an event explicitly supplies an item ID.
            for name in ['ItemActivated', 'ItemUsed']:
                if name in event and 'item' in event[name]:
                    items.update([event[name]['item']])
        evidence.append(dict(path=str(path), sha256=digest(path), build_sha256=binding,
                             level='source observation; build identity supplied' if binding else 'unbound source observation'))
    return opcodes, spells, items, evidence


def build(root, originals, scenario, observed=()):
    import engine_inventory
    import scenario_route
    original_scenario = Path(scenario) if scenario else scenario_route.SCENARIO
    base = (originals / original_scenario).resolve()
    if not base.is_relative_to(originals.resolve()):
        raise ValueError('scenario must be inside the supplied originals root')
    catalogue = scenario_route.catalogue(originals, original_scenario)
    graph = scenario_route.completion_inventory(originals, catalogue, original_scenario)
    ledger = read_json(root / 'evidence/unsupported-ledger.json')
    groups = {g['id']: dict(id=g['id'], title=g['title'], status=g['status']) for g in ledger['groups']}
    diagnostics = engine_inventory.diagnostics()
    by_file = {}
    for row in diagnostics:
        if row['layer'] != 'maintained':
            continue
        disposition = ledger['diagnostics'].get(row['id'])
        for name in row['files']:
            by_file.setdefault(name, []).append(dict(id=row['id'], group=disposition['group'] if disposition else None))
    known = engine_inventory.story_operations()
    typed = {}
    for path in (root / 'crates/core/src').rglob('*.rs'):
        for number, line in enumerate(path.read_text(encoding='utf-8').splitlines(), 1):
            for name in re.findall(r'Operation::([A-Za-z0-9_]+)', line):
                typed.setdefault(name, []).append(dict(path=path.relative_to(root).as_posix(), line=number))
    ops, spells, items, evidence = observations(observed)
    rows = []
    for opcode, entry in graph['opcodes'].items():
        typed_references = typed.get(known.get(int(opcode)), [])
        owners = sorted({a['path'] for a in entry['source_arms'] + typed_references})
        related = {d['id']: d for owner in owners for d in by_file.get(owner, [])}
        rows.append(dict(kind='operation', id=f'opcode:{opcode}', opcode=int(opcode),
                         operation=known.get(int(opcode)), source_arm_present=entry['source_arm_present'],
                         typed_source_references=typed_references,
                         source_arms=entry['source_arms'], authored_references=entry['references'],
                         source_observations=ops[int(opcode)], related_diagnostics=list(related.values()),
                         group_links=sorted({d['group'] for d in related.values() if d['group']}),
                         link_scope='Diagnostics share a source file with the arm; branch applicability is unproven.'))
    for item in graph['items']:
        rows.append(dict(kind='item', **item, source_observations=items[item['id']],
                         group_links=['I02', 'I08', 'I14', 'L01'],
                         link_scope='Equipment/item workflow planning links; eligibility and a missing branch are not inferred.'))
    for spell in graph['spells']:
        if spell['authored']:
            rows.append(dict(kind='spell', **spell, source_observations=spells[spell['id']],
                             group_links=['I16', 'I20', 'B04', 'B05'] + (['I22'] if abs(spell['effect']) == 58 else []),
                             link_scope='Packet/target family planning links; field values and source admission do not prove runtime coverage.'))
    data_inputs = [p for p in base.iterdir() if p.is_file() and p.name.startswith('Data ')]
    data_inputs += [originals / 'TheNewCenturions/Data Files' / n for n in ['Data S', 'Data ID']]
    source_inputs = [root / 'tools/scenario_route.py', root / 'evidence/unsupported-ledger.json',
                     root / 'crates/core/src/exploration/script_execution/operation.rs']
    source_inputs += [root / name for name in by_file]
    return dict(schema_version=1, scenario=str(original_scenario), rows=rows, groups=groups,
                potential_nodes=graph['potential_nodes'], unqualified_nodes=graph['unqualified_nodes'],
                nodes=graph['nodes'], edges=graph['edges'], uncertainties=graph['uncertainties'],
                conditional_sources=graph['conditional_sources'], observations=evidence,
                inputs=identities(data_inputs + source_inputs),
                qualification='Authored routes are conservative potential uses. Source observations are separate, do not establish original-game reachability, and never close backlog groups.')


def run(root, originals, scenario, observed, output):
    result = build(root, originals, scenario, observed)
    write_json(output.with_suffix('.json'), result)
    write_report(output.with_suffix('.html'), 'Scenario / backlog navigator', result)
    return result
