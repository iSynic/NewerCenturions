"""Run the retained native menu reference checks."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tests/python"))
from native_menu_reference import verify_menu_boundary

if __name__ == "__main__":
    print("PASS menu boundary:", verify_menu_boundary())
