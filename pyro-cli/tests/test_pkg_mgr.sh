#!/bin/bash
set -e

# Setup
PROJECT_ROOT=$(pwd)
TEST_DIR=$(mktemp -d)
PYRO_BIN="$PROJECT_ROOT/../target/debug/pyro-cli"
if [ ! -f "$PYRO_BIN" ]; then
    PYRO_BIN="$PROJECT_ROOT/target/debug/pyro-cli"
fi

echo "Using test dir: $TEST_DIR"
cd "$TEST_DIR"

# 1. Create a dummy package repo
mkdir -p dummy-pkg
cd dummy-pkg
git init
echo "def hello(): print('Hello from dummy package')" > main.pyro
git add .
git commit -m "Initial commit"
DUMMY_PKG_PATH=$(pwd)
DUMMY_PKG_HASH=$(git rev-parse HEAD)
cd ..

# 2. Create a consumer project
mkdir consumer
cd consumer
"$PYRO_BIN" mod init consumer

# 3. Add dependency (using file:// URL to point to local repo)
# We use a weird package name because we are using the URL as the name
PKG_URL="file://$DUMMY_PKG_PATH"
# Strip file:// for cleaner logs if preferred, but our logic uses the whole string as name if we pass it.
# Actually, the 'name' in manifest will be 'file:///...'. This is fine for testing.

"$PYRO_BIN" get "$PKG_URL"

# 4. Verify pyro.mod
if grep -q "$PKG_URL" pyro.mod; then
    echo "pyro.mod contains dependency"
else
    echo "pyro.mod MISSING dependency"
    exit 1
fi

# 5. Verify pyro.lock
if grep -q "$DUMMY_PKG_HASH" pyro.lock; then
    echo "pyro.lock contains correct commit hash"
else
    echo "pyro.lock MISSING commit hash. Found:"
    cat pyro.lock
    exit 1
fi

if grep -q "checksum" pyro.lock; then
    echo "pyro.lock contains checksum"
else
    echo "pyro.lock MISSING checksum"
    exit 1
fi

echo "Verification Passed!"
# Cleanup
rm -rf "$TEST_DIR"
