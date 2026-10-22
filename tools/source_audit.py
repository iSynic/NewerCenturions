"""Source token accounting and native-case preservation checks."""
from pathlib import Path
import re


def mask_rust(text: str, strings: bool = False) -> str:
    out = list(text)
    raw_pattern = re.compile(r'(?:b)?r(#{0,255})"')
    char_pattern = re.compile(r"'(?:[^'\\\r\n]|\\(?:[nrt0\\'\"]|x[0-9a-fA-F]{2}|u\{[0-9a-fA-F_]+\}))'")
    i = 0
    while i < len(text):
        start = i
        if text.startswith('//', i):
            end = text.find('\n', i)
            end = len(text) if end < 0 else end
        elif text.startswith('/*', i):
            depth, end = 1, i + 2
            while depth and end < len(text):
                if text.startswith('/*', end):
                    depth += 1
                    end += 2
                elif text.startswith('*/', end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
        else:
            raw = raw_pattern.match(text, i) if text[i] in 'br' else None
            if raw:
                close = '"' + raw[1]
                end = text.find(close, i + len(raw[0]))
                end = len(text) if end < 0 else end + len(close)
            elif text[i] == '"':
                end = i + 1
                while end < len(text):
                    if text[end] == '\\':
                        end += 2
                    elif text[end] == '"':
                        end += 1
                        break
                    else:
                        end += 1
            elif text[i] == "'" and (char := char_pattern.match(text, i)):
                end = char.end()
            else:
                i += 1
                continue
            if not strings:
                i = end
                continue
        for n in range(start, end):
            if text[n] not in '\r\n':
                out[n] = ' '
        i = end
    return ''.join(out)


def matching(mask: str, start: int, opening: str = '{', closing: str = '}') -> int:
    depth = 1
    for i in range(start + 1, len(mask)):
        if mask[i] == opening:
            depth += 1
        elif mask[i] == closing:
            depth -= 1
            if depth == 0:
                return i + 1
    raise ValueError(f'Unclosed {opening} at {start}')


def items(text: str):
    mask = mask_rust(text, True)
    i = 0
    while i < len(mask):
        start = i
        while i < len(mask) and mask[i].isspace():
            i += 1
        if i == len(mask):
            break
        while mask.startswith('#[', i):
            i = matching(mask, i + 1, '[', ']')
            while i < len(mask) and mask[i].isspace():
                i += 1
        header = i
        parens = brackets = 0
        while i < len(mask):
            c = mask[i]
            if c == '(':
                parens += 1
            elif c == ')':
                parens -= 1
            elif c == '[':
                brackets += 1
            elif c == ']':
                brackets -= 1
            elif not parens and not brackets and c == '{':
                i = matching(mask, i)
                if i < len(mask) and mask[i] == ';':
                    i += 1
                break
            elif not parens and not brackets and c == ';':
                i += 1
                break
            i += 1
        yield start, i, text[start:i], mask[header:i]


def code_lines(text: str) -> int:
    return sum(bool(line.strip()) for line in mask_rust(text).splitlines())


def source_lines(text, suffix):
    if suffix == ".rs":
        return code_lines(text)
    if suffix == ".py":
        import io
        import tokenize
        occupied = set()
        for token in tokenize.generate_tokens(io.StringIO(text).readline):
            if token.type not in (tokenize.COMMENT, tokenize.NL, tokenize.NEWLINE,
                                  tokenize.INDENT, tokenize.DEDENT, tokenize.ENDMARKER,
                                  tokenize.ENCODING):
                occupied.update(range(token.start[0], token.end[0] + 1))
        return len(occupied)
    return code_lines("\n".join(line for line in text.splitlines()
                                if not line.lstrip().startswith("#")))


def accounting(root):
    totals = {"production": {"physical": 0, "code": 0},
              "tests": {"physical": 0, "code": 0}}
    for part in ("maintained", "native", "reference"):
        totals[part] = {"production": {"physical": 0, "code": 0},
                        "tests": {"physical": 0, "code": 0}}
    inventory = []
    for name in ("crates/content", "crates/core", "crates/desktop", "archive/crates/reference"):
        for path in sorted((root / name / "src").rglob("*.rs")):
            if path.name == "catalog.rs":
                continue
            text = path.read_text(encoding="utf-8")
            test_parts = []
            for start, end, content, header in items(text):
                if re.search(r"#\[cfg\([^\n]*\btest\b", content[:content.find("\nmod ") + 1] if "\nmod " in content else content[:200]):
                    test_parts.append((start, end, content))
            production = text
            for start, end, content in reversed(test_parts):
                production = production[:start] + production[end:]
                add_count(totals, inventory, path.relative_to(root), content, "tests")
            add_count(totals, inventory, path.relative_to(root), production, "production")
    test_roots = [root / "tests", root / "vendor/miniquad/tests"] + [root / name / "tests"
                                     for name in ("crates/core", "crates/content", "crates/desktop",
                                                  "archive/crates/reference")]
    for test_root in test_roots:
        for path in sorted(test_root.rglob("*")):
            if path.is_file() and path.suffix in (".rs", ".py", ".ps1", ".java"):
                add_count(totals, inventory, path.relative_to(root), path.read_text(encoding="utf-8"), "tests")
    for name in ("Run-Tests.ps1", "Run-Cargo.ps1", "Build.ps1", "build_portable.py", "source_audit.py", "engine_inventory.py", "check_menu_boundary.py",
                 "audit_public.py", "oracle_cache.py", "capture_reference.py", "reference.py",
                 "reference_debug.py", "registration_status.py", "verify_mace3_audio.py",
                 "Setup-Reference.ps1"):
        path = root / "tools" / name
        if path.is_file():
            add_count(totals, inventory, path.relative_to(root), path.read_text(encoding="utf-8"), "tests")
    native = root / "tools" / "ghidra"
    probes = set(native.glob("Verify*.java"))
    probes.update(native / name for name in ("ActionOracleHarness.java", "RecordNativeContext.java",
                  "ResultAudioBoundary.java", "SaneBoundary.java", "SubmitMusicBoundary.java"))
    for path in sorted(probes):
        if path.is_file():
            add_count(totals, inventory, path.relative_to(root), path.read_text(encoding="utf-8"), "tests")
    # The combined ratio is the historical single measure, kept for continuity.
    totals["ratio"] = totals["tests"]["code"] / totals["production"]["code"]
    for part in ("maintained", "reference"):
        production = totals[part]["production"]["code"]
        totals[part]["ratio"] = totals[part]["tests"]["code"] / production if production else 0.0
    # Native evidence tests maintained code against original-binary fixtures.
    totals["native"]["ratio"] = totals["native"]["tests"]["code"] / totals["maintained"]["production"]["code"]
    totals["native"]["fixtures"] = sum(1 for path in (root / "evidence" / "fixtures").glob("*.json")
                                       if "provenance" not in path.name)
    totals["target"] = TEST_TARGET
    totals["maintained_test_ceiling"] = MAINTAINED_TEST_CEILING
    totals["passed"] = totals["maintained"]["tests"]["code"] <= MAINTAINED_TEST_CEILING
    return totals, inventory


# Reported target for maintained test code relative to maintained production code.
# Native-evidence comparisons (crates/core/tests/native) are reported separately.
TEST_TARGET = 0.25
# Ratchet on maintained test code lines: they may not grow. Lower it whenever
# consolidation removes lines. A count rather than a ratio, so that simplifying
# production code never fails the gate.
MAINTAINED_TEST_CEILING = 6827

# Recovered execution bodies, the routine catalog and expanded translations, all in the
# private archive workspace.
REFERENCE_SOURCE = re.compile(
    r"^(crates/core/src/(recovered_[^/]+\.rs|reconstruction/)|archive/crates/(reference|translated[^/]*)/src/)")
# Helpers used only by native-fixture comparisons. Shared support (support/mod.rs,
# campaign helpers and the native_reference runner) stays maintained.
NATIVE_EVIDENCE_HELPERS = {
    "crates/core/tests/enemy_callers.rs",
    "crates/core/tests/native_common.rs",
    "crates/core/tests/snapshots.rs",
    "crates/core/tests/support/native_context.rs",
    "crates/core/tests/support/native_save_receipts.rs",
    "crates/core/tests/support/psi_frame.rs",
}
# Verification that compares against or drives the reference layer or the
# original binary, rather than testing maintained behavior.
REFERENCE_TESTS = {
    "tools/Setup-Reference.ps1",
    "tools/capture_reference.py",
    "tools/oracle_cache.py",
    "tools/reference.py",
    "tools/reference_debug.py",
    "tools/registration_status.py",
    "tests/python/test_reference_capture.py",
    "tests/python/test_translation_plan.py",
}


RAW_OFFSET = re.compile(r"bytes\[0x|word\(&[^,]*bytes, 0x|put_word\(&mut [^,]*bytes, 0x")


def raw_offsets(root):
    """Hex record offsets belong in content::records views, persistence or native_save."""
    return [f"{path.relative_to(root).as_posix()}:{number}"
            for path in sorted((root / "crates/core/src").rglob("*.rs"))
            if not {"persistence", "native_save.rs"} & set(path.parts)
            for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1)
            if RAW_OFFSET.search(line)]


