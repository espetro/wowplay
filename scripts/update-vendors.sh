#!/usr/bin/env bash
# Vendor update helper — checks or fetches upstream pinned artifacts.
#
# Usage:
#   update-vendors.sh --check   print stale/up-to-date status, exit 2 if any stale
#   update-vendors.sh           download and update all stale artifacts

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
VENDORS_TOML="${REPO_ROOT}/vendor/prebuilt/VENDORS.toml"
CHECKSUMS="${REPO_ROOT}/vendor/prebuilt/CHECKSUMS.sha256"
PREBUILT="${REPO_ROOT}/vendor/prebuilt"

CHECK_ONLY=false
[[ "${1:-}" == "--check" ]] && CHECK_ONLY=true

# ── TOML helpers (stdlib tomllib, Python 3.11+) ───────────────────────────────

toml_keys() {
  python3 - "$1" <<'EOF'
import sys, tomllib
with open(sys.argv[1], "rb") as f:
    d = tomllib.load(f)
for k in d:
    print(k)
EOF
}

toml_get() {
  python3 - "$1" "$2" "$3" <<'EOF'
import sys, tomllib
with open(sys.argv[1], "rb") as f:
    d = tomllib.load(f)
print(d[sys.argv[2]][sys.argv[3]])
EOF
}

# ── GitHub tag helpers ────────────────────────────────────────────────────────

get_latest_tag() {
  local repo="$1"
  if command -v gh &>/dev/null; then
    gh api "repos/${repo}/releases/latest" --jq '.tag_name'
  elif [[ -n "${GITHUB_TOKEN:-}" ]]; then
    curl -fsSL -H "Authorization: token ${GITHUB_TOKEN}" \
      "https://api.github.com/repos/${repo}/releases/latest" \
      | python3 -c "import sys,json; print(json.load(sys.stdin)['tag_name'])"
  else
    curl -fsSL \
      "https://api.github.com/repos/${repo}/releases/latest" \
      | python3 -c "import sys,json; print(json.load(sys.stdin)['tag_name'])"
  fi
}

# ── PE32 validation ───────────────────────────────────────────────────────────

validate_pe_dll() {
  local path="$1"
  python3 - "$path" <<'EOF'
import sys
with open(sys.argv[1], "rb") as f:
    mz = f.read(2)
    if mz != b"MZ":
        sys.exit(f"error: not a PE file (bad MZ header): {sys.argv[1]}")
    f.seek(0x3C)
    pe_offset = int.from_bytes(f.read(4), "little")
    f.seek(pe_offset)
    sig = f.read(4)
    if sig != b"PE\x00\x00":
        sys.exit(f"error: not a PE file (bad PE signature at 0x{pe_offset:x}): {sys.argv[1]}")
EOF
}

# ── TOML / checksum writers ───────────────────────────────────────────────────

update_toml_tag() {
  local file="$1" table="$2" new_tag="$3"
  python3 - "$file" "$table" "$new_tag" <<'EOF'
import sys, re

file, table, new_tag = sys.argv[1], sys.argv[2], sys.argv[3]
with open(file) as f:
    text = f.read()

# Match the target table header, then replace the first pinned_tag = "..." within it
pattern = r'(\[' + re.escape(table) + r'\][^\[]*?pinned_tag\s*=\s*")[^"]*(")'
replacement = r'\g<1>' + new_tag + r'\2'
new_text, n = re.subn(pattern, replacement, text, count=1, flags=re.DOTALL)
if n == 0:
    sys.exit(f"error: could not find pinned_tag in table [{table}]")
with open(file, "w") as f:
    f.write(new_text)
EOF
}

update_checksums() {
  local file="$1" rel_path="$2" new_sha="$3"
  python3 - "$file" "$rel_path" "$new_sha" <<'EOF'
import sys, re

file, rel_path, new_sha = sys.argv[1], sys.argv[2], sys.argv[3]
with open(file) as f:
    text = f.read()

pattern = r'^[0-9a-f]{64}(\s+' + re.escape(rel_path) + r')$'
replacement = new_sha + r'\1'
new_text, n = re.subn(pattern, replacement, text, flags=re.MULTILINE)
if n == 0:
    sys.exit(f"error: no checksum line found for {rel_path}")
with open(file, "w") as f:
    f.write(new_text)
EOF
}

# ── Fetch helpers ─────────────────────────────────────────────────────────────

