#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use criterion::{criterion_group, criterion_main, Criterion};

#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Eq for Vec2 {}

fn jarvis_march(points: Vec<Vec2>) -> Vec<Vec2> {
    let n = points.len();
    let mut hull = Vec::with_capacity(n);

    if n < 3 {
        return hull;
    }

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

fn kirk_patrick_seidel(points: &[Vec2]) -> Vec<Vec2> {
    let mut upper_hull_vec = upper_hull(&points);

    let mut lower_hull_vec = upper_hull(
        &points
            .iter()
            .map(|point| Vec2 {
                x: point.x,
                y: -point.y,
            })
            .collect::<Vec<_>>(),
    );

    lower_hull_vec = lower_hull_vec
        .iter()
        .map(|point| Vec2 {
            x: point.x,
            y: -point.y,
        })
        .collect();

    let (upper_hull_min, upper_hull_max) = upper_hull_vec.iter().fold(
        (upper_hull_vec[0], upper_hull_vec[0]),
        |(min, max), &point| {
            (
                if point.x < min.x { point } else { min },
                if point.x > max.x { point } else { max },
            )
        },
    );

    let (lower_hull_min, lower_hull_max) = lower_hull_vec.iter().fold(
        (lower_hull_vec[0], lower_hull_vec[0]),
        |(min, max), &point| {
            (
                if point.x < min.x { point } else { min },
                if point.x > max.x { point } else { max },
            )
        },
    );

    if upper_hull_max.x == lower_hull_max.x && upper_hull_min.y != lower_hull_min.y {
        upper_hull_vec.push(lower_hull_max);
    }

    if upper_hull_min.x == lower_hull_min.x && upper_hull_min.y != lower_hull_min.y {
        upper_hull_vec.push(lower_hull_min);
    }

    upper_hull_vec.extend(lower_hull_vec);
    upper_hull_vec
}

#[inline]
fn upper_hull(points: &[Vec2]) -> Vec<Vec2> {
    let mut min_point = Vec2 {
        x: f32::MAX,
        y: f32::MIN,
    };
    let mut max_point = Vec2 {
        x: f32::MIN,
        y: f32::MAX,
    };

    for i in points.iter() {
        if i.x < min_point.x || (i.x == min_point.x && i.y > min_point.y) {
            min_point = *i;
        }
        if i.x > max_point.x || (i.x == max_point.x && i.y < max_point.y) {
            max_point = *i;
        }
    }

    let mut temporary = Vec::with_capacity(points.len());
    temporary.push(min_point);

    if min_point == max_point {
        return temporary;
    }

    temporary.push(max_point);
    temporary.extend(
        points
            .iter()
            .filter(|p| p.x > min_point.x && p.x < max_point.x),
    );

    connect(&min_point, &max_point, &temporary)
}

#[inline]
fn connect(min: &Vec2, max: &Vec2, points: &[Vec2]) -> Vec<Vec2> {
    let median = median_of_medians(&points.iter().map(|point| point.x).collect::<Vec<_>>()).unwrap_or(0.0);

    let (left, right) = bridge(points, median);

    let mut left_points = Vec::with_capacity(points.len());
    left_points.push(left);
    left_points.extend(points.iter().filter(|p| p.x < left.x));

    let mut right_points = Vec::with_capacity(points.len());
    right_points.push(right);
    right_points.extend(points.iter().filter(|p| p.x > right.x));

    let mut output = Vec::with_capacity(points.len());
    if left == *min {
        output.push(left);
    } else {
        output.extend(connect(&min, &left, &left_points));
    }

    if right == *max {
        output.push(right);
    } else {
        output.extend(connect(&right, &max, &right_points));
    }

    output
}

#[inline]
fn bridge(points: &[Vec2], median: f32) -> (Vec2, Vec2) {
    let mut candidates: Vec<Vec2> = Vec::with_capacity(points.len());
    if points.len() == 2 {
        return if points[0].x < points[1].x {
            (points[0], points[1])
        } else {
            (points[1], points[0])
        };
    }

    let mut pairs: Vec<(&Vec2, &Vec2)> = Vec::with_capacity(points.len());

    for chunk in points.chunks(2) {
        if chunk.len() == 2 {
            if chunk[0] > chunk[1] {
                pairs.push((&chunk[1], &chunk[0]));
            } else {
                pairs.push((&chunk[0], &chunk[1]));
            }
        } else {
            candidates.push(chunk[0]);
        }
    }

    let mut slopes = Vec::with_capacity(pairs.len());

    for (point_i, point_j) in pairs.iter() {
        if point_i.x == point_j.x {
            if point_i.y > point_j.y {
                candidates.push(**point_i);
            } else {
                candidates.push(**point_j);
            }
        } else {
            slopes.push((
                point_i,
                point_j,
                (point_i.y - point_j.y) / (point_i.x - point_j.x),
            ));
        }
    }

    let median_slope =
        median_of_medians(&slopes.iter().map(|(_, _, slope)| slope).collect::<Vec<_>>()).unwrap_or(&0.0);

    let mut small = Vec::with_capacity(slopes.len());
    let mut equal = Vec::with_capacity(slopes.len());
    let mut large = Vec::with_capacity(slopes.len());

    for slope in slopes.iter() {
        match slope.2 {
            s if s < *median_slope => small.push(slope),
            s if s == *median_slope => equal.push(slope),
            s if s > *median_slope => large.push(slope),
            _ => unreachable!(),
        }
    }

    let mut max_value = f32::MIN;
    let mut max_points = Vec::with_capacity(points.len());
    let mut min_point = None;
    let mut max_point = None;

    for p in points.iter() {
        let value = p.y - median_slope * p.x;
        if value > max_value {
            max_value = value;
            max_points.clear();
            max_points.push(p);
            min_point = Some(p);
            max_point = Some(p);
        } else if (value - max_value).abs() < f32::EPSILON {
            max_points.push(p);
            if let Some(min_p) = min_point {
                if p.x < min_p.x {
                    min_point = Some(p);
                }
            }
            if let Some(max_p) = max_point {
                if p.x > max_p.x {
                    max_point = Some(p);
                }
            }
        }
    }

    let min_point = min_point.unwrap();
    let max_point = max_point.unwrap();

    if min_point.x <= median && max_point.x > median {
        return (*min_point, *max_point);
    } else if max_point.x <= median {
        for (_, point2, _) in large {
            candidates.push(***point2);
        }

        for (_, point2, _) in equal {
            candidates.push(***point2);
        }

        for (point1, point2, _) in small {
            candidates.push(***point2);
            candidates.push(***point1);
        }
    } else if min_point.x > median {
        for (point1, _, _) in small {
            candidates.push(***point1);
        }

        for (point1, _, _) in equal {
            candidates.push(***point1);
        }

        for (point1, point2, _) in large {
            candidates.push(***point2);
            candidates.push(***point1);
        }
    }

    bridge(&candidates, median)
}

#[inline(always)]
pub fn median_of_medians<T>(nums: &[T]) -> Option<T>
where
    T: Clone + Copy + PartialOrd,
{
    match nums.len() {
        0 => None,
        1 => Some(nums[0]),
        2..=5 => {
            let mut nums = nums.to_owned();
            nums.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
            Some(nums[nums.len() / 2])
        }
        _ => median_of_medians(
            &nums
                .to_vec()
                .chunks(5)
                .map(|chunk| {
                    let mut chunk = chunk.to_vec();
                    chunk.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
                    chunk[chunk.len() / 2]
                })
                .collect::<Vec<T>>(),
        ),
    }
}

pub fn comparison(c: &mut Criterion) {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let seed = [32; 32]; // A seed for the RNG. You can put any number here.
    let mut rng: StdRng = SeedableRng::from_seed(seed);

    let points: Vec<Vec2> = (0..100_00_000)
        .map(|_| {
            Vec2::new(
                rng.gen_range(-50_000..50_000) as f32,
                rng.gen_range(-50_000..50_000) as f32,
            )
        })
        .collect();

    let mut group = c.benchmark_group("Convex-hull Algorithms Comparison");
    group.bench_function("Jarvis March", |b| {
        b.iter(|| kirk_patrick_seidel(&points.clone()))
    });
    group.bench_function("Kirk Patrick Seidel", |b| b.iter(|| jarvis_march(points.clone())));

    group.finish();
}

criterion_group!(benches, comparison);
criterion_main!(benches);
