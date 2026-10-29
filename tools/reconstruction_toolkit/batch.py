"""Resumable fixed validation plan with source, toolchain and preservation identities."""
from __future__ import annotations

import json
import os
from pathlib import Path, PurePosixPath
import shutil
import subprocess
import sys
import time

from .common import (checkout_files, digest, git, host, identities, now, process_alive,
                     read_json, verify_identities, write_json)


def source_paths(root, allow=()):
    paths = [root / name for name in checkout_files(root)
            if (name.startswith(('crates/', 'tools/', 'tests/', 'archive/crates/', 'vendor/'))
                and Path(name).suffix in {'.rs', '.py', '.ps1', '.toml', '.java'})
            or name in {'Cargo.toml', 'Cargo.lock', 'archive/Cargo.toml', 'archive/Cargo.lock',
                        'AGENTS.md', 'docs/maintainability.md', 'tools/toolkit.py'}]
    return sorted(set(paths) | {root / name for name in allow if (root / name).is_file()})


def allowed_paths(root, values):
    if not isinstance(values, list) or not values or not all(isinstance(n, str) for n in values) or len(values) != len(set(values)):
        raise ValueError('allowlist must be a nonempty array of unique literal file paths')
    for name in values:
        path = PurePosixPath(name)
        if path.is_absolute() or '..' in path.parts or ':' in name or '\\' in name:
            raise ValueError(f'not a repository-relative literal path: {name}')
        source = name.startswith(('crates/', 'tools/', 'tests/', 'archive/crates/')) and path.suffix in {'.rs', '.py', '.ps1', '.java', '.toml', '.slaspec', '.ldefs'}
        documentation = name.startswith('docs/') and path.suffix == '.md'
        metadata = name in {'evidence/unsupported-ledger.json', 'evidence/engine-inventory-notes.json',
                            'evidence/capability-map.json', 'evidence/capability-map-notes.json',
                            'Cargo.toml', 'Cargo.lock', 'archive/Cargo.toml', 'archive/Cargo.lock',
                            'AGENTS.md', 'README.md', 'LICENSE', 'THIRD_PARTY.md', '.gitignore', '.gitattributes'}
        if not (source or documentation or metadata):
            raise ValueError(f'not maintained source/documentation: {name}')
        if not (root / name).resolve().is_relative_to(root.resolve()):
            raise ValueError(f'allowlist path escapes checkout: {name}')
        if (root / name).exists() and not (root / name).is_file():
            raise ValueError(f'allowlist entries must be files: {name}')
    return sorted(values)


def commands(root):
    base = ['pwsh', '-NoLogo', '-NoProfile', '-NonInteractive', '-File',
            str(root / 'tools/Run-Cargo.ps1')]
    options = ['-BuildJobs', '1', '-AllowCompilerParallelism:$false', '-MaximumTestSeconds', '600']
    return {
        'python': [sys.executable, '-m', 'unittest', 'discover', '-s', 'tests/python'],
        'format': ['cargo', 'fmt', '--check'],
        'workspace-tests': base + ['-Action', 'Test', '-OriginalFixtures'] + options,
        'workspace-clippy': base + ['-Action', 'Clippy', '-AllTargets', '-OriginalFixtures'] + options,
        'release-build': base + ['-Action', 'Build', '-Release'] + options,
        'archive-tests': base + ['-Action', 'Test', '-ManifestPath', 'archive/Cargo.toml', '-OriginalFixtures'] + options,
        'archive-clippy': base + ['-Action', 'Clippy', '-ManifestPath', 'archive/Cargo.toml', '-AllTargets', '-OriginalFixtures'] + options,
    }


def toolchain():
    result = {}
    for name, flag in [(sys.executable, '--version'), ('cargo', '--version'),
                       ('rustc', '--version'), ('pwsh', '--version')]:
        executable = shutil.which(name)
        if executable is None:
            raise ValueError(f'required tool unavailable: {name}')
        version = subprocess.check_output([executable, flag], timeout=30).decode().strip()
        result[name] = dict(executable=str(Path(executable).resolve()), sha256=digest(executable), version=version)
    return result


def init(root, directory, allow_file, protected_file):
    if directory.exists():
        raise ValueError('batch directory must be new')
    if git(root, 'diff', '--cached', '--name-only'):
        raise ValueError('batch initialization requires an empty index')
    allow = allowed_paths(root, read_json(allow_file))
    frozen = read_json(protected_file)['fpu_frozen']
    verify_identities(frozen)
    if any(str((root / name).resolve()) in frozen for name in allow):
        raise ValueError('allowlist intersects frozen work')
    inputs = source_paths(root, allow)
    protected = [root / name for name in checkout_files(root) if name not in allow]
    protected += [p for p in (root / 'NewCenturions').rglob('*') if p.is_file()]
    protected += [Path(p) for p in frozen]
    receipt = dict(schema_version=1, created=now(), root=str(root), host=host(),
                   head=git(root, 'rev-parse', 'HEAD').decode().strip(),
                   branch=git(root, 'branch', '--show-current').decode().strip(),
                   allow=allow, source=identities(inputs), protected=identities(protected),
                   frozen=frozen, commands=commands(root), toolchain=toolchain(), checks={},
                   initial_status=git(root, 'status', '--short', '--untracked-files=all', '-z').decode())
    write_json(directory / 'receipt.json', receipt)
    return receipt


