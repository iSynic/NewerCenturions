"""Build-bound inventories, reproducible headless cases and attributed observations."""
from __future__ import annotations

import json
from pathlib import Path
import subprocess

from .common import digest, host, identities, now, read_json, verify_identities, write_json
from .comparison import compare
from .report import write_report


GATES = {
    'P01': 'Populated original-save import and continuation',
    'P02': 'Windows desktop workflows and original presentation',
    'P03': 'Other genuine scenarios', 'P04': 'Linux/macOS runtime acceptance',
    'P05': 'Graphics forms required by genuine data', 'P06': 'Sound forms required by genuine data',
}


def input_files(assets):
    originals = assets / 'NewCenturions' if (assets / 'NewCenturions').is_dir() else assets
    if originals == assets and (assets / 'Cargo.toml').is_file():
        raise ValueError('original installation unavailable')
    paths = [p for p in originals.rglob('*') if p.is_file()]
    extracted = assets / 'analysis/extracted'
    if extracted.is_dir():
        paths += [p for p in extracted.rglob('*') if p.is_file()]
    return paths


def inventory(roots):
    rows, paths = [], []
    for root in roots:
        if not root.is_dir():
            rows.append(dict(kind='unavailable', path=str(root)))
            continue
        for path in root.rglob('*'):
            if not path.is_file():
                continue
            if path.name == 'Data I1':
                kind = 'original save candidate'
            elif path.name == 'Data Mission':
                kind = 'scenario candidate'
            elif path.suffix.lower() in {'.rsrc', '.rsf'} or path.name.startswith('._'):
                kind = 'resource fork candidate'
            else:
                continue
            paths.append(path)
            rows.append(dict(kind=kind, path=str(path), size=path.stat().st_size, sha256=digest(path),
                             qualification='Candidate supplied by the selected input root; provenance requires review.'))
    return dict(schema_version=1, rows=rows, inputs=identities(paths),
                qualification='Inventory establishes available candidates, not successful import, genuine provenance or product acceptance.')