download_release_tarball() {
  local upstream="$1" tag="$2" asset_filter="$3" inner_path="$4" dest="$5"
  local tmpdir
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "${tmpdir}"' RETURN

  echo "  Fetching release assets for ${upstream}@${tag}…"
  local asset_url
  if command -v gh &>/dev/null; then
    asset_url="$(gh api "repos/${upstream}/releases/tags/${tag}" \
      --jq ".assets[] | select(.name | contains(\"${asset_filter}\")) | .browser_download_url" \
      | head -1)"
  else
    asset_url="$(curl -fsSL "https://api.github.com/repos/${upstream}/releases/tags/${tag}" \
      | python3 -c "
import sys, json
assets = json.load(sys.stdin)['assets']
filt = '${asset_filter}'
url = next(a['browser_download_url'] for a in assets if filt in a['name'])
print(url)")"
  fi

  if [[ -z "${asset_url}" ]]; then
    echo "error: no asset matching '${asset_filter}' in ${upstream}@${tag}" >&2
    exit 1
  fi

  echo "  Downloading ${asset_url}…"
  curl -fsSL -L "${asset_url}" -o "${tmpdir}/archive.tar.gz"
  tar -xf "${tmpdir}/archive.tar.gz" -C "${tmpdir}"
  local extracted="${tmpdir}/${inner_path}"
  if [[ ! -f "${extracted}" ]]; then
    echo "error: '${inner_path}' not found in tarball" >&2
    exit 1
  fi
  validate_pe_dll "${extracted}"
  cp "${extracted}" "${dest}"
}

download_repo_file() {
  local upstream="$1" tag="$2" repo_path="$3" dest="$4"
  local url="https://raw.githubusercontent.com/${upstream}/${tag}/${repo_path}"
  echo "  Downloading ${url}…"
  curl -fsSL -L "${url}" -o "${dest}"
  validate_pe_dll "${dest}"
}

# ── Main loop ─────────────────────────────────────────────────────────────────

stale_keys=()

while IFS= read -r key; do
  upstream="$(toml_get "${VENDORS_TOML}" "${key}" "upstream")"
  pinned_tag="$(toml_get "${VENDORS_TOML}" "${key}" "pinned_tag")"
  fetch_type="$(toml_get "${VENDORS_TOML}" "${key}" "fetch_type")"
  local_path="$(toml_get "${VENDORS_TOML}" "${key}" "local_path")"

  echo "Checking ${key} (${upstream})…"
  latest_tag="$(get_latest_tag "${upstream}")"

  if [[ "${pinned_tag}" == "${latest_tag}" ]]; then
    echo "  up-to-date (${pinned_tag})"
    continue
  fi

  echo "  stale: pinned=${pinned_tag} latest=${latest_tag}"
  stale_keys+=("${key}")

  if $CHECK_ONLY; then
    continue
  fi

  dest="${PREBUILT}/${local_path}"
  mkdir -p "$(dirname "${dest}")"

  if [[ "${fetch_type}" == "release_tarball" ]]; then
    asset_filter="$(toml_get "${VENDORS_TOML}" "${key}" "asset_filter")"
    inner_path="$(toml_get "${VENDORS_TOML}" "${key}" "inner_path")"
    download_release_tarball "${upstream}" "${latest_tag}" "${asset_filter}" "${inner_path}" "${dest}"
  elif [[ "${fetch_type}" == "repo_file" ]]; then
    repo_path="$(toml_get "${VENDORS_TOML}" "${key}" "repo_path")"
    download_repo_file "${upstream}" "${latest_tag}" "${repo_path}" "${dest}"
  else
    echo "error: unknown fetch_type '${fetch_type}' for ${key}" >&2
    exit 1
  fi

  new_sha="$(shasum -a 256 "${dest}" | awk '{print $1}')"
  echo "  Updated ${local_path} (sha256: ${new_sha})"

  update_toml_tag "${VENDORS_TOML}" "${key}" "${latest_tag}"
  update_checksums "${CHECKSUMS}" "${local_path}" "${new_sha}"
done < <(toml_keys "${VENDORS_TOML}")

if [[ ${#stale_keys[@]} -gt 0 ]]; then
  if $CHECK_ONLY; then
    echo "Stale artifacts: ${stale_keys[*]}"
    exit 2
  else
    echo "All stale artifacts updated."
  fi
else
  echo "All vendors up-to-date."
fi
