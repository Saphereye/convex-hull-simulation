use std::collections::HashSet;

use bevy::prelude::*;

use crate::TextComment;

#[derive(Resource)]
pub struct DrawingHistory(pub Vec<Vec<LineType>>, pub usize); // history, current

#[derive(Component)]
pub struct Gizmo;

#[derive(Component)]
pub struct ConvexHull;

#[derive(PartialEq, Clone, Copy)]
pub enum AlgorithmType {
    JarvisMarch,
    KirkPatrickSeidel,
}

#[derive(Resource)]
pub struct Algorithm(pub AlgorithmType);

pub enum LineType {
    PartOfHull(Vec2, Vec2),
    Temporary(Vec2, Vec2),
    TextComment(String),
    VerticalLine(f32), // Draws a vertical line at x
    ClearScreen,
}

/// # Implementation of the [Jarvis March](https://en.wikipedia.org/wiki/Gift_wrapping_algorithm) algorithm (Gift-Wrapping algorithm)
/// This algorithm is used to calculate the convex hull of given set of points.
/// It has a `O(nh)` time complexity, where `n` is the number of points and `h` is the number of points on the convex hull.
/// <p><a href="https://commons.wikimedia.org/wiki/File:Animation_depicting_the_gift_wrapping_algorithm.gif#/media/File:Animation_depicting_the_gift_wrapping_algorithm.gif"><img src="https://upload.wikimedia.org/wikipedia/commons/9/9c/Animation_depicting_the_gift_wrapping_algorithm.gif" alt="Animation depicting the gift wrapping algorithm.gif" height="401" width="401"></a><br></p>
///
/// ## Pseudocode
/// ```pseudocode
/// algorithm jarvis(S) is
///     # S is the set of points
///     # P will be the set of points which form the convex hull. Final set size is i.
///     pointOnHull = leftmost point in S // which is guaranteed to be part of the CH(S)
///     i := 0
///     repeat
///         P[i] := pointOnHull
///         endpoint := S[0]      // initial endpoint for a candidate edge on the hull
///         for j from 0 to |S| do
///             # endpoint == pointOnHull is a rare case and can happen only when j == 1 and a better endpoint has not yet been set for the loop
///             if (endpoint == pointOnHull) or (S[j] is on left of line from P[i] to endpoint) then
///                 endpoint := S[j]   // found greater left turn, update endpoint
///         i := i + 1
///         pointOnHull = endpoint
///     until endpoint = P[0]      // wrapped around to first hull point
/// ```
/// ## Analysis
/// First we find the smallest $x$ coordinate of all points, this takes $O(n)$ time to do. Then We
/// compare polar angles of each point from current point and choose the point with least angle.
/// This repeated $O(h)$ times to yield the hull.
///
/// Thus this algorithm yield the hull in $O(nh)$ time, wher $n$ is total number of points and $h$
/// is number of point on the hull.
pub fn jarvis_march(points: Vec<Vec2>, drawing_history: &mut Vec<Vec<LineType>>) -> Vec<Vec2> {
    let n = points.len();
    if n < 3 {
        return Vec::new();
    }

    let mut hull = Vec::new();

    // Find the leftmost point
    let mut l = 0;
    for i in 1..n {
        if points[i].x < points[l].x {
            l = i;
        }
    }

    // Start from leftmost point, keep moving counterclockwise
    // until reach the start point again
    let mut p = l;
    let mut q;
    loop {
        let mut temp = vec![];

        // Add current point to result
        hull.push(points[p]);

        // Search for a point 'q' such that orientation(p, x, q) is
        // counterclockwise for all points 'x'
        q = (p + 1) % n;
        for r in 0..n {
            // If r is more counterclockwise than current q, then update q
            if let Orientation::Counterclockwise = orientation(&points[p], &points[r], &points[q]) {
                q = r;
            }

            // Add line from points[p] to points[q] to drawing history
            // if it's not already part of the hull
            if !hull.contains(&points[r]) {
                temp.push(LineType::Temporary(points[p], points[r]));
            }
        }

        temp.push(LineType::PartOfHull(points[p], points[q]));

        // Now q is the most counterclockwise with respect to p
        // Set p as q for next iteration, so that q is added to result 'hull'
        p = q;

        // While we don't come to first point
        if p == l {
            break;
        }

        temp.push(LineType::TextComment(format!(
            "Checking all points starting from {} that are least counter clockwise",
            points[p]
        )));
        drawing_history.push(temp);
    }

    drawing_history.push(vec![
        LineType::PartOfHull(hull[hull.len() - 1], hull[0]),
        LineType::TextComment("Found all points of the Hull".to_string()),
    ]);

    hull
}