def run(root, manifest_path, build, assets, directory):
    manifest = read_json(manifest_path)
    if manifest.get('schema_version') != 1 or directory.exists():
        raise ValueError('acceptance manifest schema must be1 and output directory must be new')
    cases = manifest['cases']
    names = [c['id'] for c in cases]
    if not cases or len(names) != len(set(names)):
        raise ValueError('acceptance case IDs must be nonempty and unique')
    inputs = [manifest_path, build] + input_files(assets)
    for case in cases:
        if not case['id'].replace('-', '').replace('_', '').isalnum():
            raise ValueError('case IDs may contain letters, digits, hyphen and underscore')
        if case['kind'] not in {'resources', 'import', 'replay'}:
            raise ValueError('case kind must be resources, import or replay')
        if not set(case.get('gates', [])) <= set(GATES):
            raise ValueError('case refers to an unknown product gate')
        if not isinstance(case.get('timeout_seconds', 120), int) or not 1 <= case.get('timeout_seconds', 120) <= 3600:
            raise ValueError('case timeout must be an integer from1 through3600 seconds')
        for name in ['input', 'commands', 'expected']:
            if case.get(name):
                path = Path(case[name])
                if not path.is_absolute():
                    path = manifest_path.parent / path
                case[name] = str(path.resolve())
                if path.is_dir():
                    inputs += [p for p in path.rglob('*') if p.is_file()]
                else:
                    inputs.append(path)
        if case['kind'] == 'import':
            save = Path(case['input'])
            inputs += [p for p in (save if save.is_dir() else save.parent).iterdir()
                       if p.is_file() and p.name.startswith('Data ')]
    fingerprints = identities(inputs)
    receipt = dict(schema_version=1, started=now(), build=str(build), build_sha256=digest(build),
                   host=host(), assets=str(assets), inputs=fingerprints, rows=[], observations=[])
    directory.mkdir(parents=True)
    write_json(directory / 'receipt.json', receipt)
    for case in cases:
        verify_identities(fingerprints)
        work = directory / case['id']
        work.mkdir()
        if case['kind'] == 'resources':
            argv = [build, 'audit-resources', assets]
        elif case['kind'] == 'import':
            argv = [build, 'audit-import', assets, case['input'], case['commands'], work]
        else:
            argv = [build, 'replay', assets, case['commands'], '--from', case['input'], '--trace']
        try:
            process = subprocess.run(list(map(str, argv)), cwd=root, capture_output=True, timeout=case.get('timeout_seconds', 120))
            (work / 'stdout.jsonl').write_bytes(process.stdout)
            (work / 'stderr.txt').write_bytes(process.stderr)
            row = dict(id=case['id'], kind=case['kind'], exit_code=process.returncode,
                       argv=list(map(str, argv)), gates=case.get('gates', []), status='failed',
                       declared_provenance=case.get('provenance', 'unreviewed input'))
            if process.returncode == 0:
                values = [json.loads(line) for line in process.stdout.decode('utf-8-sig').splitlines() if line.strip()]
                value = values[-1]
                if case['kind'] == 'resources':
                    row['decoded'] = sum(v['status'] == 'decoded' for v in value['resources'])
                    row['rejected'] = sum(v['status'] == 'rejected' for v in value['resources'])
                    row['unadapted'] = sum(v['status'] == 'no decoder adapter' for v in value['resources'])
                    row['status'] = 'needs review' if row['rejected'] or row['unadapted'] else 'headless checks passed'
                else:
                    row['status'] = 'headless checks passed' if case['kind'] == 'replay' or value.get('accepted') else 'failed'
                if case['kind'] in {'replay', 'import'}:
                    events = [e for line in values[:-1] if isinstance(line, list) for e in line]
                    trace = [line for line in values[:-1] if isinstance(line, dict) and 'kind' in line]
                    for step in value.get('steps', []):
                        events.extend(step.get('events', []))
                        trace.extend(step.get('trace', []))
                    observed = dict(schema_version=1, authority='source command execution',
                                    build_sha256=receipt['build_sha256'], events=events, trace=trace,
                                    content_identity=value.get('session', value).get('content_identity'),
                                    source_stdout=str(work / 'stdout.jsonl'),
                                    source_stdout_sha256=digest(work / 'stdout.jsonl'))
                    write_json(work / 'observations.json', observed)
                    row['observations_sha256'] = digest(work / 'observations.json')
                if case.get('expected'):
                    comparison = compare(dict(work=read_json(case['expected'])), dict(work=value))
                    write_json(work / 'comparison.json', comparison)
                    if not comparison['equal']:
                        row['status'] = 'expected output differs'
            row['stdout_sha256'] = digest(work / 'stdout.jsonl')
            row['stderr_sha256'] = digest(work / 'stderr.txt')
        except subprocess.TimeoutExpired:
            row = dict(id=case['id'], kind=case['kind'], status='timeout', gates=case.get('gates', []))
        verify_identities(fingerprints)
        receipt['rows'].append(row)
        receipt['finished'] = now()
        receipt['qualification'] = 'Cases bind these inputs and executable. Headless checks do not grant visual, playback, platform or campaign acceptance.'
        write_json(directory / 'receipt.json', receipt, replace=True)
    matrix(receipt)
    write_json(directory / 'receipt.json', receipt, replace=True)
    write_report(directory / 'report.html', 'Product acceptance evidence', receipt)
    return receipt


def matrix(receipt):
    receipt['gates'] = {
        key: dict(title=title, status='review required',
                  automated_cases=[r['id'] for r in receipt['rows'] if key in r.get('gates', [])],
                  observations=[i for i, r in enumerate(receipt['observations']) if r['gate'] == key])
        for key, title in GATES.items()}


def observe(receipt_path, build, gate, observer, workflow, status, note, evidence):
    receipt = read_json(receipt_path)
    verify_identities(receipt['inputs'])
    if digest(build) != receipt['build_sha256']:
        raise ValueError('observation build differs from the acceptance receipt')
    if gate not in GATES or not observer.strip() or not workflow.strip() or not note.strip():
        raise ValueError('observation requires a gate, observer, workflow and note')
    receipt['observations'].append(dict(gate=gate, observer=observer, workflow=workflow, status=status,
                                        note=note, evidence=identities(evidence), build_sha256=digest(build),
                                        host=host(), recorded=now(), level='attributed observation'))
    matrix(receipt)
    write_json(receipt_path, receipt, replace=True)
    # Keep every rendered observation revision; do not overwrite an earlier review.
    write_report(receipt_path.parent / f"observations-{len(receipt['observations']):03}.html",
                 'Product acceptance observations', receipt)
    return receipt
