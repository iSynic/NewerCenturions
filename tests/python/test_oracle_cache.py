import copy
import unittest

from tools.oracle_cache import reusable


class OracleCacheTests(unittest.TestCase):
    def test_each_behavior_authority_dimension_invalidates_reuse(self):
        cached = {
            "executable": "code-a",
            "inputs": {"Data ED": "input-a"},
            "probe_and_boundary_definitions": {"Probe.java": "probe-a", "SaneBoundary.java": "boundary-a"},
            "fixture": "fixture-a",
            "toolchain": {"ghidra": "12.1.2", "application_properties": "build-a"},
        }
        self.assertTrue(reusable(cached, copy.deepcopy(cached)))
        for key, value in [
            ("executable", "code-b"),
            ("inputs", {"Data ED": "input-b"}),
            ("probe_and_boundary_definitions", {"Probe.java": "probe-b", "SaneBoundary.java": "boundary-a"}),
            ("probe_and_boundary_definitions", {"Probe.java": "probe-a", "SaneBoundary.java": "boundary-b"}),
            ("fixture", "fixture-b"),
            ("toolchain", {"ghidra": "12.1.3", "application_properties": "build-b"}),
        ]:
            changed = copy.deepcopy(cached)
            changed[key] = value
            self.assertFalse(reusable(cached, changed), key)