def verify(root, receipt, allow_commit=False, check_protected=True):
    if receipt['schema_version'] != 1 or Path(receipt['root']).resolve() != root.resolve():
        raise ValueError('receipt belongs to another checkout or schema')
    head = git(root, 'rev-parse', 'HEAD').decode().strip()
    if head != receipt['head']:
        if not allow_commit or not receipt.get('staged_tree'):
            raise ValueError('HEAD changed since batch initialization')
        if git(root, 'rev-parse', 'HEAD^').decode().strip() != receipt['head']:
            raise ValueError('commit parent differs from the validated checkpoint')
        if git(root, 'rev-parse', 'HEAD^{tree}').decode().strip() != receipt['staged_tree']:
            raise ValueError('committed tree differs from the reviewed index')
    if git(root, 'branch', '--show-current').decode().strip() != receipt['branch']:
        raise ValueError('branch changed')
    if {str(p.resolve()) for p in source_paths(root, receipt['allow'])} != set(receipt['source']):
        raise ValueError('source file set changed; initialize a new validation batch')
    verify_identities(receipt['source'])
    if check_protected:
        verify_identities(receipt['protected'])
    verify_identities(receipt.get('allow_identity', {}))
    if receipt['commands'] != commands(root) or receipt['toolchain'] != toolchain():
        raise ValueError('validation plan or toolchain changed; initialize a new batch')
    for name, check in receipt['checks'].items():
        if check.get('exit_code') == 0 and digest(check['log']) != check['log_sha256']:
            raise ValueError(f'validation log changed: {name}')
    return receipt


def run(root, directory, selected=None):
    path, lock = directory / 'receipt.json', directory / 'running.json'
    if lock.exists():
        active = read_json(lock)
        if any(process_alive(pid) for pid in active['pids']):
            raise ValueError('the recorded batch process is still alive; wait for that process')
        lock.unlink()
    write_json(lock, dict(pids=[os.getpid()], started=now()))
    try:
        receipt = verify(root, read_json(path))
        selected = selected or list(receipt['commands'])
        if not set(selected) <= set(receipt['commands']):
            raise ValueError('unknown check name')
        for name in selected:
            verify(root, receipt, check_protected=False)
            previous = receipt['checks'].get(name, {})
            if previous.get('exit_code') == 0:
                print(f'REUSE {name}: identical inputs and log', flush=True)
                continue
            attempt = int(previous.get('attempt', 0)) + 1
            log = directory / f'{name}-{attempt:02}.log'
            while log.exists():
                attempt += 1
                log = directory / f'{name}-{attempt:02}.log'
            argv = receipt['commands'][name].copy()
            if '-File' in argv:
                argv += ['-MetricsPath', str(directory / f'{name}-{attempt:02}.metrics.json')]
            check = dict(attempt=attempt, argv=argv, started=now(), log=str(log))
            receipt['checks'][name] = check
            write_json(path, receipt, replace=True)
            print(f'START {name}', flush=True)
            start = time.monotonic()
            with log.open('xb') as stream:
                process = subprocess.Popen(argv, cwd=root, stdout=stream, stderr=subprocess.STDOUT,
                                           creationflags=0x08000000 if os.name == 'nt' else 0)
                write_json(lock, dict(pids=[os.getpid(), process.pid], started=now()), replace=True)
                observed_exit = process.wait()
            write_json(lock, dict(pids=[os.getpid()], started=now()), replace=True)
            check.update(seconds=round(time.monotonic() - start, 2), finished=now(),
                         observed_exit_code=observed_exit, log_sha256=digest(log))
            write_json(path, receipt, replace=True)
            try:
                verify(root, receipt)
            except (ValueError, OSError, KeyError) as error:
                check['validation_error'] = str(error)
                write_json(path, receipt, replace=True)
                raise
            # Publish success only after checking the exact inputs used by this attempt.
            # An interrupted or invalidated attempt has no reusable exit_code.
            check['exit_code'] = observed_exit
            write_json(path, receipt, replace=True)
            print(f"END {name}: {check['exit_code']}", flush=True)
            if check['exit_code']:
                return False
        verify(root, receipt)
        return True
    finally:
        # If interrupted while a child remains live, retain its authoritative PID.
        if lock.exists() and not any(process_alive(pid) for pid in read_json(lock)['pids'] if pid != os.getpid()):
            lock.unlink()


def stage(root, directory):
    path = directory / 'receipt.json'
    receipt = verify(root, read_json(path))
    if set(receipt['checks']) != set(receipt['commands']) or any(v.get('exit_code') != 0 for v in receipt['checks'].values()):
        raise ValueError('all seven checks must pass before staging')
    if git(root, 'diff', '--cached', '--name-only'):
        raise ValueError('index must be empty')
    # Do not allow an unrelated working-tree change to slip into the review later.
    allow = allowed_paths(root, receipt['allow'])
    if any(not (root / n).is_file() for n in allow):
        raise ValueError('this source-only staging flow requires every allowlisted file to exist')
    pathspec = directory / 'include.pathspec'
    if pathspec.exists():
        raise ValueError('staging receipt already exists')
    pathspec.write_bytes(b''.join(b':(literal)' + n.encode() + b'\0' for n in allow))
    subprocess.run(['git', 'add', '--pathspec-from-file=' + str(pathspec), '--pathspec-file-nul'], cwd=root, check=True)
    staged = set(filter(None, git(root, 'diff', '--cached', '--no-renames', '--name-only', '-z').decode().split('\0')))
    if staged != set(allow):
        raise ValueError('staged paths differ from the allowlist; inspect the index before continuing')
    subprocess.run(['git', 'diff', '--cached', '--check'], cwd=root, check=True)
    verify(root, receipt)
    for name in allow:
        if git(root, 'hash-object', '--path=' + name, name) != git(root, 'rev-parse', ':' + name):
            raise ValueError(f'index no longer matches the validated source: {name}')
    receipt['allow_identity'] = identities([root / name for name in allow])
    receipt['staged_tree'] = git(root, 'write-tree').decode().strip()
    receipt['reviewed'] = now()
    write_json(path, receipt, replace=True)
    return receipt
