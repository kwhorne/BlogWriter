# Security Policy

## Reporting a vulnerability

Please **do not** open a public issue for security vulnerabilities.

Instead, report them privately via
[GitHub Security Advisories](https://github.com/kwhorne/BlogWriter/security/advisories/new).

You can expect an initial response within a week.

## Scope notes

- BlogWriter stores API keys and site bearer tokens in a **local SQLite
  database** (`blogwriter.db`). This file must never be committed or shared.
- Articles are published over HTTPS to endpoints you configure; use strong,
  unique bearer tokens per site and rotate them if leaked.
