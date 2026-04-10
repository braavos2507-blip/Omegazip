#!/bin/bash
# Optional: injects NSServices into OmegaZip.app (Finder shows «… (OmegaZip)»).
# If you also ran ./scripts/install-context-menu.sh, Services will list BOTH
# workflow entries («…workflow») and these — four lines total. Prefer workflows
# only: use build:macos without this script, or uninstall ~/Library/Services/*OmegaZip*.
# App handles files via openFiles (see macos_open_files.rs).
# Run after: npm run tauri build
set -e
APP="${1:-src-tauri/target/release/bundle/macos/OmegaZip.app}"
PLIST="$APP/Contents/Info.plist"
if [[ ! -f "$PLIST" ]]; then
  echo "App not found: $APP" >&2
  exit 1
fi
# Merge our Info.plist NSServices into the built app (Tauri may already do this; this ensures it)
/usr/libexec/PlistBuddy -c "Delete :NSServices" "$PLIST" 2>/dev/null || true
/usr/libexec/PlistBuddy -c "Add :NSServices array" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:0 dict" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:0:NSMenuItem dict" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:0:NSMenuItem:default string 'Сжать в OmegaZip'" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:0:NSMessage string 'openFiles'" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:0:NSPortName string 'OmegaZip'" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:0:NSSendTypes array" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:0:NSSendTypes:0 string 'public.file-url'" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:0:NSSendTypes:1 string 'NSFilenamesPboardType'" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:0:NSSendTypes:2 string 'public.item'" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:0:NSRequiredContext dict" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:0:NSRequiredContext:NSApplicationIdentifier string 'com.apple.finder'" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:1 dict" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:1:NSMenuItem dict" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:1:NSMenuItem:default string 'Распаковать в OmegaZip'" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:1:NSMessage string 'openFiles'" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:1:NSPortName string 'OmegaZip'" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:1:NSSendTypes array" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:1:NSSendTypes:0 string 'public.file-url'" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:1:NSSendTypes:1 string 'NSFilenamesPboardType'" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:1:NSSendTypes:2 string 'public.item'" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:1:NSRequiredContext dict" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSServices:1:NSRequiredContext:NSApplicationIdentifier string 'com.apple.finder'" "$PLIST"
echo "NSServices injected into $PLIST"
echo "Restart Finder (or log out/in) and check: right-click file → Services → Сжать в OmegaZip / Распаковать в OmegaZip"
