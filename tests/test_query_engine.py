from __future__ import annotations

import unittest

from src.query_engine import QueryEnginePort


class QueryEngineTests(unittest.TestCase):
    def test_query_engine_summary_mentions_workspace(self) -> None:
        summary = QueryEnginePort.from_workspace().render_summary()
        self.assertIn('Python Porting Workspace Summary', summary)
        self.assertIn('Command surface:', summary)
        self.assertIn('Tool surface:', summary)


if __name__ == '__main__':
    unittest.main()
