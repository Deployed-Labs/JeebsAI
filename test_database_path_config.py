#!/usr/bin/env python3
import importlib
import os
import unittest
from pathlib import Path
from unittest import mock

import app.models as models


class DatabasePathConfigTests(unittest.TestCase):
    def test_default_database_path(self):
        with mock.patch.dict(os.environ, {}, clear=True):
            reloaded = importlib.reload(models)
            expected = reloaded.BASE_DIR / "jeebs.db"
            self.assertEqual(reloaded.DB_PATH, expected)

    def test_env_database_path(self):
        with mock.patch.dict(os.environ, {"DATABASE_PATH": "./data/custom.db"}, clear=True):
            reloaded = importlib.reload(models)
            expected = (reloaded.BASE_DIR / Path("data/custom.db")).resolve()
            self.assertEqual(reloaded.DB_PATH, expected)


if __name__ == "__main__":
    unittest.main()
