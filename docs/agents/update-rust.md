# Runbook: Update the Rust toolchain and dependencies

This document tells an agent (or a person) how to update this repository's Rust toolchain and Rust dependencies to their latest versions, fix anything that breaks, and land the change through a pull request.

## Goal

Update the Rust toolchain and all Rust dependencies to their latest versions. Fix any code broken by API or lint changes. Make sure formatting, linting and tests pass. Open a pull request, and make sure CI passes on it. If no code changes were needed, merge the PR automatically once CI passes and delete its branch.

If this runbook was started from an issue, the issue identifier is `<issue-key>`. Use it in the branch name and comment on the issue as described below. If there is no issue, skip every issue-related instruction.

## Repo-specific facts (keep this section up to date)

Fill these in so future runs don't have to rediscover them.

- Toolchain pinned in: <!-- e.g. rust-toolchain.toml, .github/workflows/ci.yml, Dockerfile -->
- Toolchain style: <!-- pinned stable / dated nightly / unpinned channel -->
- MSRV (`rust-version`) policy: <!-- e.g. tracks the pinned toolchain / independent -->
- Workspace members: <!-- or "single crate" -->
- Default branch: <!-- e.g. main -->

## Hard rules

- **Never touch `Cargo.lock` directly.** Do not open, read, search, diff or edit it, whether with `cat`, `grep`, `git diff`, `git show` or any other tool. Only edit `Cargo.toml` files and let Cargo regenerate the lockfile. Commit whatever `Cargo.lock` Cargo produces. When reviewing changes, exclude it: `git diff -- . ':!Cargo.lock'`.
- **Formatting uses `cargo xfmt`**, not plain `cargo fmt`.
- **Clippy must pass with no warnings.** Fix the code rather than adding `#[allow(...)]`, unless a fix would be a large redesign. In that case, allow the lint as narrowly as possible and explain why in the PR.
- **Do not change the crate `edition`.**
- **Do not bypass branch protection or review requirements.**

## Definitions

- **No code changes** means the PR changes no `.rs` files, including formatting-only changes and added `#[allow(...)]` attributes. Changes limited to `Cargo.toml`, `Cargo.lock`, toolchain files, CI workflow files and Dockerfiles do not count as code changes.
- **Nothing to update** means neither the toolchain nor any dependency needs updating.

## Steps

### 1. Update the Rust toolchain

1. Find every place a Rust version is pinned. The toolchain may appear in `rust-toolchain.toml` or `rust-toolchain`, `rust-version` in `Cargo.toml` (`[package]` or `[workspace.package]`), CI workflow files (`dtolnay/rust-toolchain@<version>` or `toolchain:` inputs in `.github/workflows/*.yml`), Dockerfiles, and any scripts that name a Rust version. A starting point:
   `grep -rnE 'rust-version|toolchain|channel|rust:[0-9]' --include='*.toml' --include='*.yml' --include='*.yaml' --include='Dockerfile*' .`
