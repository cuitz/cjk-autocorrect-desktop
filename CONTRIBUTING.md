# Contributing to CJK AutoCorrect Desktop

Thank you for your interest in CJK AutoCorrect Desktop. Contributions are welcome, including bug reports, feature proposals, documentation improvements, tests, and code changes.

## Project Fit

CJK AutoCorrect Desktop is a Tauri 2 desktop app with a React 19, TypeScript, Vite, Tailwind CSS 4, and Zustand frontend. The backend is written in Rust and uses the bundled `autocorrect` engine for formatting. Contributions should fit this architecture and avoid adding external runtime requirements unless there is a clear project need.

Good contributions usually improve one of these areas:

- CJK text formatting behavior and rule configuration
- Clipboard and global shortcut workflows
- Settings, history, accessibility, and internationalization
- Tauri desktop integration, such as tray behavior, autostart, and packaging
- Tests, CI reliability, documentation, and release quality

## Reporting Issues

Use [GitHub Issues](../../issues) for bug reports and feature requests.

For bugs, please include:

- Your operating system and app version
- Steps to reproduce the issue
- Expected behavior and actual behavior
- Example input and output text, when the issue is formatting-related
- Screenshots or logs, when they help explain the problem

For feature requests, please describe the workflow you want to improve and why it matters.

## Development Setup

Requirements:

- Node.js 22, matching the GitHub Actions configuration
- pnpm 10, matching the repository package manager setting
- Stable Rust
- Tauri system dependencies for your operating system

Install dependencies:

```bash
pnpm install
```

Start the development app:

```bash
pnpm tauri dev
```

Build the app locally:

```bash
pnpm tauri build
```

Build artifacts are generated under `src-tauri/target/release/bundle/`.

## Validation

Before opening a pull request, run the checks that match CI as closely as possible:

```bash
pnpm build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --all-targets --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

If `cargo fmt --check` fails, run:

```bash
cd src-tauri
cargo fmt
```

Then rerun the validation commands.

## Pull Requests

1. Fork the repository.
2. Create a focused branch, for example `feature/rule-examples` or `fix/tray-right-click`.
3. Make the smallest coherent change that solves the problem.
4. Add or update tests when changing formatting behavior, shared Rust services, or user-visible workflows.
5. Run the validation commands above.
6. Open a pull request against `main`.

Please keep pull requests focused. Separate unrelated refactors, UI redesigns, and behavior changes into different PRs.

## Code Style

- Follow the existing Rust module structure under `src-tauri/src`.
- Use `cargo fmt` for Rust formatting.
- Keep Rust warnings clean under `cargo clippy --all-targets -- -D warnings`.
- Follow the existing React component style: functional components, hooks, and local store patterns.
- Keep UI copy localized through `src/i18n.ts` when text is user-facing.
- Keep formatting rule examples clear enough that users can understand the rule at a glance.
- Avoid introducing new dependencies when the existing stack can reasonably solve the problem.

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/) where possible:

```text
feat: add formatter rule examples
fix: only open tray window on left click
chore: update release workflow
docs: clarify local build steps
```

## Releases

Releases are created from version tags that match `v*`, such as `v0.1.2`. The release workflow builds macOS and Windows assets through GitHub Actions.

When preparing a release, keep these versions aligned:

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`

## License

By contributing, you agree that your contribution will be licensed under the [MIT License](./LICENSE).
