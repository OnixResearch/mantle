;; Compare the repaired planning receipt with the preceding blocked receipt.
;; Run with the Pi Steel prelude. Arguments: repaired JSON, preceding JSON.
(define (read-json path)
  (pi-json-read (read-port-to-string (open-input-file path))))
(define (field object name)
  (hash-ref object (if (hash-contains? object name) name (string->symbol name))))
(define (contains-field? object name)
  (or (hash-contains? object name) (hash-contains? object (string->symbol name))))
(define expected-artifact-variants-count 2)
(define planning-stages
  '("native_registry_source_planning" "native_git_source_planning"
    "native_package_target_planning" "native_unit_graph_planning"
    "native_host_unit_graph_planning" "unit_derivation_graph"))
(define (stage-check receipt stage)
  (let ([summary (field receipt stage)])
    (list stage (and (field summary "ready") (null? (field summary "blockers"))))))
(define (git-sources receipt)
  (field (field receipt "native_git_source_planning") "sources"))
(define (artifact-variants-distinct? receipt)
  (let ([sources (filter (lambda (source) (equal? (field source "name") "artifact-auth-core"))
                         (git-sources receipt))])
    (and (= (length sources) expected-artifact-variants-count)
         (not (equal? (field (car sources) "resolved_revision")
                      (field (list-ref sources 1) "resolved_revision")))
         (not (equal? (field (car sources) "manifest_path")
                      (field (list-ref sources 1) "manifest_path")))
         (not (equal? (field (car sources) "source_digest")
                      (field (list-ref sources 1) "source_digest"))))))
(define (checks receipt reference)
  (let ([graph (field receipt "unit_derivation_graph")])
    (append
      (map (lambda (stage) (stage-check receipt stage)) planning-stages)
      (list
        (list "cargo_blockers_absent" (null? (field (field receipt "cargo_mode") "blockers")))
        (list "cargo_oracle_disabled" (field (field receipt "cargo_mode") "no_cargo_oracle"))
        (list "nonempty_graph" (> (field graph "derivation_count") 0))
        (list "graph_count_matches" (= (field graph "derivation_count") (length (field graph "derivations"))))
        (list "artifact_variants_distinct" (artifact-variants-distinct? receipt))
        (list "git_source_facts_unchanged" (equal? (git-sources receipt) (git-sources reference)))
        (list "lockfile_facts_unchanged" (equal? (field receipt "lockfile") (field reference "lockfile")))
        (list "source_closure_unchanged" (equal? (field receipt "source_closure") (field reference "source_closure")))
        (list "topology_execution_absent" (not (contains-field? receipt "topology_execution")))))))
(define (failures values)
  (map car (filter (lambda (value) (not (list-ref value 1))) values)))
(define repaired (read-json (pi-arg-ref 0)))
(define preceding (read-json (pi-arg-ref 1)))
(define positive-checks (checks repaired preceding))
(define positive-failures (failures positive-checks))
(define negative-failures (failures (checks preceding preceding)))
(println (pi-json-write (hash
  "positive_checks" positive-checks
  "positive_failures" positive-failures
  "negative_control_rejections" negative-failures
  "derivation_count" (field (field repaired "unit_derivation_graph") "derivation_count"))))
(unless (null? positive-failures) (error "repaired receipt failed validation"))
(when (null? negative-failures) (error "blocked control did not fail validation"))
(println "planning replay verification: PASS")
