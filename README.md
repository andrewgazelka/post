<p align="center">
  <img src=".github/assets/header.svg" alt="xpost" width="100%"/>
</p>

<p align="center">
  <code>nix run github:andrewgazelka/xpost</code>
</p>

A minimal CLI for posting to X (Twitter) using OAuth 2.0 PKCE.

## Features

- **OAuth 2.0 PKCE**: Secure authentication flow with automatic token refresh
- **Simple**: Just `xpost "Hello world"` to post
- **Fast**: Written in Rust, single binary

## Setup

1. Create an app at [developer.x.com](https://developer.x.com)
2. Enable OAuth 2.0 with callback URL `http://localhost:8080/callback`
3. Authenticate:

```bash
xpost auth --client-id YOUR_CLIENT_ID --client-secret YOUR_CLIENT_SECRET
```

Or set environment variables `CLIENT_ID` and `CLIENT_SECRET`.

## Usage

```bash
# Post a tweet
xpost "Hello from xpost!"

# Or use the post subcommand
xpost post "Hello from xpost!"

# Check auth status
xpost status

# Logout
xpost logout
```

## Install

```bash
# With Nix
nix run github:andrewgazelka/xpost

# Or install
nix profile install github:andrewgazelka/xpost

# With Cargo
cargo install --git https://github.com/andrewgazelka/xpost
```

---

<sub>Not affiliated with X Corp. This is an unofficial tool.</sub>
