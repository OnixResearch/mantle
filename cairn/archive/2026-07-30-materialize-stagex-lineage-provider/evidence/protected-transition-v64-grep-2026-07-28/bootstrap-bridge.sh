#!/bin/sh
quiet=0
invert=0
pat=
while [ $# -gt 0 ]; do
  case "$1" in
    --version) echo 'grep (GNU grep) 2.4'; exit 0 ;;
    --help) echo 'usage: grep OPTIONS PATTERN FILES'; exit 0 ;;
    -q) quiet=1; shift ;;
    -v) invert=1; shift ;;
    -i|-n|-E|-F) shift ;;
    -qi|-iq) quiet=1; shift ;;
    -e) shift; pat="$1"; shift ;;
    -*) break ;;
    *) break ;;
  esac
done
if [ -z "$pat" ]; then pat="$1"; shift; fi
[ -n "$pat" ] || exit 2
match_line() {
  line="$1"
  p="$pat"
  case "$p" in
    ^*) p=${p#^}; case "$line" in "$p"*) return 0 ;; *) return 1 ;; esac ;;
    *'$') p=${p%$}; case "$line" in *"$p") return 0 ;; *) return 1 ;; esac ;;
    *) case "$line" in *"$p"*) return 0 ;; *) return 1 ;; esac ;;
  esac
}
scan_stream() {
  found=1
  while IFS= read -r line; do
    if match_line "$line"; then m=0; else m=1; fi
    if [ "$invert" = 1 ]; then if [ "$m" = 0 ]; then m=1; else m=0; fi; fi
    if [ "$m" = 0 ]; then
      [ "$quiet" = 1 ] && exit 0
      echo "$line"
      found=0
    fi
  done
  return $found
}
status=1
if [ $# -eq 0 ]; then
  scan_stream && status=0
else
  for f in "$@"; do
    [ -f "$f" ] || continue
    scan_stream < "$f" && status=0
  done
fi
exit $status