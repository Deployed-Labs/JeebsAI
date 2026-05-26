#!/usr/bin/env python3
import importlib
import os
import unittest
from pathlib import Path

import app.models as models


class TestVpsDeployConfig(unittest.TestCase):
    def tearDown(self):
        os.environ.pop('DATABASE_PATH', None)
        importlib.reload(models)

    def test_database_path_defaults_to_local_file(self):
        importlib.reload(models)
        self.assertEqual(models.DB_PATH, Path('./jeebs.db'))

    def test_database_path_reads_env_var(self):
        os.environ['DATABASE_PATH'] = '/tmp/jeebs-test.db'
        importlib.reload(models)
        self.assertEqual(models.DB_PATH, Path('/tmp/jeebs-test.db'))

if __name__ == '__main__':
    unittest.main(verbosity=2)
