"""Normalize native captures and replay output, then compare only recorded evidence."""
from __future__ import annotations

import json
from pathlib import Path

from .common import digest, read_json, write_json
from .record_views import describe, describe_work


MISSING = object()
NATIVE_RECORDS = dict(warriors='warriors_after', npcs='npcs_after', map='map_after',
                      fields='fields_after', usage='used_after', packet='packet_after',
                      allies='allies_after', buffs='buffs_after')
WORK = dict(ready='ready', selected_warrior='selected_warrior', selected_enemy='selected_enemy',
            script_enemy='script_enemy', attack_count='attack_count')


def raw(value):
    if isinstance(value, str):
        return bytes.fromhex(value)
    if isinstance(value, list):
        return bytes(value)
    raise ValueError('record must be hex or a byte array')


def words(value):
    value = raw(value)
    if len(value) % 2:
        raise ValueError('odd-length WORD array')
    return [int.from_bytes(value[i:i+2], 'big', signed=True) for i in range(0, len(value), 2)]


def native_document(path, case):
    doc = read_json(path)
    if 'cases' in doc:
        cases = doc['cases']
        selected = [c for index, c in enumerate(cases) if str(index) == case or c.get('kind', c.get('name')) == case]
        if len(selected) != 1:
            raise ValueError('select exactly one native case by index or unique kind/name')
        doc = selected[0]
    if doc.get('schema') == 'comparison-1':
        return doc
    if 'memory' in doc and 'registers' in doc:
        registers = doc.get('declarations', {}).get('compare_registers', [])
        rng = {k: doc[k] for k in ['seed', 'draws'] if k in doc}
        return dict(schema='comparison-1', records={k: v['hex'] for k, v in doc['memory'].items()},
                    work={k: doc['registers'][k] for k in registers}, rng=rng,
                    events=doc['calls'], event_projection='probe-services',
                    coverage=dict(probe_status=doc['status'], register_projection=registers,
                                  service_log_truncated=doc.get('calls_truncated', False),
                                  rng_sequence='see controlled service log'))
    records = {key: raw(doc[name]).hex() for key, name in NATIVE_RECORDS.items() if name in doc}
    if not records:
        raise ValueError('unrecognized native capture: supply comparison-1 or an *_after record capture')
    work = {target: doc[source] for source, target in WORK.items() if source in doc}
    for name in ['identity', 'mask', 'affected', 'rounds_after']:
        if name in doc:
            work[name] = words(doc[name])
    for name in ['cursor', 'counter', 'round_slot', 'round_flag', 'inactive', 'unavailable',
                 'health_effect', 'last_body', 'queue_pointer', 'queue_flag', 'total', 'defeated']:
        if name in doc:
            work[name] = doc[name]
    rng = {k: doc[k] for k in ['seed', 'draws'] if k in doc}
    if 'rng_trace' in doc:
        rng['trace'] = doc['rng_trace']
    events = [c for c in doc.get('calls', []) if c.startswith(('sound:', 'result:', 'status:', 'flash:'))]
    return dict(schema='comparison-1', records=records, work=work, rng=rng,
                events=events, event_projection='native-feedback-requests',
                coverage=dict(rng_sequence='captured' if 'trace' in rng else 'not captured',
                              event_amounts='not captured',
                              native_calls_omitted=len(doc.get('calls', [])) - len(events)))


def feedback(events):
    result = []
    for event in (entry for entry in events if isinstance(entry, dict)):
        if 'Sound' in event:
            result.append(f"sound:{event['Sound']['resource_id']}")
        f = event.get('Feedback', {})
        if 'CombatResult' in f:
            v = f['CombatResult']
            result.append(f"result:{v['body']}:{v['code']}")
        if 'OutdoorBodyStatus' in f:
            result.append('status:' + str(f['OutdoorBodyStatus']['permanent']).lower())
        if 'Flash' in f:
            v = f['Flash']
            result.append('flash:' + ':'.join(str(v[k]) for k in ['address', 'x', 'y', 'wait', 'sound']))
    return result


