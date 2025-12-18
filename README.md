<p align="center">
  <img src=".github/assets/header.svg" alt="post" width="100%"/>
</p>

<p align="center">
  <code>nix run github:andrewgazelka/post</code>
</p>

> [!WARNING]
> With the [Claude extension for Chrome](https://chrome.google.com/webstore/detail/claude/), Claude Code can now control your browser directly, which may make this tool less necessary for X/Twitter. However, Reddit is explicitly banned by the Chrome extension (likely for legal reasons), so this tool remains useful for Reddit posting.

A minimal CLI for posting to social media.

## Features

- **OAuth 2.0**: Secure authentication with automatic token refresh
- **Simple**: `post x post "Hello"` or `post reddit post -r rust -t "Title"`
- **Fast**: Written in Rust, single binary

## Supported Platforms

- [x] X (Twitter)
- [x] Reddit

## Setup

### X (Twitter)

1. Create an app at [developer.x.com](https://developer.x.com)
2. Enable OAuth 2.0 with callback URL `http://localhost:8080/callback`
3. Authenticate:

```bash
post x auth --client-id YOUR_CLIENT_ID --client-secret YOUR_CLIENT_SECRET
```

Or set environment variables `X_CLIENT_ID` and `X_CLIENT_SECRET`.

### Reddit

1. Create a "script" app at [reddit.com/prefs/apps](https://www.reddit.com/prefs/apps)
2. Authenticate:

```bash
post reddit auth \
  --client-id YOUR_CLIENT_ID \
  --client-secret YOUR_CLIENT_SECRET \
  --username YOUR_USERNAME \
  --password YOUR_PASSWORD
```

Or set environment variables `REDDIT_CLIENT_ID`, `REDDIT_CLIENT_SECRET`, `REDDIT_USERNAME`, `REDDIT_PASSWORD`.

## Usage

### X (Twitter)

```bash
# Post a tweet
post x post "Hello from post!"

# Check auth status
post x status

# Logout
post x logout
```

### Reddit

```bash
# Submit a text post
post reddit post -r rust -t "Check out my crate" -b "Body text here"

# Submit a link post
post reddit post -r rust -t "Check out my crate" -l "https://example.com"

# Check auth status
post reddit status

# Logout
post reddit logout
```

## Install

```bash
# With Nix
nix run github:andrewgazelka/post

# Or install
nix profile install github:andrewgazelka/post

# With Cargo
cargo install --git https://github.com/andrewgazelka/post
```
