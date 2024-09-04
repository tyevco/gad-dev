#!/bin/bash

# Build the project
cargo build --release

# Check if the build was successful
if [ $? -ne 0 ]; then
    echo "Build failed"
    exit 1
fi

# Determine the correct file extension based on the OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    EXT=".dylib"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    EXT=".dll"
else
    EXT=".so"
fi

SRC_PATH="./target/release/libgodot_asset_browser$EXT"
DEST_PATH="../godot/addons/godot_asset_browser/libgodot_asset_browser$EXT"
GDEXTENSION_PATH="../godot/addons/godot_asset_browser/godot_asset_browser.gdextension"

# Create destination directory if it doesn't exist
mkdir -p "$(dirname "$DEST_PATH")"

# Copy the file
cp "$SRC_PATH" "$DEST_PATH"

if [ $? -eq 0 ]; then
    echo "Successfully copied $SRC_PATH to $DEST_PATH"
else
    echo "Failed to copy $SRC_PATH to $DEST_PATH"
    exit 1
fi

# Create .gdextension file if it doesn't exist
if [ ! -f "$GDEXTENSION_PATH" ]; then
    cat > "$GDEXTENSION_PATH" << EOL
[configuration]

entry_symbol = "gdext_rust_init"
compatibility_minimum = 4.1

[libraries]

linux.debug.x86_64 = "res://addons/godot_asset_browser/libgodot_asset_browser.so"
linux.release.x86_64 = "res://addons/godot_asset_browser/libgodot_asset_browser.so"
windows.debug.x86_64 = "res://addons/godot_asset_browser/godot_asset_browser.dll"
windows.release.x86_64 = "res://addons/godot_asset_browser/godot_asset_browser.dll"
macos.debug = "res://addons/godot_asset_browser/libgodot_asset_browser.dylib"
macos.release = "res://addons/godot_asset_browser/libgodot_asset_browser.dylib"
EOL
    echo "Created $GDEXTENSION_PATH"
else
    echo "$GDEXTENSION_PATH already exists"
fi

# Path to Godot executable (adjust this to your Godot installation path)
GODOT_PATH="godot"

# Path to your Godot project
PROJECT_PATH="../godot"

# Launch Godot and open the project
echo "Launching Godot editor with the project..."
"$GODOT_PATH" --path "$PROJECT_PATH" --editor

echo "Godot editor closed. Cleaning up..."

# Clean up the addons folder
rm -rf "../godot/addons/godot_asset_browser"
echo "Cleaned up addons folder"