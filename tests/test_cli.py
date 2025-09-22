import unittest
import subprocess


class CLITests(unittest.TestCase):
    def test_missing_paths(self):
        process = subprocess.run(
            ["cloneholio", "-p", "gitlab", "-t", "123"], stderr=subprocess.PIPE
        )
        self.assertEqual(process.returncode, 2)
        self.assertEqual(
            process.stderr.decode().split("\n")[-2],
            "cloneholio: error: must specifiy at least --all-groups or a path(s)",  # noqa: E501
        )

    def test_invalid_provider(self):
        process = subprocess.run(
            ["cloneholio", "-p", "microsofthub", "-t", "123"], stderr=subprocess.PIPE
        )
        self.assertEqual(process.returncode, 2)
        self.assertEqual(
            process.stderr.decode().split("\n")[-2],
            "cloneholio: error: argument -p/--provider: invalid choice: 'microsofthub' (choose from 'github', 'gitlab')",  # noqa: E501
        )

    def test_missing_token(self):
        process = subprocess.run(
            ["cloneholio", "-p", "github", "apath"], stderr=subprocess.PIPE
        )
        self.assertEqual(process.returncode, 2)
        self.assertEqual(
            process.stderr.decode().split("\n")[-2],
            "cloneholio: error: the following arguments are required: -t/--token",  # noqa: E501
        )
