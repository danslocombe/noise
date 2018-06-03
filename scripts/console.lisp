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
      ;(println "Current buffer ~a" (. statenew :s))
      ;(println "~a" key)
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
    (draw-text 
      (+ view-xview (/ view-wview 2.0))
      (+ view-yview (* view-hview 0.25)) 
      (. state :s)
    )
    (let 
      ((pids (get-ids "player")))
      (if (null pids)
        (println "No player")
        (let (
             (x (. (get (first pids)) :x) )
             (y (. (get (first pids)) :y) )
           )
           (draw-rectangle x y 100.0 100.0 false)
        )
      )
    )
    ()
  )
)
