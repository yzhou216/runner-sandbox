;;; SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
;;;
;;; SPDX-License-Identifier: AGPL-3.0-or-later

;; Hazards recorded by define-hazard, in order of appearance, each a list of
;; its definition as data and the procedure that computes it
(define %hazards '())

;; Aliases of the procedures that the macros below expand into, which a
;; binding where a macro is used, such as an input named list, cannot shadow
(define %append append)
(define %cons cons)
(define %list list)

;; Quasiquote for the arguments of a macro.  Steel's quasiquote leaves an
;; unquote that reaches it through a macro unevaluated, so this one evaluates
;; unquote and unquote-splicing itself.
(define-syntax %template
  (syntax-rules (unquote unquote-splicing)
    [(_ (unquote expression)) expression]
    [(_ ((unquote-splicing expression) rest ...))
     (%append expression (%template (rest ...)))]
    [(_ (first rest ...)) (%cons (%template first) (%template (rest ...)))]
    [(_ atom) 'atom]))

;; Define a hazard:
;;
;;   (define-hazard (name input ...)
;;     declaration ...
;;     body ...)
;;
;; The signature and the declarations are recorded as data.  The body becomes
;; a procedure with one parameter per input, called each time the hazard is
;; computed.
(define-syntax define-hazard
  (syntax-rules ()
    [(_ (name input ...) clause ...)
     (%define-hazard (name input ...) () clause ...)]))

;; Move the leading declarations into a list, then record the hazard with the
;; remaining clauses as its body.
(define-syntax %define-hazard
  (syntax-rules (type description nodata)
    [(_ signature (declaration ...) (type value) clause ...)
     (%define-hazard signature (declaration ... (type value)) clause ...)]
    [(_ signature (declaration ...) (description value) clause ...)
     (%define-hazard signature (declaration ... (description value)) clause ...)]
    [(_ signature (declaration ...) (nodata value) clause ...)
     (%define-hazard signature (declaration ... (nodata value)) clause ...)]
    [(_ (name input ...) (declaration ...))
     (error "hazard has no body:" 'name)]
    [(_ (name input ...) (declaration ...) body ...)
     (set! %hazards
           (%append %hazards
                    (%list (%list (%template ((name input ...) declaration ...))
                                  (%procedure () (input ...) body ...)))))]))

;; Procedure whose parameters are the names of the inputs.  The names are
;; collected one input at a time, since an input may also be written
;; (name kind), (optional name), or (optional name kind).
(define-syntax %procedure
  (syntax-rules (optional)
    [(_ (parameter ...) () body ...) (lambda (parameter ...) body ...)]
    [(_ (parameter ...) ((optional name kind) input ...) body ...)
     (%procedure (parameter ... name) (input ...) body ...)]
    [(_ (parameter ...) ((optional name) input ...) body ...)
     (%procedure (parameter ... name) (input ...) body ...)]
    [(_ (parameter ...) ((name kind) input ...) body ...)
     (%procedure (parameter ... name) (input ...) body ...)]
    [(_ (parameter ...) (name input ...) body ...)
     (%procedure (parameter ... name) (input ...) body ...)]))

;; Raster computed at every pixel by an expression over the names bound to
;; rasters, numbers, or #f for a value missing everywhere:
;;
;;   (pixelwise (name ...) expression)
;;
;; Only the unquoted parts of the expression are evaluated as Scheme, once,
;; when pixelwise runs.
(define-syntax pixelwise
  (syntax-rules ()
    [(_ (name ...) expression)
     (%pixelwise (%list name ...) '(name ...) (%template expression))]))
