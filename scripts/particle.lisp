(use random (random))
(use math (cos sin))

(struct Part ((x float) (y float) (t float)))

(define (state-init)
  (let 
    ((pids (get-ids "player")))
    (if (null pids)
      (do
        (println "No player, destroying")
        (destroy me)
      )
      (let (
           (x (. (get (first pids)) :x) )
           (y (. (get (first pids)) :y) )
         )
         (do
          ;(println "MAKING")
          (new Part :x x :y y :t 100.0)
         )
      )
    )
  )
)

(const VEL 4.0)
(const GRAV 0.2)

(define (tick state) 
  (if (null state)
    (state-init)
    (let
      (
       (x0 (. state :x))
       (y0 (. state :y))
       (t0 (. state :t))
       (r-dir (* 6.2 (random)))
       (x1 (+ x0 (* VEL (cos r-dir))))
       (y1 (+ y0 (* VEL (sin r-dir))))
       (y2 (+ y0 GRAV))
       (t1 (- t0 1.0))
      )
      (do
        (if (> 0.015 (random))
          (do
            (destroy me)
            state
          )
          ;(.= state :x x1 :y y2 :t t1)
          (.= state :x x1 :y y2 :t t1)
        )
      )
    )
  )
)

(define (draw state)
  (do
    (draw-set-color 1.0 1.0 0.0)
    (draw-rectangle (. state :x) (. state :y) 20.0 20.0 false)
    ()
  )
)
