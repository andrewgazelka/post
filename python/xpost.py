"""Post to X (twitter) by driving a logged-in browser over the Chrome DevTools Protocol.

No API keys: it attaches to a browser you are already signed into and uses the real
composer, the same way you would by hand. This is the browser-control counterpart to
the Rust `post x` CLI (which uses the X API + OAuth).

    import asyncio, xpost
    asyncio.run(xpost.post("hello world"))           # compose + click Post
    asyncio.run(xpost.post("draft", confirm=False))  # stage only, do not send

Start your browser with remote debugging first, e.g. Chrome:

    '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' \\
        --remote-debugging-port=9222 --user-data-dir=/tmp/x-profile

then sign into x.com in that window once.

Note on Dia/Arc-engine browsers: tabs created over CDP do not appear in their tab UI,
so this module never creates a tab; it reuses (or navigates) an existing one.
"""

from __future__ import annotations

import json as _json
import sys

X_HOME = "https://x.com/home"
DEFAULT_ENDPOINT = "http://127.0.0.1:9222"
_COMPOSER = '[data-testid="tweetTextarea_0"]'
_POST_BUTTON = '[data-testid="tweetButtonInline"], [data-testid="tweetButton"]'


def select_all_chord(platform: str = sys.platform) -> str:
    """The Playwright key chord that selects all text in a field, per OS.

    macOS uses Cmd (``Meta``); everywhere else uses ``Control``. Getting this wrong
    is why a naive clear leaves the previous draft in place and the new text is
    appended to it.
    """
    return "Meta+a" if platform == "darwin" else "Control+a"


def is_reply_style(text: str) -> bool:
    """X treats a post that starts with ``@handle`` as a reply-style mention: it is
    filed under "Posts & replies" rather than the main timeline."""
    return text.lstrip().startswith("@")


async def _connect(endpoint: str):
    from playwright.async_api import async_playwright

    pw = await async_playwright().start()
    browser = await pw.chromium.connect_over_cdp(endpoint)
    return pw, browser


async def _x_page(browser):
    """Reuse a logged-in x.com tab if present, else navigate an existing tab to x.com.

    Never opens a new tab: Arc-engine browsers (Dia) do not render CDP-created tabs.
    """
    ctx = browser.contexts[0] if browser.contexts else await browser.new_context()
    page = next((p for p in ctx.pages if "x.com" in p.url or "twitter.com" in p.url), None)
    if page is None:
        page = ctx.pages[-1] if ctx.pages else await ctx.new_page()
        await page.goto(X_HOME, wait_until="domcontentloaded")
    await page.bring_to_front()
    return page


async def _composer_text(page) -> str:
    sel = _json.dumps(_COMPOSER)
    text = await page.evaluate(
        f"() => {{ const el = document.querySelector({sel}); return el ? el.innerText : null; }}"
    )
    # contenteditable reports a trailing newline; drop it so comparisons are exact.
    return text[:-1] if text and text.endswith("\n") else text


async def _clear(page) -> None:
    """Fully empty the composer (OS-correct select-all, looped until verified empty)."""
    chord = select_all_chord()
    for _ in range(6):
        await page.keyboard.press(chord)
        await page.keyboard.press("Backspace")
        await page.wait_for_timeout(120)
        if not await _composer_text(page):
            return
    remaining = await _composer_text(page)
    if remaining:
        raise RuntimeError(f"could not clear composer; still has: {remaining!r}")


async def compose(text: str, *, endpoint: str = DEFAULT_ENDPOINT, page=None):
    """Open the composer, clear it, type ``text``, and verify the field matches. No send."""
    own = page is None
    if own:
        _pw, browser = await _connect(endpoint)
        page = await _x_page(browser)
    if "/home" not in page.url and "/compose" not in page.url:
        await page.goto(X_HOME, wait_until="domcontentloaded")
        await page.wait_for_timeout(800)
    box = page.locator(_COMPOSER)
    await box.click()
    await _clear(page)
    await page.keyboard.type(text, delay=12)
    await page.wait_for_timeout(400)
    got = await _composer_text(page)
    if got != text:
        raise RuntimeError(f"composer mismatch.\n wanted: {text!r}\n got:    {got!r}")
    return page


async def post(text: str, *, endpoint: str = DEFAULT_ENDPOINT, confirm: bool = True) -> dict:
    """Compose ``text`` and click Post. ``confirm=False`` stages it without sending.

    Returns a small dict describing what happened. Raises if the field cannot be
    cleared/typed or if X disables the Post button (empty or over the length limit).
    """
    page = await compose(text, endpoint=endpoint)
    if not confirm:
        return {"sent": False, "staged": text, "reply_style": is_reply_style(text)}
    button = page.locator(_POST_BUTTON).first
    if await button.is_disabled():
        raise RuntimeError("X disabled the Post button (empty or over length); not sending")
    await button.click()
    await page.wait_for_timeout(2500)
    after = await _composer_text(page)
    return {
        "sent": not after,
        "text": text,
        "reply_style": is_reply_style(text),
        "composer_after": after,
    }


def _main(argv: list[str]) -> int:
    import argparse
    import asyncio

    parser = argparse.ArgumentParser(prog="xpost", description="Post to X via a logged-in browser (CDP).")
    parser.add_argument("text", help="the message to post")
    parser.add_argument("--endpoint", default=DEFAULT_ENDPOINT, help="CDP endpoint (default %(default)s)")
    parser.add_argument("--dry-run", action="store_true", help="stage in the composer without sending")
    args = parser.parse_args(argv)
    result = asyncio.run(post(args.text, endpoint=args.endpoint, confirm=not args.dry_run))
    print(result)
    return 0


if __name__ == "__main__":
    raise SystemExit(_main(sys.argv[1:]))