def source_document(path, projection):
    text = Path(path).read_text(encoding='utf-8-sig')
    try:
        document = json.loads(text)
    except json.JSONDecodeError:
        lines = [json.loads(line) for line in text.splitlines() if line.strip()]
        if not lines or not isinstance(lines[-1], dict) or 'warriors' not in lines[-1]:
            raise ValueError('replay output must end with the serialized Session') from None
        document = dict(session=lines[-1], events=[e for line in lines[:-1] if isinstance(line, list) for e in line],
                        trace=[line for line in lines[:-1] if isinstance(line, dict)])
    if document.get('schema') == 'comparison-1':
        return document
    state = document.get('session', document)
    records = {}
    if 'warriors' in state:
        records['warriors'] = bytes(v for w in state['warriors'] for v in w['bytes']).hex()
    slots = state.get('npc_backing', {}).get('slots', [])
    if slots and all(slot is not None for slot in slots):
        records['npcs'] = bytes(v for slot in slots for v in slot['record']['bytes']).hex()
    tactical = state.get('tactical_work') or {}
    for name, source in [('fields', 'spell_fields'), ('usage', 'field_used')]:
        if source in tactical:
            records[name] = raw(tactical[source]).hex()
    if tactical.get('map'):
        records['map'] = b''.join(int(v).to_bytes(2, 'big', signed=True) for v in tactical['map']['tiles']).hex()
    for name, source in [('packet', 'attack_packet'), ('allies', 'allies'), ('buffs', 'party_buffs')]:
        if state.get(source) is not None:
            records[name] = raw(state[source]).hex()
    work = {target: state[target] for target in WORK.values() if target in state}
    health, round_work = state.get('health_context') or {}, state.get('round_work') or {}
    for target, owner, key in [
        ('identity', state, 'attack_identity'), ('mask', state, 'party_mask'),
        ('affected', health, 'affected'), ('health_effect', health, 'effect'),
        ('last_body', health, 'last_body'), ('unavailable', health, 'unavailable'),
        ('cursor', state.get('initiative_work') or {}, 'cursor'),
        ('queue_pointer', state.get('retained_death_queue') or {}, 'pointer'),
        ('queue_flag', state.get('death_processing') or {}, 'queue'),
        ('total', state.get('victory_work') or {}, 'total'),
        ('defeated', state.get('victory_work') or {}, 'defeated'),
        ('rounds_after', round_work, 'slots'), ('counter', round_work, 'counter'),
        ('round_slot', round_work, 'slot'), ('round_flag', round_work, 'processing'),
        ('inactive', round_work, 'inactive_rounds')]:
        if key in owner:
            work[target] = owner[key]
    events = document.get('events', [])
    rng = dict(state.get('random', {}))
    draws = [v for v in document.get('trace', []) if v.get('kind') == 'draw']
    if draws:
        rng['trace'] = [{k: row[k] for k in ['call', 'result']} for row in draws]
    return dict(schema='comparison-1', records=records, work=work, rng=rng,
                events=feedback(events) if projection == 'native-feedback-requests' else events,
                event_projection=projection)


def compare(expected, actual, limit=500):
    rows, count, compared = [], 0, []

    def difference(section, path, left, right, **detail):
        nonlocal count
        if section == 'work':
            detail = {**describe_work(path), **detail}
        count += 1
        if len(rows) < limit:
            rows.append(dict(section=section, path=path,
                             expected='<not recorded>' if left is MISSING else left,
                             actual='<not recorded>' if right is MISSING else right, **detail))

    def walk(section, path, left, right):
        if left is MISSING or right is MISSING:
            difference(section, path, left, right)
        elif type(left) is not type(right):
            difference(section, path, left, right)
        elif isinstance(left, dict):
            for key in sorted(set(left) | set(right)):
                walk(section, f'{path}.{key}', left.get(key, MISSING), right.get(key, MISSING))
        elif isinstance(left, list):
            for index in range(max(len(left), len(right))):
                walk(section, f'{path}[{index}]', left[index] if index < len(left) else MISSING,
                     right[index] if index < len(right) else MISSING)
        elif left != right:
            difference(section, path, left, right)

    for kind, value in expected.get('records', {}).items():
        compared.append('records.' + kind)
        if kind not in actual.get('records', {}):
            difference('records', kind, 'recorded', MISSING)
            continue
        left, right = raw(value), raw(actual['records'][kind])
        for index in range(max(len(left), len(right))):
            a, b = left[index] if index < len(left) else MISSING, right[index] if index < len(right) else MISSING
            if a != b:
                difference('records', f'{kind}.byte[{index}]', a, b, **describe(kind, index, left, right))
    # Native adapters select an explicit captured projection. Extra source-only fields
    # are reported as unqualified coverage, rather than being treated as native data.
    for section in ['work', 'rng']:
        for key, value in expected.get(section, {}).items():
            compared.append(f'{section}.{key}')
            walk(section, key, value, actual.get(section, {}).get(key, MISSING))
    if 'events' in expected:
        if expected.get('event_projection') != actual.get('event_projection'):
            raise ValueError('event projections differ; use the same capture contract')
        compared.append('events')
        walk('events', 'events', expected['events'], actual.get('events', MISSING))
    if not compared:
        raise ValueError('nothing comparable was captured')
    coverage = expected.get('coverage', {})
    usable = coverage.get('probe_status', 'completed') == 'completed' and not coverage.get('service_log_truncated', False)
    return dict(schema_version=1, equal=count == 0, evidence_usable=usable, difference_count=count,
                first_difference=rows[0] if rows else None, differences=rows, truncated=count > len(rows),
                compared=compared, coverage=coverage,
                qualification='Equality covers the recorded projection only; it does not establish whole-caller or product acceptance.')


def run(native, source, case, output):
    expected = native_document(native, case)
    actual = source_document(source, expected.get('event_projection'))
    result = compare(expected, actual)
    result['inputs'] = {str(p): digest(p) for p in [native, source]}
    write_json(output.with_suffix('.json'), result)
    from .report import write_report
    write_report(output.with_suffix('.html'), 'Native / source comparison', result)
    return result
