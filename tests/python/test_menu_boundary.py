import copy
import json
import unittest
from pathlib import Path

from tools.check_menu_boundary import verify_menu_boundary


class MenuBoundaryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        root = Path(__file__).resolve().parents[2]
        cls.fixture = json.loads((root / "evidence/fixtures/menu-boundary.json").read_text())

    def test_original_resources_and_eleven_native_boundaries(self):
        self.assertEqual(
            verify_menu_boundary(),
            {"cases": 11, "fields": 28, "toolbox_replies": 148, "music_replies": 3},
        )

    def rejects(self, name, change):
        altered = copy.deepcopy(self.fixture)
        case = next(row for row in altered["cases"] if row["case"] == name)
        change(case)
        with self.assertRaises(AssertionError):
            verify_menu_boundary(altered)

    def test_speed_delay_uses_original_five_tick_steps(self):
        self.rejects("speed_11", lambda case: case["after"].update({"439568": 49}))

    def test_drawing_preserves_existing_menu_items(self):
        self.rejects("draw_bar", lambda case: case["after"]["menus"]["200"]["items"].pop())

    def test_shift_playlist_updates_actual_selected_byte(self):
        self.rejects("playlist_on", lambda case: case["after"].update({"playlist13": "00" * 13}))

    def test_music_stop_preserves_command_order(self):
        self.rejects("music_stop", lambda case: case["music_native"][0]["services"].reverse())

    def test_command_key_dispatch_matches_enabled_original_shortcut(self):
        def wrong_selection(case):
            reply = next(row for row in case["services"] if row.get("trap") == "a93e")
            reply["selection"] = (129 << 16) | 2

        self.rejects("command_key_load", wrong_selection)


if __name__ == "__main__":
    unittest.main()
