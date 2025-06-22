format arg="--check":
  #!/usr/bin/env bash
  just fmt {{arg}}
  just lint {{arg}}

fmt arg="--check":
  #!/usr/bin/env bash
  args=""
  while read -r line; do
    line=$(echo "$line" | tr -d "[:space:]")
    args="$args --config $line"
  done < "rust-fmt.toml"
  args=$(echo "$args" | xargs)
  if [ "{{arg}}" = "--fix" ]; then
    cargo fmt -- $args
  else
    cargo fmt --check -- $args
  fi

lint arg="--check":
  #!/usr/bin/env bash
  if [ "{{arg}}" = "--fix" ]; then
    cargo clippy --fix --allow-dirty -- --deny "warnings"
  else
    cargo clippy -- --deny "warnings"
  fi