2. Find the latest stable release: `rustup update stable && rustc +stable --version`.
3. Apply the rule that matches the project:
   - **Pinned stable version** (e.g. `channel = "1.80.0"`): bump it to the latest stable in every place it appears, keeping them consistent. Keep `components`, `targets` and `profile` in `rust-toolchain.toml` as they are.
   - **Pinned dated nightly:** bump to a recent nightly that has every component the project needs (check https://rust-lang.github.io/rustup-components-history/).
   - **Unpinned channel** (e.g. `channel = "stable"`): nothing to bump; just make sure the local toolchain is current.
4. Treat `rust-version` as the project's MSRV. If it was the same as the pinned toolchain, keep them in sync. Otherwise, only raise it if an upgraded dependency requires a newer compiler, and note this in the PR.
5. Run `rustup show` in the repo to confirm the new toolchain is installed and active.

### 2. Check for outdated dependencies

- Run `cargo update --dry-run` to see compatible (semver) updates.
- Find newer major/breaking versions with `cargo outdated` or `cargo upgrade --dry-run` (install `cargo-outdated` or `cargo-edit` if missing), or by checking crates.io for each dependency in `Cargo.toml`.

If the toolchain is current **and** no dependency has an update, the work is "nothing to update". Do not open a PR. Go to step 9 and report that nothing needed updating.

### 3. Update dependencies

- Bump versions in every `Cargo.toml` (every workspace member) to the latest releases, including breaking versions.
- Run `cargo update` so Cargo regenerates `Cargo.lock`, and **save its output**. Cargo prints each change as `Updating <crate> vX -> vY`. Use that output, not the lockfile, as your record of what changed.

### 4. Fix breaking changes

- Run `cargo build --all-targets` and fix every compile error caused by changed APIs or the new compiler.
- For major-version bumps, read the crate's changelog or release notes and move to the recommended new APIs rather than deprecated ones.
- If a dependency can't be upgraded without a large redesign, leave it at its current version and explain why in the PR.

### 5. Verify locally

1. Run `cargo xfmt` and commit any formatting changes (the new rustfmt may format some code differently).
2. Run `cargo clippy --all-targets` and fix every warning, including new lints from the toolchain upgrade.
3. Run `cargo test` and make sure all tests pass.

### 6. Raise a pull request

- Create a branch. If there is an issue, include its identifier (e.g. `<issue-key>-dependency-updates`); otherwise use something like `rust-dependency-updates`.
- Commit with clear messages. Keep the toolchain bump (and fixes for new lints) in a **separate commit** from the dependency updates.
- Push, and open a PR against the default branch with `gh pr create`.
- Title the PR exactly `Update Rust toolchain and dependencies`. Do not put the issue identifier in the title.
- The PR description must include:
  - The toolchain's old → new version, every file where it was changed, and whether `rust-version` (MSRV) changed and why.
  - Each dependency with its old → new version, taken from your `Cargo.toml` changes and the saved `cargo update` output (never from `Cargo.lock`).
  - A summary of code changes made for new APIs and new lints, or "No code changes were required."
  - Any dependencies left un-upgraded and why.

### 7. Wait for CI and fix failures

- Run `gh pr checks --watch` to wait for all checks to finish. If the PR has no CI checks, skip this step.
- If a check fails: identify the failed run with `gh pr checks`, read logs with `gh run view <run-id> --log-failed`, fix the cause, repeat step 5, commit, push, and watch again.
  - CI may use a toolchain version missed in step 1. If so, update it there too.
  - If a check fails because `Cargo.lock` is out of date (e.g. a `--locked` or `--frozen` build), run `cargo update` or `cargo build` locally and commit the result. Do not edit the lockfile by hand.
- Repeat up to 3 times. If CI still fails, stop. Report which checks are failing and what you tried (as a comment on the issue if there is one, otherwise in your final summary). **Do not merge.**

### 8. Merge only if no code changes were required

Once all CI checks pass (or the PR has none), list the PR's files with `gh pr diff --name-only`.

- **No `.rs` file in the list:** merge with `gh pr merge --merge --delete-branch`, then confirm cleanup:
  - Check the remote: `git ls-remote --heads origin <branch>`. If the branch still exists, run `git push origin --delete <branch>`.
  - Switch back to the default branch and delete the local branch with `git branch -D <branch>` if it still exists.
  - Report the merged PR link (comment on the issue if there is one).
- **Any `.rs` file changed:** do not merge. Leave the PR open for review and report the PR link with a one-line summary of the code changes that need review.
- **Merge blocked** (e.g. branch protection requires an approving review): do not bypass it. Leave the PR open, do not delete the branch, and explain why it couldn't be merged.

### 9. Close out (only if started from an issue)

Move the issue to Done (or your tracker's equivalent) only when the work is complete, which means either:

- the PR was merged in step 8, or
- nothing needed updating (comment on the issue saying so).

Do **not** move the issue to Done if the PR was left open for review, CI is still failing after 3 attempts, or the merge was blocked. Leave its status unchanged so a person can follow up.