(struct Console ((s string) (toggle bool)))

(define state-init
  (new Console :s "" :toggle false)
)

(define (tick state) 
  (do
    state
  )
)

(define (backspace state)
  (maptext init state) ; Init returns all elems in list bar last
)

(define (addchar state c)
  (maptext 
    (lambda (cur) (concat cur (chr c))) 
  state)
)

(define (maptext f state)
  (let ((init (. state :s)))
    (.= state :s (f init))
  )
)


(define (toggle-console state)
  (let (
    (init-toggle (. state :toggle))
    (toggle (not init-toggle))
    (init-text (. state :s))
    (text (if toggle init-text ""))
    )
    (.= state :toggle toggle :s text)
  )
)



(define (press state key)
  (let (
    (toggled (. state :toggle))
    (statenew 
      (cond
        ((and toggled (= key 8))               (backspace state))
        ((= key 9)                             (toggle-console state))
        ;((and toggled (> key 96) (< key 123))  (addchar state key))
        (toggled  (addchar state key))
        (else                                  state)
      )
  ))
    (do
      (println "Current buffer ~a" (. statenew :s))
      (println "~a" key)
      statenew
    )
  )
)

(define (draw state)
  (do
    (if (. state :toggle) 
      (draw-set-color 1.0 0.0 0.0)
      (draw-set-color 0.0 1.0 0.0)
    )
    ;(println "~a" view-xview)
    (draw-set-alpha 0.5)
    (draw-rectangle 130.0 300.0 40.5 100.5 false)
    (draw-set-alpha 1.0)
    (draw-rectangle 1000.0 200.0 400.5 400.5 false)
    (println "~a" view-yview)
    (draw-set-color 0.2 0.8 1.0)
    (draw-text (+ view-xview 100) (+ view-yview 200) "Hi Simon")
    (draw-rectangle 230.0 300.0 40.5 100.5 false)
    (draw-rectangle 730.0 300.0 40.5 100.5 false)
  )
)
