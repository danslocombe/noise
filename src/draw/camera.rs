use game::{Height, Pos, Width, fphys};
use graphics::Viewport;
use world::World;
use tools::weight;
use draw::Rectangle;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::f64::EPSILON;

#[derive(PartialEq, Eq)]
pub struct CameraId(usize);

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct CameraPriority(u32);

#[derive(Copy, Clone)]
pub struct ViewTransform {
    pub x: fphys,
    pub y: fphys,
    pub scale: fphys,
}

pub trait Camera {
    fn transform(&self, &Viewport) -> ViewTransform;
    fn lerp_pos_weight(&self) -> fphys;
    fn lerp_scale_weight(&self) -> fphys;
    fn update(&mut self, &World);
    fn priority(&self) -> CameraPriority;
}

pub struct IdCamera(CameraId, Box<Camera>);

impl PartialEq for IdCamera {
    fn eq(&self, other: &IdCamera) -> bool {
        self.0 == other.0 &&
        self.1.priority().eq(&other.1.priority())
    }
}

impl Eq for IdCamera {}

impl Ord for IdCamera {
    fn cmp(&self, other: &IdCamera) -> Ordering {
        self.1.priority().cmp(&other.1.priority())
    }
}

impl PartialOrd for IdCamera {
    fn partial_cmp(&self, other: &IdCamera) -> Option<Ordering> {
        Some(self.1.priority().cmp(&other.1.priority()))
    }
}

pub struct Editor {
    cameras : BinaryHeap<IdCamera>,
    transform : ViewTransform,
}


impl Editor {
    pub fn new(cameras : Vec<Box<Camera>>) -> Self {
        let id_cameras = cameras.into_iter().enumerate()
            .map(|(i, c)| IdCamera(CameraId(i), c)).collect();
        Editor {
            cameras : id_cameras,
            transform : ViewTransform {x : 0.0, y : 0.0, scale : 1.0},
        }
    }
    pub fn add_camera(&mut self, camera : Box<Camera>) -> CameraId {
        CameraId(0)
    }
    pub fn remove_camera(&mut self, id : CameraId) {
    }
    pub fn transform(&self) -> ViewTransform {
        self.transform
    }
    pub fn update(&mut self, viewport : &Viewport, world : &World) {

        //  Only update topmost camera
        //  ? is this what we want ?
        if let Some(IdCamera(id, mut camera)) = self.cameras.pop() {

            camera.update(world);

            let w_pos = camera.lerp_pos_weight();
            let w_scale = camera.lerp_scale_weight();
            let t = camera.transform(viewport);

            //  Lerp Transforms
            let new_x = weight(self.transform.x, t.x, w_pos);
            let new_y = weight(self.transform.y, t.y, w_pos);
            let new_scale = weight(self.transform.scale, t.scale, w_scale);

            self.transform = ViewTransform {
                x : new_x,
                y : new_y,
                scale : new_scale,
            };

            //  Return to cameras queue
            self.cameras.push(IdCamera(id, camera));
        }

    }
}

pub struct ViewFollower {
    follow_id: u32,
    priority : CameraPriority,

    x_offset: fphys,
    y_offset: fphys,
    scale: fphys,

    w: fphys,
    w_scale: fphys,

    offset_factor: fphys,
    scale_mult: fphys,

    follow_prev_x: fphys,
    follow_prev_y: fphys,
}

impl ViewFollower {
    pub fn new_defaults(vt: ViewTransform, id: u32) -> Self {
        ViewFollower {
            priority: CameraPriority(5),
            x_offset: 0.0,
            y_offset: 0.0,
            scale: 1.0,
            follow_id: id,
            w: 20.0,
            w_scale: 200.0,
            offset_factor: 10.0,
            scale_mult: 0.035,
            follow_prev_x: 0.0,
            follow_prev_y: 0.0,
        }
    }
}
impl Camera for ViewFollower {
    fn priority(&self) -> CameraPriority {
        self.priority
    }
    fn transform(&self, viewport: &Viewport) -> ViewTransform {
        let view_rect = viewport.rect;
        ViewTransform {
            x: self.x_offset - view_rect[2] as f64 / 2.0,
            y: self.y_offset - view_rect[3] as f64 / 2.0,
            scale: self.scale,
        }
    }
    fn lerp_pos_weight(&self) -> fphys {
        self.w
    }
    fn lerp_scale_weight(&self) -> fphys {
        self.w_scale
    }
    fn update(&mut self, world: &World) {
        world.get(self.follow_id).map(|(_, bb)| {

            let Pos(bbx, bby) = bb.pos;

            let bb_xvel = bbx - self.follow_prev_x;
            let bb_yvel = bby - self.follow_prev_y;
            let speed = (bb_xvel.powi(2) + bb_yvel.powi(2)).sqrt();

            let x_offset = bb_xvel * self.offset_factor;
            let y_offset = bb_yvel * self.offset_factor;

            self.x_offset = bbx + x_offset;
            self.y_offset = bby + y_offset;

            self.scale = 0.8 - speed * self.scale_mult;

            self.follow_prev_x = bbx;
            self.follow_prev_y = bby;
        });
    }
}

impl ViewTransform {
    pub fn to_rectangle(&self,
                        screen_width: fphys,
                        screen_height: fphys)
                        -> Rectangle {
        Rectangle::new(self.x - screen_width / 2.0,
                       self.y - screen_height / 2.0,
                       screen_width / self.scale,
                       screen_height / self.scale)
    }
}

