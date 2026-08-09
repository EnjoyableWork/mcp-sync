#!/usr/bin/env bash

set -euo pipefail

security_assurance_repository=${1:-}
security_assurance_badge_project_id=${2:-}
security_assurance_expected_main=${3:-}
security_assurance_closeout_mode=${4:-}

if [[ $# -lt 3 ]] || [[ $# -gt 4 ]] ||
  [[ "$security_assurance_repository" != EnjoyableWork/mcp-sync ]] ||
  [[ ! "$security_assurance_badge_project_id" =~ ^[0-9]+$ ]] ||
  [[ ! "$security_assurance_expected_main" =~ ^[0-9a-f]{40}$ ]] ||
  [[ -n "$security_assurance_closeout_mode" && "$security_assurance_closeout_mode" != --require-closed ]]; then
  echo "usage: $0 EnjoyableWork/mcp-sync BADGE_PROJECT_ID EXPECTED_MAIN_SHA [--require-closed]" >&2
  exit 2
fi

for security_assurance_command in curl jq grep; do
  if ! command -v "$security_assurance_command" >/dev/null 2>&1; then
    echo "public security-assurance verification requires $security_assurance_command" >&2
    exit 2
  fi
done

security_assurance_temp_parent=${TMPDIR:-/tmp}
security_assurance_temp_prefix="${security_assurance_temp_parent%/}/mcp-sync-security-assurance."
security_assurance_temp="$(mktemp -d "${security_assurance_temp_prefix}XXXXXX")"
cleanup_security_assurance() {
  if [[ "$security_assurance_temp" != "$security_assurance_temp_prefix"* ]]; then
    echo "refusing to remove an unexpected security-assurance path" >&2
    return 1
  fi
  if [[ -d "$security_assurance_temp" ]]; then
    rm -rf -- "$security_assurance_temp"
  fi
}
trap cleanup_security_assurance EXIT

security_assurance_user_agent='mcp-sync-security-assurance-verifier/0.1 (+https://github.com/EnjoyableWork/mcp-sync)'
security_assurance_curl=(
  curl
  --fail
  --silent
  --show-error
  --location
  --proto '=https'
  --proto-redir '=https'
  --tlsv1.2
  --header "User-Agent: $security_assurance_user_agent"
)

security_assurance_badge_base="https://www.bestpractices.dev/projects/$security_assurance_badge_project_id"
security_assurance_project_url="$security_assurance_badge_base"
security_assurance_assessment_url="https://www.bestpractices.dev/en/projects/$security_assurance_badge_project_id/baseline-1"
security_assurance_badge_url="$security_assurance_badge_base/baseline"

security_assurance_project_json="$security_assurance_temp/project.json"
"${security_assurance_curl[@]}" \
  --output "$security_assurance_project_json" \
  "$security_assurance_badge_base.json"
jq -e \
  --argjson project_id "$security_assurance_badge_project_id" \
  '.id == $project_id and
   .name == "mcp-sync" and
   .description ==
     "A local configuration CLI that defines MCP servers once and reconciles them into supported MCP clients." and
   .homepage_url == "https://github.com/EnjoyableWork/mcp-sync" and
   .repo_url == "https://github.com/EnjoyableWork/mcp-sync" and
   .license == "MIT" and
   .implementation_languages == "Rust, Shell, PowerShell" and
   .badge_percentage_baseline_1 == 100 and
   .baseline_tiered_percentage == 100 and
   .achieved_baseline_1_at == "2026-08-09T21:00:26.124Z" and
   .first_achieved_baseline_1_at == "2026-08-09T21:00:26.124Z" and
   .lost_baseline_1_at == null' \
  "$security_assurance_project_json" >/dev/null

security_assurance_project_page="$security_assurance_temp/project.html"
"${security_assurance_curl[@]}" \
  --output "$security_assurance_project_page" \
  "$security_assurance_project_url"
grep -F "/en/projects/$security_assurance_badge_project_id/baseline-1" \
  "$security_assurance_project_page" >/dev/null
grep -F 'value="mcp-sync" name="project[name]"' \
  "$security_assurance_project_page" >/dev/null

security_assurance_assessment="$security_assurance_temp/assessment.html"
"${security_assurance_curl[@]}" \
  --output "$security_assurance_assessment" \
  "$security_assurance_assessment_url"
grep -F 'These are criteria version v2026.02.19.' \
  "$security_assurance_assessment" >/dev/null
grep -F '<span class="satisfaction-text">24/24</span>' \
  "$security_assurance_assessment" >/dev/null

security_assurance_controls=(
  OSPS-AC-01.01
  OSPS-AC-02.01
  OSPS-AC-03.01
  OSPS-AC-03.02
  OSPS-BR-01.01
  OSPS-BR-01.03
  OSPS-BR-03.01
  OSPS-BR-03.02
  OSPS-BR-07.01
  OSPS-DO-01.01
  OSPS-DO-02.01
  OSPS-GV-02.01
  OSPS-GV-03.01
  OSPS-LE-02.01
  OSPS-LE-02.02
  OSPS-LE-03.01
  OSPS-LE-03.02
  OSPS-QA-01.01
  OSPS-QA-01.02
  OSPS-QA-02.01
  OSPS-QA-04.01
  OSPS-QA-05.01
  OSPS-QA-05.02
  OSPS-VM-02.01
)
for security_assurance_control in "${security_assurance_controls[@]}"; do
  security_assurance_control_field="$({
    printf '%s' "$security_assurance_control" | tr '[:upper:].-' '[:lower:]__'
  })"
  grep -F \
    "value=\"Met\" checked=\"checked\" name=\"project[${security_assurance_control_field}_status]\"" \
    "$security_assurance_assessment" >/dev/null
