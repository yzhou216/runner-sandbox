;;; SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
;;;
;;; SPDX-License-Identifier: AGPL-3.0-or-later

;; Earthquake hazard class from peak ground acceleration (PGA).  Class 0 is
;; where there is neither PGA nor land, 1 is land without PGA data, and 2
;; through 5 rank PGA from very low to high.  Land is wherever the optional
;; land mask has data.

(define-hazard (earthquake pga (optional land))
  (type u8)
  (description "earthquake_class")
  (pixelwise (pga land)
    (cond ((missing? pga) (if (missing? land) 0 1))
          ((< pga 50) 2)
          ((< pga 150) 3)
          ((< pga 300) 4)
          (else 5))))
