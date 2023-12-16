
# Get the directory of the script
SCRIPT_DIR=$(dirname "$0")

# Change to the script's directory
cd "$SCRIPT_DIR"

echo "Running cargo watch in $SCRIPT_DIR"

# -dev added for dev build
cargo watch -i .gitignore -i "pkg/*" -s "wasm-pack build --dev --target web --out-dir pkg poker_eval"