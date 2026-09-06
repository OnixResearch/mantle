;; Report-only review. This module does not admit payloads or authorize execution.
(provide review-dev-report review-stage-order review-non-claims)

(define digest-hex-length 64)
(define adopted-provider-stage-count 3)
(define lower-hex-characters (string->list "0123456789abcdef"))
(define review-stage-order
  '("stagex-transition" "stagex-provider" "full-source-native-provider"
    "full-source-rust-provider" "mantle-stage1" "mantle-stage2"))
(define review-non-claims
  '("restored stages were not executed again in this attempt"
    "dev resume does not satisfy a promoted fixed-point proof"
    "runtime confirmation is bounded to the recorded source profile, host, plan, and policy"))
(define input-fields
  '(mode plan_digest_blake3 completed_stage selected_bundle_identity_blake3 report))
(define report-fields
  '(schema status mode plan_digest_blake3 selected_bundle_identity_blake3
    published_bundle_identities_blake3 restored_stages executed_stages
    first_incomplete_stage rejected_candidates cache_adoption_disposition
    promoted_receipt_written release_alias_updated non_claims))

(define (all-match? predicate items)
  (if (null? items) #t
      (and (predicate (car items)) (all-match? predicate (cdr items)))))

(define (has-field? object key)
  (and (hash? object)
       (or (hash-contains? object key)
           (hash-contains? object (symbol->string key)))))

(define (field object key)
  (cond [(not (hash? object)) #f]
        [(hash-contains? object key) (hash-ref object key)]
        [(hash-contains? object (symbol->string key))
         (hash-ref object (symbol->string key))]
        [else #f]))

(define (exact-fields? object keys)
  (and (hash? object)
       (= (length (hash-keys->list object)) (length keys))
       (all-match? (lambda (key) (has-field? object key)) keys)))

(define (digest? value)
  (and (string? value)
       (= (string-length value) digest-hex-length)
       (all-match? (lambda (ch) (member ch lower-hex-characters))
                   (string->list value))))

(define (distinct? items)
  (if (null? items) #t
      (and (not (member (car items) (cdr items))) (distinct? (cdr items)))))

(define (prefix-through target items)
  (cond [(null? items) '()]
        [(equal? (car items) target) (list (car items))]
        [else (cons (car items) (prefix-through target (cdr items)))]))

(define (expected-input? input)
  (and (exact-fields? input input-fields)
       (digest? (field input 'plan_digest_blake3))
       (member (field input 'mode) '("cold" "resume" "adopt"))
       (if (equal? (field input 'mode) "resume")
           (and (member (field input 'completed_stage) review-stage-order)
                (digest? (field input 'selected_bundle_identity_blake3)))
           (and (void? (field input 'completed_stage))
                (void? (field input 'selected_bundle_identity_blake3))))))

(define (review-result errors)
  (hash 'accepted (null? errors)
        'scope "dev-report-shape-and-declared-expectations-only"
        'runtime_proven #f
        'errors errors))

(define (review-valid-input input)
  (define mode (field input 'mode))
  (define report (field input 'report))
  (define restored
    (if (equal? mode "resume")
        (prefix-through (field input 'completed_stage) review-stage-order) '()))
  (define executed
    (list-tail review-stage-order
      (if (equal? mode "adopt") adopted-provider-stage-count (length restored))))
  (define published (field report 'published_bundle_identities_blake3))
  (define expected-published-count (if (equal? mode "adopt") 0 (length executed)))
  (define disposition
    (cond [(equal? mode "cold") "cold-executed"]
          [(equal? mode "resume") "resume-restored"]
          [else "provider-cache-adopted"]))
  (define checks
    (list
      (cons "report-fields" (exact-fields? report report-fields))
      (cons "schema" (equal? (field report 'schema) "mantle-dev-stage-resume-report-v1"))
      (cons "dev-only-status" (equal? (field report 'status) "dev-only"))
      (cons "dev-mode" (equal? (field report 'mode) "dev"))
      (cons "plan-identity" (equal? (field report 'plan_digest_blake3)
                                    (field input 'plan_digest_blake3)))
      (cons "selected-bundle" (equal? (field report 'selected_bundle_identity_blake3)
                                      (field input 'selected_bundle_identity_blake3)))
      (cons "restored-prefix" (equal? (field report 'restored_stages) restored))
      (cons "executed-suffix" (equal? (field report 'executed_stages) executed))
      (cons "first-incomplete-stage"
        (if (null? executed) (void? (field report 'first_incomplete_stage))
            (equal? (field report 'first_incomplete_stage) (car executed))))
      (cons "publications"
        (and (list? published) (= (length published) expected-published-count)
             (all-match? digest? published) (distinct? published)))
      (cons "unexpected-rejections" (equal? (field report 'rejected_candidates) '()))
      (cons "adoption-disposition"
        (equal? (field report 'cache_adoption_disposition) disposition))
      (cons "no-promoted-receipt" (equal? (field report 'promoted_receipt_written) #f))
      (cons "no-release-alias" (equal? (field report 'release_alias_updated) #f))
      (cons "non-claims" (equal? (field report 'non_claims) review-non-claims))))
  (review-result (map car (filter (lambda (check) (not (cdr check))) checks))))

(define (review-dev-report input)
  (if (expected-input? input) (review-valid-input input)
      (review-result '("expected-input"))))
