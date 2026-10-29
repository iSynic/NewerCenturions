"""Launch bounded declarative probes and retain failures as inspectable receipts."""
from pathlib import Path
import os
import subprocess

from .common import digest, identities, read_json, write_json


def run(root, ghidra, project, program, spec_path, directory):
    if directory.exists():
        raise ValueError('probe output directory must be new')
    spec = read_json(spec_path)
    cases = spec.get('cases', [])
    names = [c['name'] for c in cases]
    if spec.get('schema_version') != 1 or not names or not all(isinstance(n, str) and n.strip() for n in names) or len(names) != len(set(names)):
        raise ValueError('probe schema must be1 with unique, nonempty case names')
    executable = ghidra / 'support' / ('analyzeHeadless.bat' if os.name == 'nt' else 'analyzeHeadless')
    if not executable.is_file() or not project.with_suffix('.gpr').is_file():
        raise ValueError('Ghidra installation/project unavailable')
    definitions = [root / 'tools/ghidra' / name for name in
                   ['ActionOracleHarness.java', 'SaneBoundary.java', 'ToolkitProbeHarness.java', 'VerifyToolkitProbe.java']]
    inputs = [spec_path, executable, ghidra / 'Ghidra/application.properties'] + definitions
    for case in cases:
        for memory in case.get('memory', []):
            if 'file' in memory:
                path = (root / memory['file']).resolve()
                if not path.is_relative_to(root):
                    raise ValueError('probe memory input escapes checkout')
                inputs.append(path)
    hashes = identities(inputs)
    directory.mkdir(parents=True)
    argv = [str(executable), str(project.parent), project.name, '-process', program,
            '-noanalysis', '-readOnly', '-scriptPath', str(root / 'tools/ghidra'),
            '-postScript', 'VerifyToolkitProbe.java', str(root), str(spec_path), str(directory / 'capture.json')]
    with (directory / 'ghidra.log').open('xb') as stream:
        process = subprocess.run(argv, cwd=root, stdout=stream, stderr=subprocess.STDOUT,
                                 creationflags=0x08000000 if os.name == 'nt' else 0)
    capture_path = directory / 'capture.json'
    capture = read_json(capture_path) if capture_path.is_file() else None
    log = (directory / 'ghidra.log').read_text(encoding='utf-8', errors='replace')
    warnings = [line for line in log.splitlines() if 'uninitialized' in line.lower()]
    complete = process.returncode == 0 and capture is not None and len(capture['cases']) == len(cases)
    passed = complete and not warnings and all(c['status'] == 'completed' for c in capture['cases'])
    result = dict(schema_version=1, passed=passed, capture_complete=complete, exit_code=process.returncode,
                  argv=argv, inputs=hashes, log_sha256=digest(directory / 'ghidra.log'),
                  uninitialized_memory_diagnostics=warnings,
                  capture_sha256=digest(capture_path) if capture else None)
    from .report import write_report
    result['rows'] = [dict(id=c['name'], status=c['status'], pc=c['pc'], error=c.get('error'),
                          registers=c['registers'], recent_instructions=c['recent_instructions'],
                          recent_native_calls=c.get('recent_native_calls', []), calls=c['calls'],
                          stack_hex=c.get('stack_hex'), memory=c['memory']) for c in capture['cases']] if capture else []
    result['qualification'] = 'Completed and failed cases retain separate status. Controlled services and numeric models do not qualify original ROM/FPU or whole callers.'
    from .common import verify_identities
    verify_identities(hashes)
    write_json(directory / 'receipt.json', result)
    write_report(directory / 'report.html', 'Bounded native probe cases', result)
    return result
