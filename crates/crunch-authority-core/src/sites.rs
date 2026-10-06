use alloc::string::String;

use crate::FilterChain;
use crate::InvalidCaveat;
use crate::MAX_VALUE_BYTES;
use crate::Pattern;
use crate::bindings_for;
use crate::parse_pattern;
use crate::segment_name_is_valid;
use crate::text_is_valid;

pub const MAX_DECLARED_JOBS: usize = 32;

/// A receiver-provided observation, not proof of UCAN authentication or a
/// substitute for its existing ticket/session policy.
pub struct RemoteJobScope<'a> {
    pub job_ids: &'a [&'a str],
    pub output_class: &'a str,
    pub deadline_unix_s: u64,
}

impl RemoteJobScope<'_> {
    pub fn admits(&self, chain: &FilterChain, job_id: &str, output_class: &str, now_unix_s: u64) -> bool {
        if !declared_jobs_are_valid(self.job_ids) || now_unix_s >= self.deadline_unix_s {
            return false;
        }
        if !segment_name_is_valid(job_id) || !segment_name_is_valid(output_class) || output_class != self.output_class {
            return false;
        }
        if !self.job_ids.contains(&job_id) {
            return false;
        }
        let request = alloc::format!("/job/{job_id}/output/{output_class}");
        if request.len() > MAX_VALUE_BYTES {
            return false;
        }
        chain.admit_with_policy(&request, |value| value == request).is_some()
    }
}

/// This wrapper can only decide about normalized logical paths. The actual
/// store receiver must gate *every* metadata/content read on this decision.
pub struct StoreView {
    prefix: String,
    pattern: Pattern,
    chain: FilterChain,
}

impl StoreView {
    pub fn new(prefix: &str, path_pattern: &str, chain: FilterChain) -> Result<Self, InvalidCaveat> {
        if !text_is_valid(prefix) {
            return Err(InvalidCaveat::Malformed);
        }
        let prefix_with_separator = alloc::format!("{prefix}/");
        if !path_pattern.starts_with(&prefix_with_separator) {
            return Err(InvalidCaveat::Malformed);
        }
        Ok(Self {
            prefix: prefix_with_separator,
            pattern: parse_pattern(path_pattern, true)?,
            chain,
        })
    }

    /// None exposes neither unauthorized paths nor a lookup reason.
    pub fn admits(&self, logical_path: &str) -> bool {
        if !logical_path.starts_with(&self.prefix) || bindings_for(&self.pattern, logical_path).is_none() {
            return false;
        }
        self.chain
            .admit_with_policy(logical_path, |value| {
                value.starts_with(&self.prefix) && bindings_for(&self.pattern, value).is_some()
            })
            .is_some()
    }
}

/// Pure project-goal projection, not a signed grant or a live CLI gate.
pub struct ProjectGoals {
    project: String,
    declaration_prefix: String,
    goal_prefix: String,
    chain: FilterChain,
}

impl ProjectGoals {
    pub fn new(project: &str, chain: FilterChain) -> Result<Self, InvalidCaveat> {
        if !segment_name_is_valid(project) || project.len() > 64 {
            return Err(InvalidCaveat::Malformed);
        }
        let declaration_prefix = alloc::format!("/project/{project}/declared/");
        let goal_prefix = alloc::format!("/project/{project}/goal/");
        Ok(Self {
            project: String::from(project),
            declaration_prefix,
            goal_prefix,
            chain,
        })
    }

    /// Lower a declaration into a project-namespaced goal. A supplied project
    /// name is never accepted as a replacement for the grant's own project.
    pub fn admit_goal(&self, requested_project: &str, goal: &str) -> Option<String> {
        if requested_project != self.project || !segment_name_is_valid(goal) || goal.len() > 128 {
            return None;
        }
        let input = alloc::format!("{}{}", self.declaration_prefix, goal);
        let output = self.chain.admit_with_policy(&input, |value| {
            let name = value.strip_prefix(&self.declaration_prefix).or_else(|| value.strip_prefix(&self.goal_prefix));
            name.is_some_and(|name| segment_name_is_valid(name) && name.len() <= 128)
        })?;
        let goal_name = output.strip_prefix(&self.goal_prefix)?;
        if !segment_name_is_valid(goal_name) || goal_name.len() > 128 {
            return None;
        }
        Some(output.into_owned())
    }
}

