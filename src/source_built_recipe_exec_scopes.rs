//! Original source timeout arenas survive real request/continuation boundaries.
use super::*;
#[derive(Default)]
pub(super) struct SourceScopes {
    seen: BTreeMap<String, ScopeRecord>,
    active: Vec<String>,
}
struct ScopeRecord {
    seconds_max: u64,
    monotonic_deadline_ns: u64,
    deadline: Instant,
    closed: bool,
}
fn monotonic_ns() -> Result<u64, RunError> {
    let mut time = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut time) } != 0 {
        return Err(error("reading actual monotonic source clock"));
    }
    u64::try_from(time.tv_sec)
        .ok()
        .and_then(|s| s.checked_mul(1_000_000_000))
        .and_then(|n| n.checked_add(time.tv_nsec as u64))
        .ok_or_else(|| error("source monotonic clock overflow"))
}
impl SourceScopes {
    pub(super) fn update(&mut self, attributes: Option<&RecipeProcessAttributes>) -> Result<(), RunError> {
        let scopes = attributes.map_or(&[][..], |attributes| attributes.timeout_scopes.as_slice());
        let common = self.active.iter().zip(scopes).take_while(|(old, new)| old.as_str() == new.id).count();
        for id in self.active.drain(common..) {
            self.seen.get_mut(&id).expect("active original timeout identity").closed = true;
        }
        for scope in scopes {
            if let Some(original) = self.seen.get(&scope.id) {
                if original.closed {
                    return Err(error("original timeout scope reentered after return"));
                }
                if original.seconds_max != scope.seconds_max
                    || Some(original.monotonic_deadline_ns) != scope.monotonic_deadline_ns
                {
                    return Err(error("original timeout scope identity or deadline changed"));
                }
                continue;
            }
            let now = Instant::now();
            let deadline = if let Some(absolute) = scope.monotonic_deadline_ns {
                let clock = monotonic_ns()?;
                let duration = scope
                    .seconds_max
                    .checked_mul(1_000_000_000)
                    .ok_or_else(|| error("source timeout duration overflow"))?;
                if absolute > clock.checked_add(duration).ok_or_else(|| error("source timeout clock overflow"))? {
                    return Err(error("source timeout deadline increases its declared original arena"));
                }
                let remaining = absolute
                    .checked_sub(clock)
                    .filter(|ns| *ns > 0)
                    .ok_or_else(|| error("actual original source timeout expired"))?;
                now.checked_add(Duration::from_nanos(remaining))
            } else {
                return Err(error("source timeout scope lacks its authenticated original entry deadline"));
            }
            .ok_or_else(|| error("source timeout deadline overflow"))?;
            self.seen.insert(scope.id.clone(), ScopeRecord {
                seconds_max: scope.seconds_max,
                monotonic_deadline_ns: scope.monotonic_deadline_ns.expect("authenticated original deadline"),
                deadline,
                closed: false,
            });
        }
        self.active.extend(scopes[common..].iter().map(|scope| scope.id.clone()));
        self.check()
    }
    pub(super) fn deadline(&self, outer: Instant) -> Instant {
        self.active.iter().fold(outer, |deadline, id| deadline.min(self.seen[id].deadline))
    }
    pub(super) fn check(&self) -> Result<(), RunError> {
        let now = Instant::now();
        if self.active.iter().any(|id| now >= self.seen[id].deadline) {
            return Err(error("actual original source timeout arena expired"));
        }
        Ok(())
    }
    pub(super) fn finish(&mut self) {
        for id in self.active.drain(..) {
            self.seen.get_mut(&id).expect("active original timeout identity").closed = true;
        }
    }
}
