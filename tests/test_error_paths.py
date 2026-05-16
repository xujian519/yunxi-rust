from __future__ import annotations

import subprocess
import sys
import unittest


class ErrorPathTests(unittest.TestCase):
    def test_unknown_tool_returns_unhandled(self) -> None:
        from src.tools import execute_tool

        result = execute_tool('NonExistentToolXYZ', 'test payload')
        self.assertFalse(result.handled)
        self.assertIn('Unknown mirrored tool', result.message)

    def test_get_tool_returns_none_for_unknown(self) -> None:
        from src.tools import get_tool

        self.assertIsNone(get_tool('absolutely_not_a_real_tool_name'))

    def test_cli_invalid_arguments_nonzero_exit(self) -> None:
        result = subprocess.run(
            [sys.executable, '-m', 'src.main'],
            capture_output=True,
            text=True,
        )
        self.assertNotEqual(result.returncode, 0)


if __name__ == '__main__':
    unittest.main()
