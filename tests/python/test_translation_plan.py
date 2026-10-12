import unittest

from tools.plan_translation import build_plan, dependency_batches, rust_catalog


def routine(entry, ranges=None, calls=()):
    return {"entry": f"{entry:08x}", "name": f"routine_{entry:x}", "body_bytes": 4,
            "ranges": ranges or [{"start": f"{entry:08x}", "end": f"{entry + 3:08x}"}],
            "calls": [{"entry": f"{called:08x}", "name": "called"} for called in calls],
            "decompiled": True,
            "blocks": [{"entry": f"{entry:08x}", "destinations": [], "instructions": [
                {"address": f"{entry:08x}", "operations": [{"opcode": "INT_ADD",
                    "output": {"bytes": 2}, "inputs": [{"bytes": 2}, {"bytes": 2}]}]}]}]}


class TranslationPlanTests(unittest.TestCase):
    def test_dependency_layers_keep_cycles_together_and_wait_for_callees(self):
        self.assertEqual(dependency_batches({1: {2}, 2: {3}, 3: {2, 4}, 4: set(), 5: {1}, 6: set()}),
                         [[4, 6], [2, 3], [1], [5]])

    def test_rust_ownership_uses_all_actual_ranges_not_compact_body_size(self):
        recovered = routine(0x1000, [{"start": "1000", "end": "1001"},
                                    {"start": "2000", "end": "2001"}])
        registry = {"actions": [
            {"id": "implemented", "mapped_entry_addresses": ["2001"], "stage": "implemented",
             "implementation": "crates/core/src/example.rs", "case_count": 3},
            {"id": "gap", "mapped_entry_addresses": ["1002"], "stage": "integrated"},
            {"id": "only_recovered", "mapped_entry_addresses": ["1001"], "stage": "discovered"}]}
        row = build_plan({"functions": [recovered], "scope": "synthetic"}, registry)["routines"][0]
        self.assertEqual(row["status"], "partial")
        self.assertEqual([b["action"] for b in row["rust_bindings"]], ["implemented"])
        self.assertEqual(row["native_comparison"], "scoped_cases")

    def test_recovery_stays_distinct_from_rust_and_external_calls(self):
        plan = build_plan({"functions": [routine(0x1000, calls=[0x2000, 0x3000]), routine(0x2000)],
                           "scope": "synthetic"}, {"actions": []})
        self.assertEqual(plan["summary"]["translation"], {"recovered": 2})
        self.assertEqual(plan["summary"]["operations"], 2)
        self.assertEqual(plan["routines"][0]["external_calls"], [0x3000])
        self.assertEqual(plan["dependency_batches"], [[0x2000], [0x1000]])
        self.assertEqual(plan["actions"][0]["steps"][0]["operand_bytes"], [2])
        catalog = rust_catalog(plan)
        self.assertIn("Translation::Recovered", catalog)
        self.assertNotIn("INT_ADD", catalog)
        self.assertNotIn("opcode", catalog)

    def test_duplicate_routine_entries_fail(self):
        with self.assertRaisesRegex(ValueError, "Duplicate"):
            build_plan({"functions": [routine(0x1000), routine(0x1000)], "scope": "synthetic"},
                       {"actions": []})

    def test_python_tooling_is_recovery_and_mixed_notes_are_not_rust_paths(self):
        registry = {"actions": [
            {"id": "python", "mapped_entry_addresses": ["1000"], "stage": "implemented",
             "implementation": "tools/decode_globals.py"},
            {"id": "rust", "mapped_entry_addresses": ["2000"], "stage": "integrated",
             "implementation": "crates/core/src/equipment.rs; immutable Data S"}]}
        plan = build_plan({"functions": [routine(0x1000), routine(0x2000)], "scope": "synthetic"}, registry)
        first, second = plan["routines"]
        self.assertEqual(first["status"], "recovered")
        self.assertEqual(first["rust_bindings"], [])
        self.assertEqual(first["recovery_bindings"][0]["action"], "python")
        self.assertEqual(second["rust_bindings"][0]["rust"], ["crates/core/src/equipment.rs"])
        self.assertNotIn("immutable", rust_catalog(plan))
