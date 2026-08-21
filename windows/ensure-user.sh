#!/usr/bin/env bash
# Idempotent. Run as root inside a WSL distro.
# Configures an existing user: NOPASSWD sudo + /etc/wsl.conf default=.
# Does not create users or lock passwords.
# Usage: ensure-user.sh [linux-user]
set -euo pipefail

u="${1:-}"

if [ -n "$u" ] && ! id -u "$u" >/dev/null 2>&1; then
  u=""
fi
if [ -z "$u" ] && getent passwd 1000 >/dev/null; then
  u="$(getent passwd 1000 | cut -d: -f1)"
fi
if [ -z "$u" ] && [ -d /home ]; then
  u="$(find /home -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | head -n1)"
fi
if [ -z "$u" ] || ! id -u "$u" >/dev/null 2>&1; then
  echo "no linux user to configure — finish the WSL username/password prompt first" >&2
  exit 1
fi

usermod -aG sudo,adm,wheel "$u" 2>/dev/null || usermod -aG wheel "$u" 2>/dev/null || usermod -aG sudo,adm "$u" 2>/dev/null || true

printf '%s ALL=(ALL) NOPASSWD:ALL\n' "$u" >"/etc/sudoers.d/90-${u}"
chmod 440 "/etc/sudoers.d/90-${u}"
if command -v visudo >/dev/null; then
  visudo -cf "/etc/sudoers.d/90-${u}" >/dev/null
fi

conf=/etc/wsl.conf
tmp="$(mktemp)"
# WSL sometimes writes NUL bytes into wsl.conf ("Invalid key name" on every shell).
if [ -f "$conf" ]; then
  tr -d '\0' <"$conf" | awk '
    /^\[user\]/ {skip=1; next}
    /^\[interop\]/ {skip=1; next}
    /^\[/ {skip=0}
    !skip {print}
  ' >"$tmp"
else
  : >"$tmp"
fi
{
  awk '{ lines[NR]=$0 } END { n=NR; while (n>0 && lines[n]=="") n--; for (i=1;i<=n;i++) print lines[i] }' "$tmp"
  printf '\n[user]\ndefault=%s\n\n[interop]\nenabled=true\n' "$u"
} >"${tmp}.out"
mv "${tmp}.out" "$conf"
rm -f "$tmp"
chmod 644 "$conf"

echo "ok user=$u"
