(require "dev-report-review.scm")

(define fixture-digest-length 64)
(define fixture-plan (make-string fixture-digest-length #\a))
(define fixture-bundle (make-string fixture-digest-length #\b))
(define wrong-digest (make-string fixture-digest-length #\f))
(define json-null (when #f #t))
(define publication-digests
  (map (lambda (ch) (make-string fixture-digest-length ch)) (string->list "abcdef")))
(define stages review-stage-order)

(define (fixture mode completed restored executed publications disposition)
  (define selected (if (equal? mode "resume") fixture-bundle json-null))
  (hash 'mode mode 'plan_digest_blake3 fixture-plan
        'completed_stage completed 'selected_bundle_identity_blake3 selected
        'report
        (hash 'schema "mantle-dev-stage-resume-report-v1" 'status "dev-only" 'mode "dev"
              'plan_digest_blake3 fixture-plan 'selected_bundle_identity_blake3 selected
              'published_bundle_identities_blake3 publications
              'restored_stages restored 'executed_stages executed
              'first_incomplete_stage (if (null? executed) json-null (car executed))
              'rejected_candidates '() 'cache_adoption_disposition disposition
              'promoted_receipt_written #f 'release_alias_updated #f
              'non_claims review-non-claims)))

(define cold (fixture "cold" json-null '() stages publication-digests "cold-executed"))
(define complete (fixture "resume" "mantle-stage2" stages '() '() "resume-restored"))
(define adopted (fixture "adopt" json-null '()
  '("full-source-rust-provider" "mantle-stage1" "mantle-stage2") '() "provider-cache-adopted"))
(define prefixes
  '(("stagex-transition")
    ("stagex-transition" "stagex-provider")
    ("stagex-transition" "stagex-provider" "full-source-native-provider")
    ("stagex-transition" "stagex-provider" "full-source-native-provider" "full-source-rust-provider")
    ("stagex-transition" "stagex-provider" "full-source-native-provider" "full-source-rust-provider" "mantle-stage1")
    ("stagex-transition" "stagex-provider" "full-source-native-provider" "full-source-rust-provider" "mantle-stage1" "mantle-stage2")))
(define positives
  (append (list (cons "cold" cold) (cons "adopt" adopted))
    (map (lambda (prefix)
      (define completed (car (reverse prefix)))
      (cons completed (fixture "resume" completed prefix
        (list-tail stages (length prefix))
        (list-tail publication-digests (length prefix)) "resume-restored"))) prefixes)))

(define (changed input key value)
  (hash-insert input 'report (hash-insert (hash-ref input 'report) key value)))
(define negatives
  (list
    (list "wrong-plan" (changed cold 'plan_digest_blake3 wrong-digest) "plan-identity")
    (list "wrong-selected-bundle" (changed complete 'selected_bundle_identity_blake3 wrong-digest) "selected-bundle")
    (list "stale-schema" (changed cold 'schema "v0") "schema")
    (list "promoted-mode" (changed cold 'mode "promoted") "dev-mode")
    (list "wrong-status" (changed cold 'status "success") "dev-only-status")
    (list "restored-not-executed" (changed complete 'executed_stages stages) "executed-suffix")
    (list "wrong-order" (changed complete 'restored_stages (reverse stages)) "restored-prefix")
    (list "missing-stage" (changed cold 'executed_stages (cdr stages)) "executed-suffix")
    (list "wrong-first-stage" (changed cold 'first_incomplete_stage json-null) "first-incomplete-stage")
    (list "unknown-stage" (changed complete 'restored_stages '("unknown")) "restored-prefix")
    (list "duplicate-publications" (changed cold 'published_bundle_identities_blake3
      (map (lambda (unused) fixture-bundle) publication-digests)) "publications")
    (list "missing-publication" (changed cold 'published_bundle_identities_blake3 (cdr publication-digests)) "publications")
    (list "malformed-publication" (changed cold 'published_bundle_identities_blake3
      (cons "not-a-digest" (cdr publication-digests))) "publications")
    (list "wrong-publication-type" (changed cold 'published_bundle_identities_blake3 "none") "publications")
    (list "unexpected-rejection" (changed cold 'rejected_candidates '("rejected")) "unexpected-rejections")
    (list "wrong-adoption" (changed adopted 'cache_adoption_disposition "cold-executed") "adoption-disposition")
    (list "promoted-receipt" (changed cold 'promoted_receipt_written #t) "no-promoted-receipt")
    (list "alias-write" (changed cold 'release_alias_updated #t) "no-release-alias")
    (list "boolean-type" (changed cold 'release_alias_updated "false") "no-release-alias")
    (list "missing-non-claims" (changed cold 'non_claims '()) "non-claims")
    (list "unknown-field" (changed cold 'unknown #t) "report-fields")
    (list "missing-field" (hash-insert cold 'report
      (hash-remove (hash-ref cold 'report) 'release_alias_updated)) "report-fields")
    (list "missing-report" (hash-remove cold 'report) "expected-input")
    (list "wrong-input-type" '() "expected-input")
    (list "wrong-report-type" (hash-insert cold 'report #f) "report-fields")
    (list "invalid-mode" (hash-insert cold 'mode "promoted") "expected-input")
    (list "invalid-completed-stage" (hash-insert complete 'completed_stage "unknown") "expected-input")
    (list "stage-id-is-not-report-enum" (hash-insert complete 'completed_stage "stagex-provider-publication") "expected-input")
    (list "uppercase-expected-digest" (hash-insert cold 'plan_digest_blake3
      (make-string fixture-digest-length #\A)) "expected-input")))

(for-each (lambda (case)
  (define result (review-dev-report (cdr case)))
  (unless (hash-ref result 'accepted) (error (string-append "positive rejected: " (car case))))
  (when (hash-ref result 'runtime_proven) (error "report review claimed runtime proof"))) positives)
(for-each (lambda (case)
  (define result (review-dev-report (list-ref case 1)))
  (when (hash-ref result 'accepted) (error (string-append "negative accepted: " (car case))))
  (define reason (car (reverse case)))
  (unless (member reason (hash-ref result 'errors))
    (error (string-append "wrong rejection: " (car case))))) negatives)
(display "dev-report-review: PASS positive_cases=")
(display (length positives))
(display " negative_cases=")
(display (length negatives))
(display " runtime_proven=false scope=fixture-only-report-review")
(newline)