/// The effective caveat set is bounded before allocation at the decoder.
/// This bounds the job list handed to the receiver; duplicate IDs deny.
pub fn declared_jobs_are_valid(jobs: &[&str]) -> bool {
    if jobs.is_empty() || jobs.len() > MAX_DECLARED_JOBS {
        return false;
    }
    for (index, job) in jobs.iter().enumerate() {
        if !segment_name_is_valid(job) || job.len() > 128 || jobs[..index].contains(job) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Caveat;

    fn rewrite(pattern: &str, template: &str) -> Caveat {
        Caveat::Rewrite {
            pattern: String::from(pattern),
            template: String::from(template),
        }
    }

    #[test]
    fn declared_job_and_output_deadline_are_checked_locally() {
        let jobs = ["compile-1"];
        let scope = RemoteJobScope {
            job_ids: &jobs,
            output_class: "public",
            deadline_unix_s: 50,
        };
        let grant =
            FilterChain::empty().attenuate(rewrite("/job/:id/output/public", "/job/:id/output/public")).unwrap();
        assert!(scope.admits(&grant, "compile-1", "public", 49));
        assert!(!scope.admits(&grant, "compile-2", "public", 49));
        assert!(!scope.admits(&grant, "compile-1", "private", 49));
        assert!(!scope.admits(&grant, "compile-1", "public", 50));
        assert!(!scope.admits(&grant.attenuate(Caveat::Unknown).unwrap(), "compile-1", "public", 49));
        assert!(!declared_jobs_are_valid(&["compile-1", "compile-1"]));
        assert!(!declared_jobs_are_valid(&["."]));
    }

    #[test]
    fn path_view_refuses_escape_without_returning_the_path() {
        let chain = FilterChain::empty().attenuate(rewrite("/mantle/store/:path", "/mantle/store/:path")).unwrap();
        let view = StoreView::new("/mantle/store", "/mantle/store/a-allowed", chain).unwrap();
        assert!(view.admits("/mantle/store/a-allowed"));
        assert!(!view.admits("/mantle/store/b-private"));
        assert!(!view.admits("/mantle/store/a-allowed/../b-private"));
        assert!(!view.admits("/mantle/storeX/a-allowed"));
        assert!(!view.admits("/mantle/store/a-allowed/private"));
        let unknown = StoreView::new(
            "/mantle/store",
            "/mantle/store/a-allowed",
            FilterChain::empty().attenuate(Caveat::Unknown).unwrap(),
        )
        .unwrap();
        assert!(!unknown.admits("/mantle/store/a-allowed"));
        assert!(StoreView::new("/mantle/store", "/other/store/:path", FilterChain::empty()).is_err());
        assert!(!view.admits("/mantle/store/a-allowed%2fprivate"));
    }

    #[test]
    fn project_namespaced_goals_cannot_cross_project() {
        let chain = FilterChain::empty()
            .attenuate(rewrite("/project/alpha/declared/:goal", "/project/alpha/goal/:goal"))
            .unwrap();
        let scope = ProjectGoals::new("alpha", chain).unwrap();
        assert_eq!(scope.admit_goal("alpha", "build").as_deref(), Some("/project/alpha/goal/build"));
        assert_eq!(scope.admit_goal("beta", "build"), None);
        assert_eq!(scope.admit_goal("alpha", "../secret"), None);
        assert!(ProjectGoals::new("..", FilterChain::empty()).is_err());
        let escape = ProjectGoals::new(
            "alpha",
            FilterChain::empty()
                .attenuate(rewrite("/project/alpha/declared/:goal", "/project/beta/goal/:goal"))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(escape.admit_goal("alpha", "build"), None);
        let nested = ProjectGoals::new(
            "alpha",
            FilterChain::empty()
                .attenuate(rewrite("/project/alpha/declared/:goal", "/project/alpha/goal/unrequested/:goal"))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(nested.admit_goal("alpha", "build"), None);
    }
}
