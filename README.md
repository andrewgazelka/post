<p align="center">
  <img src=".github/assets/header.svg" alt="post" width="100%"/>
</p>

<p align="center">
  <code>nix run github:andrewgazelka/post</code>
</p>

A minimal CLI for posting to social media.

## Features

- **OAuth 2.0 PKCE**: Secure authentication flow with automatic token refresh
- **Simple**: Just `post "Hello world"` to post
- **Fast**: Written in Rust, single binary

## Supported Platforms

- [x] X (Twitter)
- [ ] Reddit ([#2](https://github.com/andrewgazelka/post/issues/2))

## Setup (X/Twitter)

1. Create an app at [developer.x.com](https://developer.x.com)
2. Enable OAuth 2.0 with callback URL `http://localhost:8080/callback`
3. Authenticate:

```bash
post auth --client-id YOUR_CLIENT_ID --client-secret YOUR_CLIENT_SECRET
```

Or set environment variables `CLIENT_ID` and `CLIENT_SECRET`.

## Usage

```bash
# Post a tweet
post "Hello from post!"

# Or use the post subcommand
post post "Hello from post!"

# Check auth status
post status

# Logout
post logout
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
