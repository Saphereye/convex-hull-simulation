use rand::Rng;

#[derive(Debug, Clone, Copy)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

fn jarvis_march(points: Vec<Vec2>) -> Vec<Vec2> {
    let n = points.len();
    if n < 3 {
        return Vec::new();
    }

    let mut hull = Vec::new();

    let mut l = 0;
    for i in 1..n {
        if points[i].x < points[l].x {
            l = i;
        }
    }

    let mut p = l;
    let mut q;
    loop {
        hull.push(points[p]);

        q = (p + 1) % n;
        for r in 0..n {
            if let Orientation::Counterclockwise = orientation(&points[p], &points[r], &points[q]) {
                q = r;
            }
        }

        p = q;

        if p == l {
            break;
        }
    }

    hull
}

enum Orientation {
    Clockwise,
    Counterclockwise,
    Colinear,
}

#[inline(always)]
fn orientation(p: &Vec2, q: &Vec2, r: &Vec2) -> Orientation {
    let val = (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y);
    if val == 0.0 {
        return Orientation::Colinear;
    }
    if val > 0.0 {
        return Orientation::Clockwise;
    }
    Orientation::Counterclockwise
}

fn main() {
    let n = 100000; // replace with the number of points you want
    let mut rng = rand::thread_rng();
    let points: Vec<Vec2> = (0..n).map(|_| Vec2::new(rng.gen(), rng.gen())).collect();

    jarvis_march(points);
}
