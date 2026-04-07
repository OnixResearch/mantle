## Context

`crunch-project` defines `RefreshResolver` as a trait with two methods:

```rust
pub trait RefreshResolver {
    fn resolve_git_rev(
        &self,
        repository: &str,
        reference: &GitReference,
    ) -> Result<Option<String>, Error>;

    fn hash_url_content(
        &self,
        url: &str,
        algo: &HashAlgo,
    ) -> Result<Option<String>, Error>;
}
```

The binary crate currently provides `StubResolver` returning `Ok(None)`.
Both methods are synchronous — crunch-project has no async dependency.

## Goals / Non-Goals

**Goals:** make `crunch refresh` and `crunch list-stale` work for real.
Keep the implementation straightforward — subprocess for git, blocking
HTTP for URL hashing.

**Non-Goals:** async resolution, parallel input resolution, caching
beyond the lockfile, progress reporting, retry logic.

## Decisions

### 1. LiveResolver struct in project_cmd.rs

**Choice:** Single `LiveResolver` struct implementing `RefreshResolver`,
defined in `src/project_cmd.rs` (or extracted to `src/resolve.rs` if it
exceeds ~70 lines).

**Rationale:** The resolver needs I/O (subprocess, HTTP). That belongs
in the binary crate's imperative shell, not in any library crate.
`project_cmd.rs` already owns the CLI dispatch for project commands, so
it's the natural home. If the implementation is compact enough, a
separate module isn't warranted.

**Alternative:** Put the resolver in crunch-pipeline or crunch-build.
Rejected — those crates don't depend on crunch-project and shouldn't.
The dependency arrow is: binary → crunch-project (for trait + types),
binary → reqwest/Command (for I/O).

### 2. Git resolution via std::process::Command

**Choice:** Run `git ls-remote <repo> <refspec>` as a subprocess, parse
stdout for the SHA.

**Rationale:** git ls-remote is fast (no clone), widely available, and
produces stable output. The refspec varies by ref type:
- branch: `refs/heads/<value>`
- tag: `refs/tags/<value>`
- rev: no ls-remote needed (validate format, return as-is)

**Implementation:**
```
git ls-remote <repo> <refspec>
→ "<sha>\t<ref>\n"
```
Parse the first whitespace-delimited token. If stdout is empty, the ref
doesn't exist — return an error. Tags may have `^{}` (peeled) entries;
prefer the peeled line if present (that's the commit, not the tag
object).

**Alternative:** libgit2 / git2 crate. Rejected — adds a C dependency
(libgit2-sys), and `git ls-remote` is already on PATH for `fetchGit`.

### 3. URL hashing via reqwest blocking + streaming hash

**Choice:** Use `reqwest::blocking::get()` with a size-limited read
loop, feeding bytes into the appropriate hasher, return SRI string.

**Rationale:** reqwest is already in the workspace (crunch-build fetcher
uses it). Blocking is fine — the resolver methods are synchronous per
the trait signature. Streaming avoids buffering the entire file in memory.

**Implementation:**
1. `reqwest::blocking::get(url)?`
2. Check `Content-Length` if present; reject if > MAX_DOWNLOAD_BYTES.
3. Read in chunks (8 KiB), feed into hasher, track total bytes.
4. Abort if total exceeds MAX_DOWNLOAD_BYTES.
5. Finalize hash, encode as SRI (`<algo>-<base64>`).

Supported hashers:
- sha256: `sha2::Sha256`
- sha512: `sha2::Sha512`
- blake3: `blake3::Hasher`

**Alternative:** Download to tempfile, then hash. Unnecessary
indirection — streaming hash is simpler and uses less disk.

### 4. Error handling: per-input, not abort-all

**Choice:** Each resolver method returns `Result<Option<String>, Error>`.
`Err` means resolution failed for this input. The caller
(`refresh_inputs` in crunch-project) already handles per-input errors
and continues to the next input.

**Rationale:** Matches the existing trait contract. One unreachable git
repo shouldn't block refreshing a dozen URL inputs.

`Ok(None)` is reserved for "resolution not applicable" (e.g., rev
references that don't need network). It does NOT mean "I'm a stub".

## Risks / Trade-offs

**[git not on PATH]** → Clear error message. Already the case for
`fetchGit` builds. Users who don't use git inputs are unaffected.

**[Large downloads for hash_url_content]** → MAX_DOWNLOAD_BYTES limit
(4 GiB, same as fetcher). For refresh, this is generous — most source
tarballs are <500 MiB.

**[HTTP redirects / auth]** → reqwest handles redirects by default.
Auth (private repos, token-gated URLs) is out of scope for this change.
Future work could add header/token config to the manifest.

**[Blocking I/O in sync context]** → The trait is sync, the CLI is
the caller. No risk of blocking a tokio runtime. If crunch later needs
async resolution (parallel refresh), the trait signature changes — that's
a future openspec.
