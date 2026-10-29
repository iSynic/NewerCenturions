"""Names and big-endian widths from content::records and retained work views.

Unlisted bytes remain compared; an unknown field is never inferred from its value.
"""
import re


WORK_VIEWS = {
    'ready': (0x438760, 2, True), 'selected_warrior': (0x4435d2, 2, True),
    'selected_enemy': (0x447196, 2, True), 'script_enemy': (0x43800e, 2, True),
    'attack_count': (0x4435cc, 2, True), 'identity': (0x4435c6, 2, True),
    'mask': (0x4434e0, 2, True), 'affected': (0x442798, 2, True),
    'cursor': (0x447194, 2, True), 'counter': (0x44255c, 1, True),
    'round_slot': (0x408576, 2, True), 'round_flag': (0x438583, 1, True),
    'rounds_after': (0x439cec, 2, True), 'inactive': (0x4427a6, 2, True),
    'unavailable': (0x442739, 1, True), 'health_effect': (0x443760, 1, True),
    'last_body': (0x408574, 2, True), 'queue_pointer': (0x437a90, 4, False),
    'queue_flag': (0x438584, 1, True), 'total': (0x4435c4, 2, True),
    'defeated': (0x44374c, 2, True),
}

VIEWS = {
    'warriors': (3138, 0x448aa0, [
        (0x10, 2, True, 'faction'), (0x16, 2, True, 'guarding'),
        (0x50, 2, True, 'stamina'), (0x52, 2, True, 'maximum stamina'),
        (0x60, 2, True, 'turn skip duration'), (0x98, 2, True, 'confusion'),
        (0x9df, 1, False, 'active flag'),
    ]),
    'npcs': (288, 0x439d3c, [
        (0xe, 2, True, 'faction'), (0x10, 2, True, 'footprint'),
        (0x110, 2, True, 'stamina'), (0x112, 2, True, 'maximum stamina'),
        (0x11e, 2, True, 'under tile'),
    ]),
    'fields': (20, 0x442bd6, [
        (0, 2, True, 'top'), (2, 2, True, 'left'), (4, 2, True, 'bottom'),
        (6, 2, True, 'right'), (8, 2, True, 'spell'), (15, 1, False, 'duration'),
        (16, 1, True, 'power'), (17, 1, True, 'rotation'), (19, 1, True, 'level'),
    ]),
    'map': (2, 0x4085b8, [(0, 2, True, 'tile')]),
    'usage': (1, 0x435884, [(0, 1, False, 'usage byte')]),
    'packet': (58, 0x44374e, [
        (1, 1, False, 'targeting mode'), (18, 1, True, 'health effect'),
        (19, 1, False, 'defense category'), (38, 2, True, 'impact sound'),
        (40, 2, True, 'effect'),
    ]),
    'allies': (288, 0x440edc, []), 'buffs': (2, 0x442562, [(0, 2, True, 'duration')]),
}


def describe(kind, index, expected, actual):
    stride, base, fields = VIEWS.get(kind, (1, None, []))
    slot, offset = divmod(index, stride)
    field = next((f for f in fields if f[0] <= offset < f[0] + f[1]), None)
    result = dict(record=slot, offset=offset, address=hex(base + index) if base else None,
                  field='unmapped byte', width=1, signed=False)
    if field:
        start, width, signed, label = field
        result.update(field=label, offset=start, width=width, signed=signed,
                      address=hex(base + slot * stride + start) if base else None)
        for name, raw in [('expected_value', expected), ('actual_value', actual)]:
            value = raw[slot * stride + start:slot * stride + start + width]
            result[name] = int.from_bytes(value, 'big', signed=signed) if len(value) == width else None
    if kind == 'map':
        result['coordinates'] = [slot // 90, slot % 90]
    return result


def describe_work(path):
    match = re.fullmatch(r'(\w+)(?:\[(\d+)\])?', path)
    if match and match[1] in WORK_VIEWS:
        address, width, signed = WORK_VIEWS[match[1]]
        return dict(field=match[1].replace('_', ' '), address=hex(address + int(match[2] or 0) * width),
                    width=width, signed=signed)
    return {}
