## Response style

- Lead with the outcome. No preamble, no restating the task, no conversational filler.
- Use short sections or bullets only when they aid parsing.
- Keep the structure sleek and functional.

## Comments

Do not leave comments unless absolutely necessary. The default is no comment; code carries the meaning. A comment earns its place only when it states something the code cannot — a non-obvious reason, a constraint, or a platform quirk — and nothing else will do.

- Never comment the obvious: no narration of the next line, no restating a name, type or signature, no section banners or `//` separators.
- When in doubt, omit the comment. A missing comment is the normal case; an added one must be defensible.
- Comment intent or a reason, never a description of what the code does — if a comment merely restates the code, delete the comment, not the code.
- Do not add doc comments to code that already reads clearly from its type and name.
- Add a comment only when omitting it would plausibly cause a wrong "fix" later.
- Leave existing comments alone. Removing or rewriting them is a separate, explicit request.

## Commands

| Command | Purpose |
|---|---|
| `cargo check --quiet --workspace` | Validation gate for every change. |
| `cargo build --quiet` | Validation gate for every change. |
| `cargo xfmt` | Format after `cargo check` passes. |
| `cargo clippy --quiet --all-targets --workspace -- -D warnings` | Checks lint rules after `cargo check` passes |
| `cargo clippy --quiet --fix --all-targets --workspace --allow-dirty` | Automatically fix clippy warnings if check fails. |
| `cargo run` | Launch the application locally. Examples are under `./examples` |
