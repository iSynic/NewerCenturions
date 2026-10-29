"""Validate the unsupported-behavior ledger and render its current work queue."""
import argparse
from collections import Counter
import json
from pathlib import Path

import engine_inventory

ROOT = Path(__file__).resolve().parent.parent
LEDGER = ROOT / 'evidence/unsupported-ledger.json'
REPORT = ROOT / 'docs/unsupported-scope.md'
STATUSES = {'open', 'in_progress', 'partial', 'closed', 'blocked', 'frozen'}


def reconcile(ledger, rows):
    active = {r['id']:r for r in rows if r['kind']=='diagnostic' and r['layer']=='maintained'}
    diagnostics = ledger['diagnostics']
    if missing := sorted(set(active)-set(diagnostics)):
        raise ValueError('Assign new diagnostic identities before refreshing: ' + ', '.join(missing))
    for ident, diagnostic in diagnostics.items():
        # Frozen sites belong to their checkpoint; the inventory supplies current locations.
        assert diagnostic['category']!='frozen' or ident in active, 'Frozen diagnostic disappeared: ' + ident
        if diagnostic['category']!='frozen':
            diagnostic.update(current=ident in active, sites=active[ident]['sites'] if ident in active else [])
    groups = {g['id']:g for g in ledger['groups']}
    assert set(ledger['baseline']['group_ids']) <= set(groups)
    assert groups['F01']['status']=='frozen' or ledger['policy']['fpu']!='frozen', 'F01 must remain frozen'
    for group in groups.values():
        assert group['status'] in STATUSES, group['id']
        assert set(group['dependencies']) <= set(groups), group['id']
        if group['status']=='closed':
            assert group['resolution'] and group['validation'], group['id']
        group['active_diagnostics'] = sorted(i for i,d in diagnostics.items()
                                             if d['current'] and d['group']==group['id'])
    for ident, diagnostic in diagnostics.items():
        assert diagnostic['category'] in ledger['baseline']['diagnostic_categories'], ident
        assert diagnostic['group'] is None or diagnostic['group'] in groups, ident
    return dict(total=len(active), categories=dict(Counter(diagnostics[i]['category'] for i in active)),
                groups=dict(Counter(g['status'] for g in groups.values())),
                non_fpu=dict(Counter(g['status'] for g in groups.values() if g['id']!='F01')),
                product=dict(Counter(g['status'] for g in ledger['product_entries'])))


