from __future__ import annotations

import unittest

from src.port_manifest import build_port_manifest


class ManifestTests(unittest.TestCase):
    def test_manifest_counts_python_files(self) -> None:
        manifest = build_port_manifest()
        self.assertGreaterEqual(manifest.total_python_files, 20)
        self.assertTrue(manifest.top_level_modules)


if __name__ == '__main__':
    unittest.main()
