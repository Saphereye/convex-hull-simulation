use rand::Rng;
use std::collections::HashSet;
use std::hash::{Hasher, Hash};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
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

impl Hash for Vec2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Convert the floating point numbers to integers with a fixed number of decimal places
        let x = (self.x * 1_000_000.0) as i64;
        let y = (self.y * 1_000_000.0) as i64;

        // Hash the integers
        x.hash(state);
        y.hash(state);
    }
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

    let upper_hull_max = *upper_hull_vec
        .iter()
        .max_by(|a, b| a.x.partial_cmp(&b.x).unwrap())
        .unwrap();
    let upper_hull_min = *upper_hull_vec
        .iter()
        .min_by(|a, b| a.x.partial_cmp(&b.x).unwrap())
        .unwrap();
    let lower_hull_max = *lower_hull_vec
        .iter()
        .max_by(|a, b| a.x.partial_cmp(&b.x).unwrap())
        .unwrap();
    let lower_hull_min = *lower_hull_vec
        .iter()
        .min_by(|a, b| a.x.partial_cmp(&b.x).unwrap())
        .unwrap();

    if upper_hull_max.x == lower_hull_max.x && upper_hull_min.y != lower_hull_min.y {
        upper_hull_vec.push(lower_hull_max);
    }

    if upper_hull_min.x == lower_hull_min.x && upper_hull_min.y != lower_hull_min.y {
        upper_hull_vec.push(lower_hull_min);
    }

    upper_hull_vec.extend(lower_hull_vec);
    upper_hull_vec
}

fn upper_hull(points: &[Vec2]) -> Vec<Vec2> {
    let mut min_point = Vec2 {
        x: f32::MAX,
        y: f32::MIN,
    };
    for i in points.iter() {
        if i.x < min_point.x || (i.x == min_point.x && i.y > min_point.y) {
            min_point = *i;
        }
    }

    let mut max_point = Vec2 {
        x: f32::MIN,
        y: f32::MAX,
    };
    for i in points.iter() {
        if i.x > max_point.x || (i.x == max_point.x && i.y > max_point.y) {
            max_point = *i;
        }
    }

    if min_point == max_point {
        return vec![min_point];
    }

    let mut temporary = vec![min_point, max_point];
    temporary.extend(
        points
            .iter()
            .filter(|p| p.x > min_point.x && p.x < max_point.x),
    );

    connect(&min_point, &max_point, &temporary)
}

fn connect(min: &Vec2, max: &Vec2, points: &[Vec2]) -> Vec<Vec2> {
    let median = median_of_medians(&points.iter().map(|point| point.x).collect::<Vec<_>>());

    let (left, right) = bridge(points, median);

    let mut left_points = vec![left];
    left_points.extend(points.iter().filter(|p| p.x < left.x));

    let mut right_points = vec![right];
    right_points.extend(points.iter().filter(|p| p.x > right.x));

    let mut output = vec![];
    if left == *min {
        output.extend(vec![left]);
    } else {
        output.extend(connect(&min, &left, &left_points));
    }

    if right == *max {
        output.extend(vec![right]);
    } else {
        output.extend(connect(&right, &max, &right_points));
    }

    output
}

fn bridge(points: &[Vec2], median: f32) -> (Vec2, Vec2) {
    let mut candidates: HashSet<&Vec2> = HashSet::new();
    if points.len() == 2 {
        return if points[0].x < points[1].x {
            (points[0], points[1])
        } else {
            (points[1], points[0])
        };
    }

    let mut pairs: Vec<(&Vec2, &Vec2)> = Vec::new();

    for chunk in points.chunks(2) {
        if chunk.len() == 2 {
            if chunk[0] > chunk[1] {
                pairs.push((&chunk[1], &chunk[0]));
            } else {
                pairs.push((&chunk[0], &chunk[1]));
            }
        } else {
            candidates.insert(&chunk[0]);
        }
    }

    let mut slopes = vec![];

    for (point_i, point_j) in pairs.iter() {
        if point_i.x == point_j.x {
            if point_i.y > point_j.y {
                candidates.insert(point_i);
            } else {
                candidates.insert(point_j);
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
        median_of_medians(&slopes.iter().map(|(_, _, slope)| slope).collect::<Vec<_>>());

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

    // set of points with maximum value of p.y - median_slope * p.x
    let max_value = points
        .iter()
        .map(|p| p.y - median_slope * p.x)
        .fold(f32::MIN, f32::max);
    let max_points: Vec<_> = points
        .iter()
        .filter(|p| ((p.y - median_slope * p.x) - max_value).abs() < 0.01)
        .collect();
    let min_point = max_points
        .iter()
        .min_by(|a, b| a.x.partial_cmp(&b.x).unwrap())
        .unwrap();
    let max_point = max_points
        .iter()
        .max_by(|a, b| a.x.partial_cmp(&b.x).unwrap())
        .unwrap();

    if min_point.x <= median && max_point.x > median {
        return (**min_point, **max_point);
    } else if max_point.x <= median {
        for (_, point2, _) in large {
            candidates.insert(point2);
        }

        for (_, point2, _) in equal {
            candidates.insert(point2);
        }

        for (point1, point2, _) in small {
            candidates.insert(point2);
            candidates.insert(point1);
        }
    } else if min_point.x > median {
        for (point1, _, _) in small {
            candidates.insert(point1);
        }

        for (point1, _, _) in equal {
            candidates.insert(point1);
        }

        for (point1, point2, _) in large {
            candidates.insert(point2);
            candidates.insert(point1);
        }
    }

    bridge(&candidates.into_iter().cloned().collect::<Vec<Vec2>>(), median)
}

pub fn median_of_medians<T>(nums: &[T]) -> T
where
    T: Clone + Copy + PartialOrd,
{
    match nums.len() {
        0 => panic!("No median of an empty list"),
        1 => nums[0],
        2..=5 => {
            let mut nums = nums.to_owned();
            nums.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
            nums[nums.len() / 2]
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

fn main() {
    let n = 100000; // replace with the number of points you want
    let mut rng = rand::thread_rng();
    let points: Vec<Vec2> = (0..n).map(|_| Vec2::new(rng.gen(), rng.gen())).collect();

    kirk_patrick_seidel(&points);
}
