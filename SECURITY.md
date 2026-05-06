# Security Policy

## Reporting a Vulnerability

Please do not open public GitHub issues for suspected vulnerabilities.

Email the maintainer at `security@haneulchi.app` with:

- affected version or commit
- reproduction steps
- impact assessment
- any relevant logs, screenshots, or proof-of-concept details

If that address is not yet configured for your deployment, contact the repository owner privately through GitHub first and avoid posting exploit details in public threads.

## Scope

Security-sensitive areas include:

- secret persistence and redaction
- policy approvals and dangerous terminal input gates
- workflow hook execution and workspace path safety
- project file read/write boundaries
- local control API access
- release signing, notarization, update feed, and crash-symbol upload workflows

## Supported Versions

Haneulchi is currently pre-1.0. Security fixes target the latest commit on `main` until versioned release branches are introduced.

## Maintainer Expectations

- Keep credentials, private docs, local state, and build artifacts out of git.
- Prefer local-first defaults and explicit user approval for risky actions.
- Add regression tests for security fixes whenever practical.
