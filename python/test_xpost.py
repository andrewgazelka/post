"""Pure-logic tests for xpost (stdlib unittest, no browser, no third-party deps).

The browser-driving paths are validated live (see README.md), but the bits that
caused real bugs -- the OS-specific select-all chord and reply-style detection --
are unit-tested here so they cannot regress.

Run: ``python -m unittest`` (or ``python -m pytest`` if available).
"""

import inspect
import unittest

import xpost


class SelectAllChordTests(unittest.TestCase):
    def test_cmd_on_macos(self):
        self.assertEqual(xpost.select_all_chord("darwin"), "Meta+a")

    def test_ctrl_elsewhere(self):
        self.assertEqual(xpost.select_all_chord("linux"), "Control+a")
        self.assertEqual(xpost.select_all_chord("win32"), "Control+a")


class ReplyStyleTests(unittest.TestCase):
    def test_detects_leading_at(self):
        self.assertTrue(xpost.is_reply_style("@diabrowser hello"))
        self.assertTrue(xpost.is_reply_style("   @someone with leading space"))

    def test_false_for_normal_post(self):
        self.assertFalse(xpost.is_reply_style("hello @diabrowser in the middle"))
        self.assertFalse(xpost.is_reply_style("just a normal post"))


class ApiShapeTests(unittest.TestCase):
    def test_post_and_compose_are_async(self):
        self.assertTrue(inspect.iscoroutinefunction(xpost.post))
        self.assertTrue(inspect.iscoroutinefunction(xpost.compose))


if __name__ == "__main__":
    unittest.main()
