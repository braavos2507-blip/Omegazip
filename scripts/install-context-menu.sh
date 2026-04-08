#!/usr/bin/env bash
# Ставит сервисы Finder (CLI): «Сжать в OmegaZip» и «Распаковать в OmegaZip».
# Формат при сжатии подбирается автоматически (.zip/.oz), без участия пользователя.
# Для .oz дополнительно создаётся self-extracting .command (macOS, без установки OmegaZip).
# Вызов: из корня проекта  ./scripts/install-context-menu.sh
# Требует: собранный OmegaZip.app в /Applications или в dist/

set -e
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
SERVICES="$HOME/Library/Services"
APP="/Applications/OmegaZip.app"
CLI="$APP/Contents/MacOS/omegazip"
[[ -d "$APP" ]] || APP="$ROOT/dist/OmegaZip.app"; CLI="$APP/Contents/MacOS/omegazip"
LOG_FILE="/tmp/OmegaZip-workflow.log"

# 1) Собрать CLI и положить в .app
echo "Сборка CLI..."
CARGO_TARGET_DIR="$ROOT/target" cargo build --release -p omegazip 2>/dev/null || true
if [[ -x "$ROOT/target/release/omegazip" ]]; then
  mkdir -p "$APP/Contents/MacOS"
  cp "$ROOT/target/release/omegazip" "$CLI"
  chmod +x "$CLI"
  echo "CLI установлен в $CLI"
else
  echo "Предупреждение: бинарь omegazip не найден; контекстное меню будет вызывать скрипт, проверьте путь к CLI в workflow."
fi