def render(ledger, totals):
    lines = ['# Unsupported behavior backlog', '',
             'The Centurions 1.3.2 binary is the behavior authority. This ledger tracks implementation, remaining uncertainty and native fidelity separately.', '',
             '## Current status', '',
             '| Work | Status counts |', '|---|---|',
             '| Non-FPU rule groups | ' + ', '.join(f'{n} {k.replace("_"," ")}' for k,n in sorted(totals['non_fpu'].items())) + ' |',
             '| F01 | ' + next(g for g in ledger['groups'] if g['id']=='F01')['status'].capitalize() + ' |',
             '| Subsequent product acceptance | ' + ', '.join(f'{n} {k.replace("_"," ")}' for k,n in sorted(totals['product'].items())) + ' |', '',
             f"Current maintained diagnostic identities: **{totals['total']}**. " + ', '.join(f'{k}: {v}' for k,v in sorted(totals['categories'].items())) + '.', '',
             'Baseline: 24 implementation groups, nine investigations, two context groups and one frozen arithmetic group. The corrected initial census contained 236 maintained diagnostic identities. Groups are unequal planning units; diagnostic counts do not measure completion or fidelity.', '',
             ledger['policy'].get('fpu_summary', 'FPU implementation, fixtures and qualification receipts remain frozen. Existing regression checks may read their fixtures. Rules with an unresolved FPU dependency remain explicitly blocked.'), '',
             '## Latest batches', '']
    for batch in ledger['batches'][-6:][::-1]:
        lines += [f"### {batch['id']} — {batch['date']}", '', batch['summary'], '',
                  '**Closed:** ' + (', '.join(batch['closed']) or 'None') + '. ' +
                  '**Partial:** ' + (', '.join(batch['partial']) or 'None') + '. ' +
                  '**Blocked:** ' + (', '.join(batch['blocked']) or 'None') + '.', '',
                  '**Still open:** ' + (', '.join(batch.get('still_open', [])) or 'See current priority queue') + '.', '',
                  '**Discoveries:** ' + (' '.join(batch.get('discoveries', [])) or 'None recorded') + '.', '',
                  '**Reclassifications:** ' + (' '.join(batch.get('reclassifications', [])) or 'None recorded') + '.', '',
                  '**Verification:** ' + batch['validation'] + '.', '',
                  '**Next:** ' + batch['next'] + '.', '']
    lines += ['## Priority queue', '',
              'A reproduced normal-play failure comes first, followed by an authored valid caller, then source-supported variants and custom data. Dependencies move before their consumers. Absence from Thunder lowers priority without closing an entry.', '',
              '| Rank | ID | Work | Wave | Status | Active diagnostics | Evidence |',
              '|---:|---|---|---:|---|---:|---|']
    for g in sorted(ledger['groups'],key=lambda g:g['priority']):
        lines.append(f"| {g['priority']} | [{g['id']}](#{g['id'].lower()}) | {g['title']} | {g['wave']} | {g['status'].replace('_',' ')} | {len(g['active_diagnostics'])} | {g['evidence_level']} |")
    lines += ['', '## Entry details', '']
    for g in sorted(ledger['groups'],key=lambda g:g['priority']):
        lines += [f"<a id=\"{g['id'].lower()}\"></a>", f"### {g['id']} — {g['title']}", '',
                  g['behavior'], '', '**Complete when:** ' + g['completion_condition'], '',
                  '**Dependencies:** ' + (', '.join(g['dependencies']) or 'None') + '.', '',
                  '**Current disposition:** ' + (g['resolution'] or 'Open; no completion claim.') + '', '',
                  '**Remaining:** ' + ' '.join(g['remaining_limitations']), '',
                  '**Supplied callers:** ' + ' '.join(g['supplied_callers']), '',
                  '**Observed failures:** ' + (' '.join(g['observed_failures']) or 'No new failure captured for this entry.'), '']
        sources = sorted({(s['file'],s['line']) for i in g['diagnostics'] for s in ledger['diagnostics'][i]['sites']})
        if sources:
            lines += ['**Current source:** ' + '; '.join(f'[{p}:{n}](../{p})' for p,n in sources) + '.', '']
        if g['evidence']:
            lines += ['**Evidence:** ' + ' '.join(e['summary'] for e in g['evidence']), '']
        lines += ['**Validation:** ' + ('; '.join(g['validation']) or 'Pending'), '']
    lines += ['## Subsequent product acceptance', '', '| ID | Work | Status | Remaining |', '|---|---|---|---|']
    for g in ledger['product_entries']:
        lines.append(f"| {g['id']} | {g['title']} | {g['status']} | {g['remaining']} |")
    lines += ['', '## Census interpretation', '',
              'The initial 293 count combined 228 maintained and 65 private-reference identities. Corrected import ownership and parameterized helpers produced 236 maintained identities. The detailed identity migration is retained in the ledger baseline; corrections are not implementation progress.', '',
              'Guard and narrow-helper identities remain classified without being counted as missing features. Source locations refresh automatically. Removed diagnostics remain in entry history, and new identities require explicit classification. Closed investigation entries record whether their outcome was an implemented route, original restriction, helper contract or undefined boundary.', '',
              'All 147 supplied player-spell IDs have admission routes, and all 112 original newland arms have names/routes. Neither fact establishes every effect/caller combination or original-game fidelity. Silent errors and omitted original behavior can escape an explicit-error census.', '',
              'The machine-readable source of truth is [unsupported-ledger.json](../evidence/unsupported-ledger.json). Run `python tools/unsupported_ledger.py --refresh` after edits and `python tools/unsupported_ledger.py --check` to validate source coverage, dispositions and report consistency. Initial source-review artifacts remain under ignored `analysis/unsupported-scope-01/`.', '']
    return '\n'.join(lines)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--refresh', action='store_true')
    parser.add_argument('--check', action='store_true')
    args=parser.parse_args()
    ledger=json.loads(LEDGER.read_text(encoding='utf-8'))
    totals=reconcile(ledger,engine_inventory.diagnostics())
    ledger['current_counts']=totals
    report=render(ledger,totals)
    if args.refresh:
        LEDGER.write_text(json.dumps(ledger,indent=2)+'\n',encoding='utf-8')
        REPORT.write_text(report,encoding='utf-8')
    elif args.check:
        assert LEDGER.read_text(encoding='utf-8')==json.dumps(ledger,indent=2)+'\n', 'Refresh ledger source locations/counts'
        assert REPORT.read_text(encoding='utf-8')==report, 'Refresh readable ledger'
    print(json.dumps(totals,indent=2))


if __name__=='__main__':
    main()
