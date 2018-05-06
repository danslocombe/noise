(struct TestState ((x integer) (y float)))

(define (state-init) 
  (new TestState :x (- 1 2) :y 0.0)
)

(define (tick state) 
  (let ((oldx (. state :x)))
    (do
      (println "priawfa ~a" oldx)
      (.= state :x (+ oldx 1))
    )
  )
)
