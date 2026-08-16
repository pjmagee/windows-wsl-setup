#!/usr/bin/env bash
# Idempotent. Run as root inside Ubuntu-26.04.
# Usage: ensure-user.sh <linux-user> [--create]
#   default: configure an existing user (uid 1000 or the named account)
#   --create: create the named user if missing, lock the password
set -euo pipefail

u="${1:?usage: ensure-user.sh <linux-user> [--create]}"
create=0
[ "${2:-}" = "--create" ] && create=1

if ! [[ "$u" =~ ^[a-z_][a-z0-9_-]{0,31}$ ]]; then
  echo "invalid linux username: $u" >&2
  exit 1
fi

if [ "$create" -eq 0 ] && ! id -u "$u" >/dev/null 2>&1; then
  if getent passwd 1000 >/dev/null; then
    u="$(getent passwd 1000 | cut -d: -f1)"
  elif [ -d /home ] && [ -n "$(ls -A /home 2>/dev/null || true)" ]; then
    u="$(find /home -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | head -n1)"
  else
    create=1
  fi
fi

created=0
if ! id -u "$u" >/dev/null 2>&1; then
  if [ "$create" -eq 0 ]; then
    echo "no linux user to configure" >&2
    exit 1
  fi
  extra=()
  for g in adm sudo dialout cdrom floppy audio dip video plugdev netdev; do
    getent group "$g" >/dev/null && extra+=("$g")
  done
  if [ "${#extra[@]}" -gt 0 ]; then
    IFS=,
    useradd -m -s /bin/bash -G "${extra[*]}" "$u"
    unset IFS
  else
    useradd -m -s /bin/bash "$u"
  fi
  created=1
fi

usermod -aG sudo,adm "$u" 2>/dev/null || true
if [ "$created" -eq 1 ]; then
  passwd -l "$u" >/dev/null
fi

printf '%s ALL=(ALL) NOPASSWD:ALL\n' "$u" >"/etc/sudoers.d/90-${u}"
chmod 440 "/etc/sudoers.d/90-${u}"
if command -v visudo >/dev/null; then
  visudo -cf "/etc/sudoers.d/90-${u}" >/dev/null
fi

conf=/etc/wsl.conf
tmp="$(mktemp)"
if [ -f "$conf" ]; then
  awk '
    /^\[user\]/ {skip=1; next}
    /^\[interop\]/ {skip=1; next}
    /^\[/ {skip=0}
    !skip {print}
  ' "$conf" >"$tmp"
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