done

security_assurance_badge="$security_assurance_temp/baseline.svg"
security_assurance_badge_headers="$security_assurance_temp/baseline.headers"
"${security_assurance_curl[@]}" \
  --dump-header "$security_assurance_badge_headers" \
  --output "$security_assurance_badge" \
  "$security_assurance_badge_url"
grep -Eiq '^content-type:[[:space:]]*image/svg\+xml([[:space:]]|$)' \
  "$security_assurance_badge_headers"
grep -F 'aria-label="openssf baseline v2026.02.19: 1"' \
  "$security_assurance_badge" >/dev/null

security_assurance_main_json="$security_assurance_temp/main.json"
"${security_assurance_curl[@]}" \
  --header 'Accept: application/vnd.github+json' \
  --header 'X-GitHub-Api-Version: 2026-03-10' \
  --output "$security_assurance_main_json" \
  "https://api.github.com/repos/$security_assurance_repository/commits/main"
jq -e \
  --arg expected_main "$security_assurance_expected_main" \
  '.sha == $expected_main and
   (.commit.author.name | type == "string" and length > 0) and
   (.commit.author.date | type == "string" and length > 0) and
   (.commit.committer.name | type == "string" and length > 0) and
   (.commit.committer.date | type == "string" and length > 0)' \
  "$security_assurance_main_json" >/dev/null

security_assurance_raw_base="https://raw.githubusercontent.com/$security_assurance_repository/$security_assurance_expected_main"
security_assurance_readme="$security_assurance_temp/README.md"
security_assurance_contract="$security_assurance_temp/security-assurance.md"
security_assurance_project="$security_assurance_temp/PROJECT.md"
security_assurance_proposal="$security_assurance_temp/bestpractices.json"
"${security_assurance_curl[@]}" --output "$security_assurance_readme" \
  "$security_assurance_raw_base/README.md"
"${security_assurance_curl[@]}" --output "$security_assurance_contract" \
  "$security_assurance_raw_base/docs/security-assurance.md"
"${security_assurance_curl[@]}" --output "$security_assurance_project" \
  "$security_assurance_raw_base/PROJECT.md"
"${security_assurance_curl[@]}" --output "$security_assurance_proposal" \
  "$security_assurance_raw_base/.bestpractices.json"

jq -e '
    type == "object" and length == 52 and
    .name == "mcp-sync" and
    .description ==
      "A local configuration CLI that defines MCP servers once and reconciles them into supported MCP clients." and
    .license == "MIT" and
    .implementation_languages == "Rust, Shell, PowerShell"
  ' "$security_assurance_proposal" >/dev/null
for security_assurance_control in "${security_assurance_controls[@]}"; do
  security_assurance_control_field="$({
    printf '%s' "$security_assurance_control" | tr '[:upper:].-' '[:lower:]__'
  })"
  jq -e \
    --arg status "${security_assurance_control_field}_status" \
    --arg justification "${security_assurance_control_field}_justification" '
      .[$status] == "Met" and
      (.[$justification] | type == "string" and contains("https://github.com/EnjoyableWork/"))
    ' "$security_assurance_proposal" >/dev/null
done

security_assurance_badge_markdown="[![OpenSSF Baseline]($security_assurance_badge_url)]($security_assurance_project_url)"
grep -F "$security_assurance_badge_markdown" "$security_assurance_readme" >/dev/null
grep -F "$security_assurance_assessment_url" "$security_assurance_contract" >/dev/null
if grep -F 'MCP-035: replace this line' "$security_assurance_contract" >/dev/null; then
  echo "public security-assurance contract retains its unpublished badge placeholder" >&2
  exit 1
fi

for security_assurance_control in "${security_assurance_controls[@]}"; do
  grep -F "| \`${security_assurance_control}\` | Pass |" \
    "$security_assurance_contract" >/dev/null
done

for security_assurance_required_claim in \
  "OpenSSF OSPS Baseline \`v2026.02.19\` Level 1" \
  'This is a maintainer self-assessment, not an independent certification' \
  'SLSA v1.0 Build Level 2 artifact statement' \
  'Reassess the complete baseline at least annually'; do
  grep -F "$security_assurance_required_claim" \
    "$security_assurance_contract" >/dev/null
done

if [[ "$security_assurance_closeout_mode" == --require-closed ]]; then
  grep -F '| MCP-035 | Self-assess, publish, and showcase the zero-cost enterprise assurance baseline | M3 | P1 | Codex | Done |' \
    "$security_assurance_project" >/dev/null
  grep -F '| M3 | Trusted project — enterprise assurance and adoption ' \
    "$security_assurance_project" | \
    grep -F "| Done — \`MCP-030\` through \`MCP-035\` Done |" >/dev/null
fi

security_assurance_rendered_readme="$security_assurance_temp/README.html"
"${security_assurance_curl[@]}" \
  --request POST \
  --header 'Content-Type: text/plain' \
  --data-binary "@$security_assurance_readme" \
  --output "$security_assurance_rendered_readme" \
  'https://api.github.com/markdown/raw'
grep -F "<a href=\"$security_assurance_project_url\"" \
  "$security_assurance_rendered_readme" >/dev/null
grep -F 'alt="OpenSSF Baseline"' "$security_assurance_rendered_readme" >/dev/null
grep -F "data-canonical-src=\"$security_assurance_badge_url\"" \
  "$security_assurance_rendered_readme" >/dev/null

printf 'Verified OpenSSF OSPS Baseline v2026.02.19 Level 1 at 24/24, its official dynamic badge and rendered exact-main README destination, and the public assurance contract on %s.\n' \
  "$security_assurance_expected_main"
