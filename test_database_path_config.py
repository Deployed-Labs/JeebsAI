#!/usr/bin/env python3
import importlib
import os
import unittest
from unittest import mock

import app.models as models


class DatabasePathConfigTests(unittest.TestCase):
    def test_default_database_path(self):
        with mock.patch.dict(os.environ, {}, clear=True):
            reloaded = importlib.reload(models)
            self.assertEqual(str(reloaded.DB_PATH), "jeebs.db")

    def test_env_database_path(self):
        with mock.patch.dict(os.environ, {"DATABASE_PATH": "./data/custom.db"}, clear=True):
            reloaded = importlib.reload(models)
            self.assertEqual(str(reloaded.DB_PATH), "data/custom.db")


if __name__ == "__main__":
    unittest.main()