enum Orientation {
    Clockwise,
    Counterclockwise,
    Colinear,
}

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

enum HullType {
    UpperHull,
    LowerHull,
}

/// Implementation of the [Kirkpatrick Seidel](https://graphics.stanford.edu/courses/cs268-16-fall/Notes/KirkSeidel.pdf) Algorithm
///
/// ![](https://d3i71xaburhd42.cloudfront.net/9565745ce8b6c2114072e9620981fb97ed38e471/2-Figure1-1.png)
///
/// ## Pseudocode [ref](http://www.cse.yorku.ca/~andy/courses/6114/lecture-notes/KirkSeidel.pdf)
/// ```pseudocode
/// Algorithm UpperHull(P)
/// 0. if |P| ≤ 2 then return the obvious answer
/// 1. else begin
/// 2. Compute the median of x-coordinates of points in P.
/// 3. Partition P into two sets L and R each of size about n/2 around the median.
/// 4. Find the upper bridge pq of L and R, p∈L, and q∈R
/// 5. L′ ← { r ∈L x(r) ≤ x(p) }
/// 6. R′ ← { r ∈R x(r) ≥ x(q) }
/// 7. LUH ← UpperHull(L′)
/// 8. RUH ← UpperHull(R′)
/// 9. return the concatenated list LUH, pq, RUH as the upper hull of P.
/// 10. end
/// ```
///
/// ## Analysis
/// This is a divide-&-conquer algorithm. The key step is the computation of the upper
/// bridge which is based on the prune-&-search technique. This step can be done in O(n) time. We also know that step
/// 2 can be done in $O(n)$ time by the linear time median finding algorithm. Hence, steps 3-6
/// can be done in $O(n)$ time. For the purposes of analyzing algorithm `UpperHall(P)`, let us
/// assume the upper hull of $P$ consists of $h$ edges. Our analysis will use both parameters n
/// (input size) and $h$ (output size). Let $T(n, h)$ denote the worst-case time complexity of the
/// algorithm. Suppose $\text{LUH}$ and $\text{RUH}$ in steps 7 and 8 consist of $h1$ and $h2$ edges, respectively. Since $|L'| ≤ |L|$ and $|R'| ≤ |R|$, the two recursive calls in steps 7 and 8 take time
/// $T(\frac{n}2, h_1)$ and $T(\frac{n}2, h_2)$ time. (Note that $h = 1 + h1 + h2$. Hence, $h2 = h − 1 − h1$.)
/// Therefore, the recurrence that describes the worst-case time complexity of the algorithm is
/// $$
/// T(n, h) = O(n) + max_{h1} \{ T(\frac{n}{2}, h_1) + T(\frac{n}{2}, h-1-h_1)\}
/// $$
/// if $h > 2$, otherwise $T(n, h) = O(n)$.
///
/// Suppose the two occurences of $O(n)$ in the above recurrence are at most $cn$,
/// where $c$ is a suitably large constant. We will show by induction on $h$ that
/// $T(n, h) \leq cn \log(h)$ for all $n$ and $h \leq 2$. For the base case where $h = 2$,
/// $T(n, h) \leq cn \leq cn\log(2) = cn \log(h)$. For the inductive case,
///
/// $T(n, h)$
///
/// $\leq cn + max_{h1} \{ c\frac{n}{2}\log(h_1) + c\frac{n}{2}\log(h-1-h_1)\}$
///
/// $= cn + c\frac{n}{2}max_{h1} \log(h_1 (h - 1 - h_1))$
///
/// $\leq cn + c\frac{n}{2}\log(\frac{h}{2} \cdot \frac{h}{2})$
///
/// $= cn + c\frac{n}{2}2\log(\frac{h}2)$
///
/// $= cn\log(h)$
///
/// Thus we can claim runtime of kirpatrick seidel algorithm to be $O(n\log(h))$.
pub fn kirk_patrick_seidel(
    points: Vec<Vec2>,
    drawing_history: &mut Vec<Vec<LineType>>,
) -> Vec<Vec2> {
    let mut upper_hull_vec = upper_hull(&points, drawing_history, &HullType::UpperHull);
    drawing_history.push(vec![LineType::TextComment("Added upper hull".to_string())]);

    let mut lower_hull_vec = upper_hull(
        &points
            .iter()
            .map(|point| Vec2 {
                x: point.x,
                y: -point.y,
            })
            .collect(),
        drawing_history,
        &HullType::LowerHull,
    );
    drawing_history.push(vec![LineType::TextComment("Added lower hull".to_string())]);
    lower_hull_vec = lower_hull_vec
        .iter()
        .map(|point| Vec2 {
            x: point.x,
            y: -point.y,
        })
        .collect();

    drawing_history.push(vec![LineType::TextComment(
        "Kirkseidel algorithm is complete".to_string(),
    )]);

    upper_hull_vec.extend(lower_hull_vec);
    upper_hull_vec
}

