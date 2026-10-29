"""Find literal rule diagnostics and follow their Rust helper imports.

This is a source census, not a Rust reachability analysis. Parameterized error
builders are expanded only at callers with literal action/dependency arguments.
"""
import re
from collections import defaultdict
from pathlib import Path

LITERAL = re.compile(r'Unsupported\s*\{\s*action:\s*"([^"]*)",\s*dependency:\s*"([^"]*)"')
FUNCTION = re.compile(r'\bfn\s+(\w+)(?:<[^{};]*?>)?\s*\(([^)]*)\)[^{;]*\{')
CONSTRUCTOR = re.compile(r'(?:RuleError|Self)::Unsupported\s*\{([^}]+)\}')
USE = re.compile(r'\buse\s+([^;]+);')
TOKEN = re.compile(r'//[^\n]*|/\*.*?\*/|"(?:\\.|[^"\\])*"|\x27(?:\\.|[^\x27\\])\x27', re.S)


def mask(text):
    return TOKEN.sub(lambda m: ''.join('\n' if c == '\n' else ' ' for c in m[0]), text)


def matching(text, start, opening='{', closing='}'):
    depth = 0
    for position in range(start, len(text)):
        depth += text[position] == opening
        depth -= text[position] == closing
        if depth == 0:
            return position
    raise ValueError('unbalanced Rust source')


def split_top(text):
    depth, start, values = 0, 0, []
    for i, char in enumerate(mask(text)):
        if char in '({[<':
            depth += 1
        elif char in ')}]>':
            depth -= 1
        elif char == ',' and depth == 0:
            values.append(text[start:i].strip())
            start = i + 1
    values.append(text[start:].strip())
    return [v for v in values if v]


def use_paths(expression, prefix=''):
    """Flatten grouped use trees, retaining explicit aliases."""
    for part in split_top(expression):
        if '{' in part:
            head, tail = part.split('{', 1)
            yield from use_paths(tail.rsplit('}', 1)[0], prefix + head)
        else:
            path, _, alias = part.partition(' as ')
            path = prefix + path.strip()
            if path.endswith('::self'):
                path = path[:-6]
            yield alias.strip() or path.rsplit('::', 1)[-1], path


def module_key(path):
    parts = Path(path).parts
    index = parts.index('src')
    tail = list(parts[index + 1:])
    tail[-1] = Path(tail[-1]).stem
    if tail[-1] in ('lib', 'mod'):
        tail.pop()
    return tuple(parts[:index]) + ('src',) + tuple(tail)


def qualify(module, path):
    pieces = path.split('::')
    base = list(module)
    if pieces[0] == 'crate':
        base = base[:base.index('src') + 1]
        pieces.pop(0)
    while pieces and pieces[0] == 'super':
        base.pop()
        pieces.pop(0)
    if pieces and pieces[0] == 'self':
        pieces.pop(0)
    return tuple(base + pieces)


def field(body, name, parameters):
    found = re.search(r'\b' + name + r'\s*:\s*("[^"]*"|\w+)', body)
    value = found[1] if found else name if re.search(r'\b' + name + r'\s*[,}]?', body) else None
    if value and value.startswith('"'):
        return ('literal', value[1:-1])
    if value in parameters:
        return ('parameter', parameters.index(value))
    return None


def scan(raw):
    """Return pairs with locations and unresolved maintained helper calls."""
    modules = {module_key(path): path for path in raw}
    imports, builders, rows, unresolved = {}, {}, defaultdict(list), []
    for module, path in modules.items():
        text = raw[path]
        code = mask(text)
        imports[module] = {alias: qualify(module, target)
                           for declaration in USE.finditer(code)
                           for alias, target in use_paths(text[declaration.start(1):declaration.end(1)])}
        functions = [(m, matching(code, m.end() - 1)) for m in FUNCTION.finditer(code)]
        for m in CONSTRUCTOR.finditer(text):
            if not code[m.start():m.start()+9].strip():
                continue
            owners = [(f, end) for f, end in functions if f.end() <= m.start() < end]
            if not owners:
                continue
            f, _ = max(owners, key=lambda pair: pair[0].start())
            params = [p.split(':', 1)[0].strip().removeprefix('mut ') for p in split_top(f[2])]
            action, dependency = field(m[1], 'action', params), field(m[1], 'dependency', params)
            if action and dependency and ('parameter' in (action[0], dependency[0])):
                builders[module + (f[1],)] = (action, dependency)

    def add(pair, path, position):
        site = {'file': path, 'line': raw[path][:position].count('\n') + 1}
        if site not in rows[pair]:
            rows[pair].append(site)

    def resolve(module, path):
        seen = {}
        current = qualify(module, path)
        while (prefix := next((i for i in range(len(current) - 1, 0, -1)
                               if current[i] in imports.get(current[:i], {}) and (i == len(current) - 1 or current[:i + 1] not in modules)), None)) is not None:
            binding = current[:prefix + 1]
            if binding in seen and len(current) >= seen[binding]:
                raise ValueError('cyclic helper import: ' + '::'.join(current))
            seen[binding] = len(current)
            current = imports[current[:prefix]][current[prefix]] + current[prefix + 1:]
        return current

    names = sorted({key[-1] for key in builders} | {alias for aliases in imports.values() for alias in aliases})
    calls = re.compile(r'\b((?:\w+::)*(?:' + '|'.join(map(re.escape, names)) + r'))\s*\(') if names else None
    for module, path in modules.items():
        text, code = raw[path], mask(raw[path])
        for m in LITERAL.finditer(text):
            if code[m.start():m.start()+11].strip():
                add(m.groups(), path, m.start())
        if not calls:
            continue
        for m in calls.finditer(code):
            if re.search(r'\bfn\s*$', code[max(0,m.start()-8):m.start()]):
                continue
            builder = builders.get(resolve(module, m[1]))
            if builder is None:
                continue
            end = matching(code, m.end()-1, '(', ')')
            args = split_top(text[m.end():end])
            pair = []
            for kind, value in builder:
                arg = args[value] if kind == 'parameter' and value < len(args) else None
                if kind == 'literal':
                    pair.append(value)
                elif arg and re.fullmatch(r'"[^"\\]*"', arg):
                    pair.append(arg[1:-1])
                else:
                    unresolved.append({'file': path, 'line': text[:m.start()].count('\n')+1,
                                       'helper': m[1]})
                    break
            if len(pair) == 2:
                add(tuple(pair), path, m.start())
    return rows, unresolved
