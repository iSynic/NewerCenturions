"""Local comparison, probe, validation, navigation and product acceptance toolkit."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import sys

from reconstruction_toolkit import acceptance, batch, comparison, navigation, probes
from reconstruction_toolkit.common import ROOT, private_path, read_json, write_json
from reconstruction_toolkit.report import write_report


def parser():
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument('--root', type=Path, default=ROOT, help='this tool\'s checkout root; use each checkout\'s own copy')
    commands = result.add_subparsers(dest='tool', required=True)
    view = commands.add_parser('compare', help='compare a native case with canonical JSON or replay JSONL')
    view.add_argument('--native', type=Path, required=True)
    view.add_argument('--source', type=Path, required=True)
    view.add_argument('--case', help='unique native kind/name or zero-based index')
    view.add_argument('--out', required=True, help='new analysis report prefix; emits JSON and offline HTML')
    validate = commands.add_parser('batch', help='resumable required checks and explicit staging')
    actions = validate.add_subparsers(dest='action', required=True)
    initialize = actions.add_parser('init')
    initialize.add_argument('--allow', type=Path, required=True, help='JSON array of literal source/doc paths')
    initialize.add_argument('--protected-from', type=Path, required=True, help='preservation baseline containing fpu_frozen hashes')
    for action in [initialize, actions.add_parser('run'), actions.add_parser('verify'), actions.add_parser('stage')]:
        action.add_argument('--dir', required=True)
        if action.prog.endswith(' run'):
            action.add_argument('--checks', nargs='+', help='check names; default: all seven')
        if action.prog.endswith(' verify'):
            action.add_argument('--after-commit', action='store_true')
    probe = commands.add_parser('probe', help='execute explicit bounded original-instruction cases')
    probe.add_argument('--ghidra', type=Path, required=True)
    probe.add_argument('--project', type=Path, required=True, help='existing project prefix or .gpr path')
    probe.add_argument('--program', default='code1-mapped.bin')
    probe.add_argument('--spec', type=Path, required=True)
    probe.add_argument('--out', required=True, help='new analysis directory')
    route = commands.add_parser('navigate', help='join authored routes to source and backlog planning links')
    route.add_argument('--originals', type=Path)
    route.add_argument('--scenario', help='scenario level directory relative to originals root')
    route.add_argument('--observations', type=Path, action='append', default=[])
    route.add_argument('--out', required=True)
    kit = commands.add_parser('acceptance', help='inventory, execute and attribute build-bound acceptance evidence')
    actions = kit.add_subparsers(dest='action', required=True)
    inventory = actions.add_parser('inventory')
    inventory.add_argument('--input', type=Path, action='append', required=True)
    inventory.add_argument('--out', required=True)
    run = actions.add_parser('run')
    run.add_argument('--manifest', type=Path, required=True)
    run.add_argument('--assets', type=Path, required=True, help='prepared development root or original installation')
    run.add_argument('--build', type=Path, required=True, help='centurions-tools executable')
    run.add_argument('--out', required=True)
    observation = actions.add_parser('observe')
    observation.add_argument('--receipt', required=True, help='existing analysis acceptance receipt')
    observation.add_argument('--build', type=Path, required=True)
    observation.add_argument('--gate', choices=list(acceptance.GATES), required=True)
    observation.add_argument('--observer', required=True)
    observation.add_argument('--workflow', required=True)
    observation.add_argument('--status', choices=['pass', 'fail', 'unavailable'], required=True)
    observation.add_argument('--note', required=True)
    observation.add_argument('--evidence', type=Path, action='append', default=[])
    return result


def main():
    args = parser().parse_args()
    root = args.root.resolve()
    if root != ROOT or not (root / 'Cargo.toml').is_file() or not (root / 'tools/Run-Cargo.ps1').is_file():
        raise ValueError('--root must select this tool\'s checkout; use --assets/--originals for input installations')
    output = private_path(root, args.out) if hasattr(args, 'out') else None
    if args.tool == 'compare':
        result = comparison.run(args.native.resolve(), args.source.resolve(), args.case, output)
        print(json.dumps(dict(equal=result['equal'], evidence_usable=result['evidence_usable'], differences=result['difference_count'], report=str(output.with_suffix('.html')))))
        return 0 if result['equal'] and result['evidence_usable'] else 1
    if args.tool == 'batch':
        directory = private_path(root, args.dir)
        if args.action == 'init':
            value = batch.init(root, directory, args.allow.resolve(), args.protected_from.resolve())
        elif args.action == 'run':
            return 0 if batch.run(root, directory, args.checks) else 1
        elif args.action == 'stage':
            value = batch.stage(root, directory)
        else:
            value = batch.verify(root, read_json(directory / 'receipt.json'), args.after_commit)
            if args.after_commit and batch.git(root, 'diff', '--cached', '--name-only'):
                raise ValueError('index is not empty after commit')
        passed = len(value['checks']) == 7 and all(v.get('exit_code') == 0 for v in value['checks'].values())
        print(json.dumps(dict(source_files=len(value['source']), protected_files=len(value['protected']),
                              checks_passed=passed, staged_tree=value.get('staged_tree'))))
        return 1 if args.action == 'verify' and not passed else 0
    if args.tool == 'probe':
        project = args.project.resolve()
        if project.suffix == '.gpr':
            project = project.with_suffix('')
        result = probes.run(root, args.ghidra.resolve(), project, args.program, args.spec.resolve(), output)
        print(json.dumps(dict(passed=result['passed'], capture_complete=result['capture_complete'], receipt=str(output / 'receipt.json'))))
        return 0 if result['passed'] else 1
    if args.tool == 'navigate':
        value = navigation.run(root, (args.originals or root / 'NewCenturions').resolve(), args.scenario,
                               [p.resolve() for p in args.observations], output)
        print(json.dumps(dict(entries=len(value['rows']), potential_nodes=len(value['potential_nodes']), report=str(output.with_suffix('.html')))))
        return 0
    if args.action == 'inventory':
        value = acceptance.inventory([p.resolve() for p in args.input])
        write_json(output.with_suffix('.json'), value)
        write_report(output.with_suffix('.html'), 'Available original data', value)
        print(json.dumps(dict(candidates=len(value['rows']), report=str(output.with_suffix('.html')))))
        return 0
    if args.action == 'run':
        value = acceptance.run(root, args.manifest.resolve(), args.build.resolve(), args.assets.resolve(), output)
        print(json.dumps(dict(cases=[dict(id=r['id'], status=r['status']) for r in value['rows']], receipt=str(output / 'receipt.json'))))
        return 0 if all(r['status'] == 'headless checks passed' for r in value['rows']) else 1
    value = acceptance.observe(private_path(root, args.receipt), args.build.resolve(), args.gate,
                               args.observer, args.workflow, args.status, args.note,
                               [p.resolve() for p in args.evidence])
    print(json.dumps(dict(observations=len(value['observations']), gate_status=value['gates'][args.gate]['status'])))
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (ValueError, KeyError, OSError, subprocess.SubprocessError) as error:
        print(f'toolkit: {error}', file=sys.stderr)
        raise SystemExit(2)