fn upper_hull(
    points: &Vec<Vec2>,
    drawing_history: &mut Vec<Vec<LineType>>,
    hull_type: &HullType,
) -> Vec<Vec2> {
    let min_point = *points
        .iter()
        .min_by(|a, b| {
            a.x.partial_cmp(&b.x)
                .unwrap_or_else(|| a.y.partial_cmp(&b.y).unwrap())
        })
        .unwrap();

    let max_point = *points
        .iter()
        .max_by(|a, b| {
            a.x.partial_cmp(&b.x)
                .unwrap_or_else(|| a.y.partial_cmp(&b.y).unwrap())
        })
        .unwrap();

    if min_point == max_point {
        drawing_history.push(vec![LineType::TextComment(format!(
            "Single point convex hull found, returning the point",
        ))]);
        return vec![min_point];
    }

    let mut temporary = vec![min_point, max_point];
    temporary.extend(
        points
            .iter()
            .filter(|p| p.x > min_point.x && p.x < max_point.x),
    );

    return connect(min_point, max_point, &temporary, drawing_history, hull_type);
}

fn connect(
    min: Vec2,
    max: Vec2,
    points: &Vec<Vec2>,
    drawing_history: &mut Vec<Vec<LineType>>,
    hull_type: &HullType,
) -> Vec<Vec2> {
    let median = median_of_medians(&points.iter().map(|point| point.x).collect());
    drawing_history.push(vec![
        LineType::VerticalLine(median),
        LineType::TextComment(format!("Found the median at {}", median)),
    ]);

    let (left, right) = bridge(points, median);
    let (drawing_left, drawing_right) = match hull_type {
        HullType::LowerHull => (
            Vec2 {
                x: left.x,
                y: -left.y,
            },
            Vec2 {
                x: right.x,
                y: -right.y,
            },
        ),
        _ => (left, right),
    };
    drawing_history.push(vec![
        LineType::PartOfHull(drawing_left, drawing_right),
        LineType::TextComment(format!(
            "Found the bridge points {} and {}",
            drawing_left, drawing_right
        )),
    ]);

    let mut left_points = vec![left];
    left_points.extend(points.iter().filter(|p| p.x < left.x));

    let mut right_points = vec![right];
    right_points.extend(points.iter().filter(|p| p.x > right.x));

    let mut output = vec![];
    if left == min {
        output.extend(vec![left]);
    } else {
        output.extend(connect(min, left, &left_points, drawing_history, hull_type));
    }

    if right == max {
        output.extend(vec![right]);
    } else {
        output.extend(connect(
            right,
            max,
            &right_points,
            drawing_history,
            hull_type,
        ));
    }

    let mut temp = vec![];
    for i in 0..output.len() - 1 {
        temp.push(LineType::Temporary(output[i], output[i + 1]))
    }
    temp.push(LineType::TextComment(
        "Found the connecting hull".to_string(),
    ));

    return output;
}

