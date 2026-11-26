;;; SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
;;;
;;; SPDX-License-Identifier: 0BSD OR CC0-1.0

;;; Directory Local Variables
;;; For more information, see (info "(emacs) Directory Variables")

;; Formatting lives in .editorconfig.  Do not duplicate it here, since
;; directory locals take precedence and the two would drift.

((nil . ((mode . editorconfig)
         (eglot-workspace-configuration
          . (:rust-analyzer (:check (:command "clippy")))))))