# 2) Скрипты (выполняются внутри workflow; OZ подставляется при установке)
STEM_FN=$(cat <<'ENDSTEM'
omegazip_stem() {
  local f seg
  f="$1"
  seg="${f##*/}"
  seg="${seg%/}"
  if [[ -d "$f" ]]; then
    printf '%s' "$seg"
    return 0
  fi
  if [[ "$seg" == .* ]]; then
    printf '%s' "$seg"
  elif [[ "$seg" == *.* ]]; then
    printf '%s' "${seg%.*}"
  else
    printf '%s' "$seg"
  fi
}
ENDSTEM
)
INPUT_FN=$(cat <<'ENDINPUT'
decode_input_path() {
  local raw="$1"
  local s
  s="$(printf '%s' "$raw" | tr -d '\r\n')"
  [[ -z "$s" ]] && return 1
  if [[ "$s" == file://* ]]; then
    s="${s#file://}"
    s="${s//%20/ }"
    s="${s//%5B/[}"
    s="${s//%5D/]}"
    s="${s//%28/(}"
    s="${s//%29/)}"
    s="${s//%23/#}"
    s="${s//%26/&}"
    s="${s//%2B/+}"
    s="${s//%2C/,}"
    s="${s//%3B/;}"
    s="${s//%40/@}"
    s="${s//%25/%}"
  fi
  printf '%s' "$s"
}

collect_inputs() {
  local -a in=()
  if [[ "$#" -gt 0 ]]; then
    in=("$@")
  else
    while IFS= read -r line; do
      [[ -n "$line" ]] && in+=("$line")
    done
  fi
  if [[ "${#in[@]}" -eq 0 ]]; then
    while IFS= read -r f; do
      [[ -n "$f" ]] && in+=("$f")
    done < <(osascript <<'OSA'
tell application "Finder"
  set sel to selection as alias list
  repeat with i in sel
    POSIX path of i
  end repeat
end tell
OSA
)
  fi
  printf '%s\n' "${in[@]}"
}
ENDINPUT
)
COMPRESS_AUTO_SCRIPT="OZ=$(printf '%q' "$CLI")"$'\n'"$STEM_FN"$'\n'"$INPUT_FN"$'\n'$(cat <<'ENDAUTO'
echo "[$(date '+%F %T')] [compress-auto] start args=$#" >> "/tmp/OmegaZip-workflow.log"

pick_ext_auto() {
  local f="$1"
  local base ext
  if [[ -d "$f" ]]; then
    printf '%s' "oz"
    return 0
  fi
  base="${f##*/}"
  ext="${base##*.}"
  ext="$(printf '%s' "$ext" | tr '[:upper:]' '[:lower:]')"
  if [[ "$ext" == "txt" || "$ext" == "md" || "$ext" == "markdown" || "$ext" == "csv" || "$ext" == "json" || "$ext" == "xml" || "$ext" == "yaml" || "$ext" == "yml" || "$ext" == "toml" || "$ext" == "ini" || "$ext" == "log" || "$ext" == "sql" || "$ext" == "rs" || "$ext" == "js" || "$ext" == "ts" || "$ext" == "tsx" || "$ext" == "jsx" || "$ext" == "html" || "$ext" == "css" || "$ext" == "java" || "$ext" == "kt" || "$ext" == "go" || "$ext" == "py" || "$ext" == "c" || "$ext" == "cpp" || "$ext" == "h" || "$ext" == "hpp" ]]; then
    printf '%s' "oz"
  else
    printf '%s' "zip"
  fi
}

make_sfx_for_oz() {
  local archive="$1"
  local sfx="${archive%.oz}.extract.command"
  {
    echo '#!/bin/bash'
    echo 'set -euo pipefail'
    echo 'SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"'
    echo 'WORK="$(mktemp -d)"'
    echo 'cleanup(){ rm -rf "$WORK"; }'
    echo 'trap cleanup EXIT'
    echo 'BIN="$WORK/omegazip"'
    echo 'ARC="$WORK/payload.oz"'
    echo 'OUT="${1:-$SCRIPT_DIR/$(basename "$0" .extract.command)_распаковано}"'
    echo '/usr/bin/base64 -D > "$BIN" <<'"'"'__OMEGAZIP_BIN__'"'"''
    /usr/bin/base64 -i "$OZ"
    echo '__OMEGAZIP_BIN__'
    echo '/usr/bin/base64 -D > "$ARC" <<'"'"'__OMEGAZIP_ARCHIVE__'"'"''
    /usr/bin/base64 -i "$archive"
    echo '__OMEGAZIP_ARCHIVE__'
    echo 'chmod +x "$BIN"'
    echo 'xattr -d com.apple.quarantine "$BIN" 2>/dev/null || true'
    echo '"$BIN" decompress "$ARC" "$OUT"'
    echo 'echo "Распаковано в: $OUT"'
  } > "$sfx"
  chmod +x "$sfx"
}

while IFS= read -r item; do
  f="$(decode_input_path "$item")"
  echo "[$(date '+%F %T')] [compress-auto] input=$item decoded=$f" >> "/tmp/OmegaZip-workflow.log"
  [[ -e "$f" ]] || continue
  stem="$(omegazip_stem "$f")"
  [[ -z "$stem" ]] && stem="archive"
  d="$(dirname "$f")"
  ext="$(pick_ext_auto "$f")"
  out="$d/$stem.$ext"
  echo "[$(date '+%F %T')] [compress-auto] chosen_ext=$ext out=$out" >> "/tmp/OmegaZip-workflow.log"
  if "$OZ" compress "$f" "$out" >> "/tmp/OmegaZip-workflow.log" 2>&1; then
    if [[ "$ext" == "oz" ]]; then
      make_sfx_for_oz "$out" >> "/tmp/OmegaZip-workflow.log" 2>&1 || true
    fi
    echo "Сжато: $out"
  fi
done < <(collect_inputs "$@")
ENDAUTO
)
EXTRACT_SCRIPT="OZ=$(printf '%q' "$CLI")"$'\n'"$STEM_FN"$'\n'"$INPUT_FN"$'\n'$(cat <<'ENDEX'
echo "[$(date '+%F %T')] [extract] start args=$#" >> "/tmp/OmegaZip-workflow.log"
while IFS= read -r item; do
  f="$(decode_input_path "$item")"
  echo "[$(date '+%F %T')] [extract] input=$item decoded=$f" >> "/tmp/OmegaZip-workflow.log"
  [[ -e "$f" ]] || continue
  stem="$(omegazip_stem "$f")"
  [[ -z "$stem" ]] && stem="archive"
  d="$(dirname "$f")"
  out="$d/${stem}_распаковано"
  "$OZ" decompress "$f" "$out" >> "/tmp/OmegaZip-workflow.log" 2>&1 && echo "Распаковано: $out"
done < <(collect_inputs "$@")
ENDEX
)

# Экранировать для XML (для вставки в plist string)
escape_xml() { sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g; s/"/\&quot;/g'; }

write_wflow() {
  local path="$1"
  local script="$2"
  local esc
  esc=$(printf '%s' "$script" | escape_xml)
  cat > "$path" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0">
<dict>
	<key>AMApplicationBuild</key><string>310</string>
	<key>AMApplicationVersion</key><string>2.2</string>
	<key>AMDocumentVersion</key><string>2</string>
	<key>actions</key>
	<array>
		<dict>
			<key>action</key>
			<dict>
				<key>AMAccepts</key>
				<dict>
					<key>Container</key><string>List</string>
					<key>Optional</key><true/>
					<key>Types</key><array><string>com.apple.cocoa.string</string></array>
				</dict>
				<key>AMActionVersion</key><string>2.0.3</string>
				<key>AMParameterProperties</key><dict/>
				<key>AMProvides</key>
				<dict>
					<key>Container</key><string>List</string>
					<key>Types</key><array><string>com.apple.cocoa.string</string></array>
				</dict>
				<key>ActionBundlePath</key><string>/System/Library/Automator/Run Shell Script.action</string>
				<key>ActionName</key><string>Run Shell Script</string>
				<key>ActionParameters</key>
				<dict>
					<key>COMMAND_STRING</key><string>${esc}</string>
					<key>CheckedForUserDefaultShell</key><true/>
					<key>inputMethod</key><integer>1</integer>
					<key>shell</key><string>/bin/zsh</string>
					<key>source</key><string>${esc}</string>
				</dict>
				<key>Application</key><array><string>Automator</string></array>
				<key>BundleIdentifier</key><string>com.apple.RunShellScript</string>
				<key>CFBundleVersion</key><string>2.0.3</string>
				<key>CanShowSelectedItemsWhenRun</key><false/>
				<key>CanShowWhenRun</key><true/>
				<key>Class Name</key><string>RunShellScriptAction</string>
				<key>arguments</key><dict/>
			</dict>
		</dict>
	</array>
	<key>connectors</key><dict/>
	<key>workflowMetaData</key>
	<dict>
		<key>serviceApplicationBundleID</key><string>com.apple.finder</string>
		<key>serviceInputTypeIdentifier</key><string>com.apple.Automator.fileSystemObject</string>
		<key>serviceOutputTypeIdentifier</key><string>com.apple.Automator.nothing</string>
		<key>serviceProcessesInput</key><integer>0</integer>
		<key>workflowTypeIdentifier</key><string>com.apple.Automator.servicesMenu</string>
	</dict>
</dict>
</plist>
EOF
}

# 3) Создать workflow «Сжать в OmegaZip»
mkdir -p "$SERVICES/Сжать в OmegaZip.workflow/Contents"
cat > "$SERVICES/Сжать в OmegaZip.workflow/Contents/Info.plist" << 'INFO'
<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0">
<dict>
	<key>CFBundleIdentifier</key><string>com.omegazip.compress.auto</string>
	<key>CFBundleName</key><string>Сжать в OmegaZip</string>
	<key>CFBundlePackageType</key><string>BNDL</string>
	<key>NSServices</key>
	<array>
		<dict>
			<key>NSMenuItem</key><dict><key>default</key><string>Сжать в OmegaZip</string></dict>
			<key>NSMessage</key><string>runWorkflowWithInput</string>
			<key>NSRequiredContext</key><dict><key>NSApplicationIdentifier</key><string>com.apple.finder</string></dict>
			<key>NSSendTypes</key><array><string>public.file-url</string></array>
		</dict>
	</array>
</dict>
</plist>
INFO
write_wflow "$SERVICES/Сжать в OmegaZip.workflow/Contents/document.wflow" "$COMPRESS_AUTO_SCRIPT"

# 4) Создать workflow «Распаковать в OmegaZip»
mkdir -p "$SERVICES/Распаковать в OmegaZip.workflow/Contents"
cat > "$SERVICES/Распаковать в OmegaZip.workflow/Contents/Info.plist" << 'INFO'
<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0">
<dict>
	<key>CFBundleIdentifier</key><string>com.omegazip.extract.auto</string>
	<key>CFBundleName</key><string>Распаковать в OmegaZip</string>
	<key>CFBundlePackageType</key><string>BNDL</string>
	<key>NSServices</key>
	<array>
		<dict>
			<key>NSMenuItem</key><dict><key>default</key><string>Распаковать в OmegaZip</string></dict>
			<key>NSMessage</key><string>runWorkflowWithInput</string>
			<key>NSRequiredContext</key><dict><key>NSApplicationIdentifier</key><string>com.apple.finder</string></dict>
			<key>NSSendTypes</key><array><string>public.file-url</string></array>
		</dict>
	</array>
</dict>
</plist>
INFO
write_wflow "$SERVICES/Распаковать в OmegaZip.workflow/Contents/document.wflow" "$EXTRACT_SCRIPT"

echo ""
echo "Готово. Установлено:"
echo "  $SERVICES/Сжать в OmegaZip.workflow"
echo "  $SERVICES/Распаковать в OmegaZip.workflow"
echo ""
echo "Перезапусти Finder, потом: ПКМ → Сервисы → Сжать в OmegaZip / Распаковать в OmegaZip"
killall Finder 2>/dev/null || true
sleep 1
open /System/Library/CoreServices/Finder.app
echo "Finder перезапущен."
