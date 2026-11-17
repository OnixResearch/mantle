#!/usr/bin/env bash
set -euo pipefail
ssh -o BatchMode=yes -o ConnectTimeout=20 leviathan.cymric-daggertooth.ts.net '/nix/store/c4khxwf5169c0m01gmz5hlflqpqihhz7-pueue-4.0.1/bin/pueue wait --quiet 1684 && test -f /home/brittonr/sbfp-20261113-berdKH/review-subject-facts-df51377d-unit-plans.json && test "$(cat /home/brittonr/sbfp-20261113-berdKH/review-subject-facts-df51377d-unit-plans.exit)" = 0 && /run/current-system/sw/bin/cat /home/brittonr/sbfp-20261113-berdKH/prepared-cohort-df51377d-unit-plans/source-profile.json' | b3sum --no-names
