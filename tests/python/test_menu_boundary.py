import unittest

from native_menu_reference import verify_menu_boundary


class MenuBoundaryTests(unittest.TestCase):
    def test_original_resources_and_eleven_native_boundaries(self):
        verify_menu_boundary()


if __name__ == "__main__":
    unittest.main()
