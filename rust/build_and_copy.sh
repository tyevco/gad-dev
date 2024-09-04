#!/bin/bash

# Default to debug build
BUILD_TYPE=

# Check for --release flag
if [[ "$1" == "--release" ]]; then
    BUILD_TYPE="release"
fi

echo "Building in ${BUILD_TYPE:-debug} mode"

# Build the project
cargo build ${BUILD_TYPE:+--$BUILD_TYPE}

# Check if the build was successful
if [ $? -ne 0 ]; then
    echo "Build failed"
    exit 1
fi

# Path to your Godot project
GODOT_PROJECT_PATH="../godot"
GDEXTENSION_PATH="$GODOT_PROJECT_PATH/godot_asset_browser.gdextension"

# Create .gdextension file
cat > "$GDEXTENSION_PATH" << EOL
[configuration]

entry_symbol = "gdext_rust_init"
compatibility_minimum = 4.1
reloadable = true

[libraries]

linux.debug.x86_64 =     "res://../rust/target/debug/libgodot_asset_browser.so"
linux.release.x86_64 =   "res://../rust/target/release/libgodot_asset_browser.so"
windows.debug.x86_64 =   "res://../rust/target/debug/godot_asset_browser.dll"
windows.release.x86_64 = "res://../rust/target/release/godot_asset_browser.dll"
macos.debug =            "res://../rust/target/debug/libgodot_asset_browser.dylib"
macos.release =          "res://../rust/target/release/libgodot_asset_browser.dylib"
macos.debug.arm64 =      "res://../rust/target/debug/libgodot_asset_browser.dylib"
macos.release.arm64 =    "res://../rust/target/release/libgodot_asset_browser.dylib"
EOL
echo "Created or updated $GDEXTENSION_PATH"

# Path to Godot executable (adjust this to your Godot installation path)
GODOT_PATH="godot"


# Launch Godot and open the project
echo "Launching Godot editor with the project..."
"$GODOT_PATH" --path "$GODOT_PROJECT_PATH" --editor

echo "Godot editor closed. Cleaning up..."
