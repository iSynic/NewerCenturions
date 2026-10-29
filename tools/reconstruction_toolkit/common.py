"""File identities, bounded subprocesses and private report storage."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile
from datetime import datetime, timezone


ROOT = Path(__file__).resolve().parents[2]


def now():
    return datetime.now(timezone.utc).isoformat()


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def read_json(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))


def private_path(root, name):
    path = (root / name).resolve()
    area = (root / 'analysis').resolve()
    if area != root.resolve() / 'analysis':
        raise ValueError('analysis must be inside the checkout, without an external symlink')
    if path == area or not path.is_relative_to(area):
        raise ValueError('generated outputs must be inside this checkout\'s analysis directory')
    return path


def write_json(path, value, replace=False):
    path = Path(path)
    if path.exists() and not replace:
        raise ValueError(f'refusing to overwrite {path}')
    path.parent.mkdir(parents=True, exist_ok=True)
    handle, temporary = tempfile.mkstemp(suffix='.tmp', dir=path.parent)
    try:
        with os.fdopen(handle, 'w', encoding='utf-8', newline='\n') as stream:
            json.dump(value, stream, indent=2, ensure_ascii=False)
            stream.write('\n')
        if replace:
            os.replace(temporary, path)
        else:
            # A hard link publishes the completed file without replacing a concurrent writer.
            os.link(temporary, path)
    finally:
        Path(temporary).unlink(missing_ok=True)


def git(root, *args):
    return subprocess.check_output(['git', *args], cwd=root)


def checkout_files(root):
    return sorted({name for name in git(root, 'ls-files', '-co', '--exclude-standard', '-z')
                   .decode('utf-8').split('\0') if name})


def identities(paths):
    return {str(Path(path).resolve()): digest(path) for path in sorted(set(map(Path, paths)))}


def verify_identities(expected):
    changed = [name for name, sha in expected.items()
               if not Path(name).is_file() or digest(name) != sha]
    if changed:
        raise ValueError('input identities changed: ' + ', '.join(changed[:12]))


def host():
    return dict(system=platform.system(), release=platform.release(), machine=platform.machine(),
                python=platform.python_version())


def process_alive(pid):
    if os.name == 'nt':
        import ctypes
        api = ctypes.windll.kernel32
        api.OpenProcess.restype = ctypes.c_void_p
        handle = api.OpenProcess(0x1000, False, int(pid))
        if not handle:
            if api.GetLastError() == 5:
                return True
            return False
        try:
            status = ctypes.c_ulong()
            if not api.GetExitCodeProcess(ctypes.c_void_p(handle), ctypes.byref(status)):
                raise OSError('cannot determine process state')
            return status.value == 259
        finally:
            api.CloseHandle(ctypes.c_void_p(handle))
    try:
        os.kill(int(pid), 0)
        return True
    except ProcessLookupError:
        return False
    except PermissionError:
        return True


def run_logged(argv, root, output, timeout=None):
    output = Path(output)
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open('xb') as stream:
        process = subprocess.Popen(list(map(str, argv)), cwd=root, stdout=stream,
                                   stderr=subprocess.STDOUT,
                                   creationflags=0x08000000 if os.name == 'nt' else 0)
        try:
            return process.wait(timeout=timeout)
        except BaseException:
            process.terminate()
            process.wait()
            raise
