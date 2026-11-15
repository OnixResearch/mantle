#!/usr/bin/env bash
# Read-only audit of BOTH frozen/published AWS-LC original copies against pinned Git.
set -euo pipefail
r=/home/brittonr/sbfp-20261113-berdKH
f="$r/runner-source-8fa84260-aws/bootstrap/source-inputs/aws-lc-original-02561621ffa4cf17c0c4f70bc11a82df36b42ae9"
c="$r/prepared-cohort-8fa84260-aws/inputs/mantle-source/bootstrap/source-inputs/aws-lc-original-02561621ffa4cf17c0c4f70bc11a82df36b42ae9"
db="$r/aws-original-source-02561621.TGISLN/aws-lc.git"
rev=02561621ffa4cf17c0c4f70bc11a82df36b42ae9
tree=969d75db5c0494de4a26bbb2dcbf6837058054c1
expected=8f731ad6e1c7b40d9e046a788dd7a7a088686b098e02d83c89750e7965a17f89
git=/run/current-system/sw/bin/git
tar=/run/current-system/sw/bin/tar
b3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
fail(){ printf 'BLOCKER %s\n' "$*" >&2; exit 3; }
[[ $("$git" --git-dir="$db" cat-file -t "$rev") == commit && $("$git" --git-dir="$db" rev-parse "$rev^{tree}") == "$tree" ]] || fail 'pinned Git commit/tree unavailable'
printf 'PIN commit=%s tree=%s git_db=%s\n' "$rev" "$tree" "$db"
for label in FROZEN COHORT; do
  if [[ $label == FROZEN ]]; then root=$f; else root=$c; fi
  [[ -d "$root" && ! -L "$root" ]] || fail "$label original root missing or symlink"
  "$git" --git-dir="$db" ls-tree -r -z "$rev" | perl -MFile::Find -0ne '
    BEGIN { $root=shift @ARGV; $count=0; $exec=0; $regular=0; }
    chomp;
    my ($mode,$type,$oid,$rel)=/^([0-7]{6}) (\S+) ([a-f0-9]{40})\t(.*)$/s or die "invalid git tree entry\n";
    die "unexpected object/mode $rel\n" unless $type eq "blob" && ($mode eq "100644" || $mode eq "100755");
    die "unsafe Git path $rel\n" if $rel =~ /[\r\n]/ || exists $paths{$rel} || $rel =~ m{(?:^|/)\.\.(?:/|$)};
    $paths{$rel}=1; $count++;
    my @s=lstat("$root/$rel"); die "missing original $rel\n" unless @s;
    die "nonregular original $rel\n" unless -f _ && !-l _;
    my $required=$mode eq "100755" ? 0755 : 0644;
    die sprintf("wrong mode %s actual=%04o wanted=%04o\n",$rel,$s[2]&07777,$required) unless ($s[2]&07777)==$required;
    $exec++ if $required==0755; $regular++ if $required==0644;
    END {
      my $seen=0;
      find({no_chdir=>1,wanted=>sub {my $path=$File::Find::name; my @s=lstat($path);die "stat $path: $!\n" unless @s;
        return if -d _;
        my $rel=substr($path,length($root)+1);
        die "extra or nonregular original $rel\n" unless -f _ && !-l _ && exists $paths{$rel}; $seen++;
      }},$root);
      die "file count mismatch expected=$count actual=$seen\n" if $seen!=$count;
      printf "TREE_FILES %s total=%d regular_0644=%d executable_0755=%d extra=0 symlinks=0\n",$root,$count,$regular,$exec;
    }
  ' "$root" || fail "$label path/mode inventory differs from Git"
  diff -q <("$git" --git-dir="$db" ls-tree -r --format='%(objectname)' "$rev") <("$git" --git-dir="$db" ls-tree -r --name-only -z "$rev" | perl -0ne 'BEGIN { $root=shift @ARGV; } chomp; die "newline in Git path" if /[\r\n]/;print "$root/$_\n"' "$root" | "$git" hash-object --stdin-paths) || fail "$label Git blob hashes differ"
  printf 'GIT_BLOBS %s match=true\n' "$label"
  observed=$("$tar" --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$root" . | "$b3" --no-names)
  [[ $observed == "$expected" ]] || fail "$label normalized tar differs: $observed"
  printf 'NORMALIZED_GNU_TAR %s blake3=%s\n' "$label" "$observed"
done
printf 'ORIGINAL_CONCLUSION frozen_and_cohort_match_pinned_git=true\n'