def layer(path, role):
    name = str(path).replace("\\", "/")
    if role == "production":
        return "reference" if REFERENCE_SOURCE.match(name) else "maintained"
    if (name in REFERENCE_TESTS or name.startswith("tools/ghidra/")
            or name.startswith("archive/crates/reference/tests/")):
        return "reference"
    if name.startswith("crates/core/tests/native/") or name in NATIVE_EVIDENCE_HELPERS:
        return "native"
    return "maintained"


def add_count(totals, inventory, path, text, role):
    count = {"physical": len(text.splitlines()), "code": source_lines(text, path.suffix)}
    part = layer(path, role)
    inventory.append({"path": str(path).replace("\\", "/"), "role": role, "layer": part, **count})
    for key, value in count.items():
        totals[role][key] += value
        totals[part][role][key] += value


def preserved(root, manifest):
    import hashlib
    failures = []
    for group in ("fixtures", "originals"):
        for name, expected in manifest[group].items():
            path = root / name
            if not path.is_file():
                failures.append(f"Missing {name}")
                continue
            with path.open("rb") as source:
                digest = hashlib.file_digest(source, "sha256").hexdigest()
            expected_hash = expected if isinstance(expected, str) else expected["sha256"]
            expected_size = None if isinstance(expected, str) else expected["bytes"]
            if digest != expected_hash or expected_size is not None and path.stat().st_size != expected_size:
                failures.append(f"Changed {name}")
    return failures


