# xpost — post to X from Python via the browser

A tiny Python module that posts to X (twitter) by driving a browser you are already
signed into, over the Chrome DevTools Protocol (CDP). No API keys, no OAuth: it uses
the real composer, exactly as you would by hand.

This is the browser-control sibling of the Rust `post x` CLI in this repo (which uses
the X API + OAuth 2.0). Use whichever fits: the CLI for headless/server posting with a
developer app, this module when you just want to drive a logged-in window.

## Use

```python
import asyncio, xpost

asyncio.run(xpost.post("hello world"))            # compose + click Post
asyncio.run(xpost.post("draft", confirm=False))   # stage only, do not send
```

Or from the shell:

```bash
python -m xpost "hello world"
python -m xpost "draft" --dry-run        # stage without sending
python -m xpost "hi" --endpoint http://127.0.0.1:9333
```

## Setup

1. Launch a browser with remote debugging on (Chrome shown; any Chromium works):

   ```bash
   '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' \
       --remote-debugging-port=9222 --user-data-dir=/tmp/x-profile
   ```

2. Sign into x.com in that window once.
3. Run the examples above.

## Notes

- **Reply-style posts:** a message that starts with `@handle` is treated by X as a
  reply-style mention and is filed under "Posts & replies", not the main timeline.
  `xpost.is_reply_style(text)` reflects this.
- **Clearing the composer** uses the OS-correct select-all chord (Cmd+A on macOS,
  Ctrl+A elsewhere) and verifies the field is empty before typing, so a previous
  draft can never bleed into a new post.
- **Dia / Arc-engine browsers:** tabs created over CDP do not appear in their tab UI,
  so this module never creates a tab; it reuses or navigates an existing one. Plain
  Chrome shows everything normally.

## Tests

```bash
cd python && python -m pytest
```

The unit tests cover the pure logic (select-all chord, reply detection). The
browser-driving path is verified live against a real logged-in session.
