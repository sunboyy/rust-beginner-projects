# URL Shortener in Rust

This tutorial will guide you through building a simple URL shortening service in Rust, using popular HTTP libraries like `axum`, and integrating with an SQLite or PostgreSQL database.

## What you will learn

- Building a web server using [axum](https://github.com/tokio-rs/axum)
- Asynchronous programming with [tokio](https://tokio.rs/)
- Handling HTTP requests and responses
- Use [SQLx](https://github.com/launchbadge/sqlx) for SQLite database integration
- Error handling

## Overview

This project demonstrates how to build a URL shortener from scratch using Rust. The application will allow users to input long URLs, generate short unique codes, and handle redirection from the short URL to the original one.

## Features

- Shorten long URLs into unique, short URLs.
- Redirect users from the short URL to the original one.
- Store URL mappings in a database.
- REST API to interact with the URL shortener.
- Error handling for invalid URLs and non-existent short codes.

## Walkthrough

1. URL Shortening:
    - Expose an endpoint `POST /shorten` that accepts a long URL in the request body as JSON.
    - Generate a unique short code for the URL.
    - Store the mapping between the short code and the original URL in the database.
    - Return the short URL and the short code as JSON.
2. Redirection:
    - Expose an endpoint `GET /:short_code` that accepts a short code as a path parameter.
    - Retrieve the original URL from the database using the short code.
    - Return a `302 Found` response with the redirect URL if the short code exists.
    - Return a `404 Not Found` response and an error message `Short URL not found` as a string if the short code doesn't exist.
3. Lookup:
    - Expose an endpoint `GET /lookup` that accepts a short code as a query parameter.
    - Retrieve the original URL from the database using the short code.
    - Return the original URL as a string if the short code exists.
    - Return a `404 Not Found` response and an error message `Short URL not found` as a string if the short code doesn't exist.
4. Error Handling:
    - Gracefully handles invalid URLs and short codes that don't exist, returning appropriate error messages.