fn bridge(points: &Vec<Vec2>, median: f32) -> (Vec2, Vec2) {
    let mut candidates: Vec<Vec2> = Vec::new();
    if points.len() == 2 {
        return if points[0].x < points[1].x {
            (points[0], points[1])
        } else {
            (points[1], points[0])
        };
    }

    let mut sorted_points = points.clone();
    sorted_points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());

    let mut pairs: Vec<(Vec2, Vec2)> = Vec::new();

    for chunk in sorted_points.chunks(2) {
        if chunk.len() == 2 {
            pairs.push((chunk[0], chunk[1]));
        } else {
            candidates.push(chunk[0]);
        }
    }

    let mut slopes = vec![];

    for (point_i, point_j) in pairs.iter() {
        if point_i.x == point_j.x {
            if point_i.y > point_j.y {
                if !candidates.contains(&point_i) {
                    candidates.push(*point_i);
                }
            } else {
                if !candidates.contains(&point_j) {
                    candidates.push(*point_j);
                }
            }
        } else {
            slopes.push((
                point_i,
                point_j,
                (point_i.y - point_j.y) / (point_i.x - point_j.x),
            ));
        }
    }

    let median_slope = median_of_medians(&slopes.iter().map(|(_, _, slope)| slope).collect());
    let small = slopes.iter().filter(|(_, _, slope)| slope < median_slope);
    let equal = slopes.iter().filter(|(_, _, slope)| slope == median_slope);
    let large = slopes.iter().filter(|(_, _, slope)| slope > median_slope);

    // set of points with maximum value of p.y - median_slope * p.x
    let max_value = points
        .iter()
        .map(|p| p.y - median_slope * p.x)
        .fold(f32::MIN, f32::max);
    let max_points: Vec<_> = points
        .iter()
        .filter(|p| (p.y - median_slope * p.x) == max_value)
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
            if !candidates.contains(point2) {
                candidates.push(**point2);
            }
        }

        for (_, point2, _) in equal {
            if !candidates.contains(point2) {
                candidates.push(**point2);
            }
        }

        for (point1, point2, _) in small {
            if !candidates.contains(point2) {
                candidates.push(**point2);
            }
            if !candidates.contains(point1) {
                candidates.push(**point1);
            }
        }
    } else if min_point.x > median {
        for (point1, _, _) in small {
            if !candidates.contains(point1) {
                candidates.push(**point1);
            }
        }

        for (point1, _, _) in equal {
            if !candidates.contains(point1) {
                candidates.push(**point1);
            }
        }

        for (point1, point2, _) in large {
            if !candidates.contains(point2) {
                candidates.push(**point2);
            }
            if !candidates.contains(point1) {
                candidates.push(**point1);
            }
        }
    }

    return bridge(&candidates, median);
}

pub fn median_of_medians<T: Clone + Copy + PartialOrd>(nums: &Vec<T>) -> T {
    match nums.len() {
        0 => panic!("No median of an empty list"),
        1 => nums[0],
        2..=5 => {
            let mut nums = nums.clone();
            nums.sort_by(|a, b| a.partial_cmp(&b).unwrap());
            nums[nums.len() / 2]
        }
        _ => median_of_medians(
            &nums
                .to_vec()
                .chunks(5)
                .map(|chunk| {
                    let mut chunk = chunk.to_vec();
                    chunk.sort_by(|a, b| a.partial_cmp(&b).unwrap());
                    chunk[chunk.len() / 2]
                })
                .collect::<Vec<T>>(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jarvis_march() {
        let points = vec![
            Vec2::new(0.0, 3.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(2.0, 1.0),
            Vec2::new(3.0, 0.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(3.0, 3.0),
        ];

        let mut drawing_history = Vec::new();
        let hull = jarvis_march(points, &mut drawing_history);

        let expected_hull = vec![
            Vec2::new(0.0, 3.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(3.0, 0.0),
            Vec2::new(3.0, 3.0),
        ];

        assert_eq!(hull, expected_hull);
    }

    #[test]
    fn test_kirk_patrick_seidel() {
        let points = vec![
            Vec2::new(0.0, 3.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(2.0, 1.0),
            Vec2::new(3.0, 0.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(3.0, 3.0),
        ];

        let mut drawing_history = Vec::new();
        let hull: Vec<Vec2> = kirk_patrick_seidel(points, &mut drawing_history);

        let expected_hull = vec![
            Vec2::new(0.0, 3.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(3.0, 0.0),
            Vec2::new(3.0, 3.0),
        ];

        assert_eq!(hull, expected_hull);
    }
}