def qualify_entries(result, receipt_path):
    """Apply a reviewed source-domain closure only to its exact inputs and graph."""
    import hashlib
    import json
    receipt = json.loads(receipt_path.read_text(encoding="utf-8-sig"))
    project = Path(__file__).resolve().parent.parent
    for name, digest in receipt["source_hashes"].items():
        if hashlib.sha256((project / name).read_bytes()).hexdigest() != digest:
            raise ValueError(f"Stale script entry qualification: {name}")
    graph = json.dumps({key: result[key] for key in ("nodes", "edges")},
                       sort_keys=True, separators=(",", ":")).encode()
    if hashlib.sha256(graph).hexdigest() != receipt["graph_sha256"]:
        raise ValueError("Script entry qualification does not match the current graph")
    classifications = {entry["node"]: entry for entry in receipt["not_selected"]}
    if set(classifications) != set(result["unqualified_nodes"]):
        raise ValueError("Script entry qualification does not cover the unselected records")
    result["unselected_nodes"] = result["unqualified_nodes"]
    result["unqualified_nodes"] = []
    result["entry_classifications"] = classifications
    result["entry_qualification"] = dict(receipt=receipt_path.as_posix(),
                                        scope=receipt["classification_scope"],
                                        remaining=receipt["remaining"])
    result["qualification"] = ("Reviewed source-domain dispatch closure; conditional activation, "
                               "native effects and live acceptance remain separate. No campaign route "
                               "absence is used as unreachability proof.")
    return result


def main():
    import argparse
    import json
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--report", type=Path)
    parser.add_argument("--preserve", type=Path)
    options = parser.parse_args()
    totals, inventory = accounting(options.root)
    report = {"totals": totals, "files": inventory}
    if options.preserve:
        report["preservation_failures"] = preserved(options.root, json.loads(options.preserve.read_text()))
    if options.report:
        options.report.parent.mkdir(parents=True, exist_ok=True)
        options.report.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(totals, indent=2))
    # Every maintained diagnostic needs a judgment and every catalog binding a file. The
    # public source has neither the private evidence nor the reference archive.
    if (options.root / "evidence").is_dir() and (options.root / "archive").is_dir():
        from engine_inventory import build, problems
        inventory_problems = problems(build(options.root / "NewCenturions"))
    else:
        inventory_problems = []
        print("Engine inventory skipped: evidence/ or archive/ is not in this checkout")
    offsets = [f"Raw record offset outside records: {site}" for site in raw_offsets(options.root)]
    for problem in inventory_problems + offsets:
        print(problem)
    failed = not totals["passed"] or report.get("preservation_failures") or inventory_problems or offsets
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
