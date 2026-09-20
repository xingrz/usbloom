#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
profile="${1:-debug}"
if [ "$profile" = release ]; then
  cargo build --locked --release
else
  cargo build --locked
fi
app="$PWD/dist/USBloom.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "target/$profile/usbloom" "$app/Contents/MacOS/USBloom"
cp LICENSE "$app/Contents/Resources/LICENSE"
cp assets/USBloom.icns "$app/Contents/Resources/USBloom.icns"
cat > "$app/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleExecutable</key><string>USBloom</string>
  <key>CFBundleIdentifier</key><string>me.xingrz.usbloom</string>
  <key>CFBundleIconFile</key><string>USBloom.icns</string>
  <key>CFBundleName</key><string>USBloom</string>
  <key>CFBundleDisplayName</key><string>USBloom</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>0.1.0</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>LSMinimumSystemVersion</key><string>12.0</string>
</dict></plist>
PLIST
codesign --force --deep --sign - --identifier me.xingrz.usbloom "$app"
printf '%s\n' "$app"
