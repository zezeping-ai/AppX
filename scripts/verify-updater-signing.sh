#!/usr/bin/env bash
# 本地验证 updater 签名密钥与密码，确认后再写入 GitHub Secrets。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
KEY_FILE="${1:-$ROOT/src-tauri/keys/updater.key}"

if [[ ! -f "$KEY_FILE" ]]; then
  echo "missing key file: $KEY_FILE" >&2
  exit 1
fi

if [[ -z "${TAURI_SIGNING_PRIVATE_KEY_PASSWORD:-}" ]]; then
  read -r -s -p "Updater key password: " TAURI_SIGNING_PRIVATE_KEY_PASSWORD
  echo
  export TAURI_SIGNING_PRIVATE_KEY_PASSWORD
fi

# 与 CI 保持一致的规范化方式
export TAURI_SIGNING_PRIVATE_KEY="$(
  python3 -c 'import sys; print(sys.stdin.read().translate(str.maketrans("", "", "\r\n\t ")))' < "$KEY_FILE"
)"

test_file="$(mktemp)"
trap 'rm -f "$test_file" "$test_file.sig"' EXIT
printf 'verify' > "$test_file"

(
  cd "$ROOT"
  pnpm tauri signer sign "$test_file"
)

echo
echo "Signing credentials OK."
echo "GitHub Secrets:"
echo "  TAURI_SIGNING_PRIVATE_KEY        = contents of $KEY_FILE (single line, no extra spaces)"
echo "  TAURI_SIGNING_PRIVATE_KEY_PASSWORD = the password you just used (length: ${#TAURI_SIGNING_PRIVATE_KEY_PASSWORD})"
