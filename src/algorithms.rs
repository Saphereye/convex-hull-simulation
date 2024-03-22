use bevy::prelude::*;

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
    GrahamScan,
}

#[derive(Resource)]
pub struct Algorithm(pub AlgorithmType);

pub enum LineType {
    PartOfHull(Vec2, Vec2),
    Temporary(Vec2, Vec2),
    TextComment(String),
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
            points[p].to_string()
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

pub fn graham_scan(mut points: Vec<Vec2>, drawing_history: &mut Vec<Vec<LineType>>) -> Vec<Vec2> {
    warn_once!("Graham scan is wrong");
    if points.len() < 3 {
        return Vec::new();
    }

    // sort points by 'angle'
    // We can take points[0] as reference point
    let first_point = points[0];
    points.sort_by(|a, b| {
        let a_angle = (a.y - first_point.y).atan2(a.x - first_point.x);
        let b_angle = (b.y - first_point.y).atan2(b.x - first_point.x);
        a_angle.partial_cmp(&b_angle).unwrap()
    });

    let mut hull = vec![first_point];
    let mut current_index = 1;

    while current_index + 1 != points.len() - 1 {
        let previous = points[current_index - 1];
        let current = points[current_index];
        let next = points[current_index + 1];

        match orientation(&previous, &current, &next) {
            Orientation::Counterclockwise => {
                // Add line from previous to current to drawing history
                hull.push(points[current_index]);
                drawing_history.push(vec![LineType::Temporary(previous, current)]);
                current_index += 1;
            }
            Orientation::Clockwise => {
                // Remove current from points
                points.remove(current_index);
            }
            Orientation::Colinear => {
                // Remove current from points
                points.remove(current_index);
            }
        }
    }

    points
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
    let convex_hull = upper_hull(points, drawing_history);

    let mut temp = vec![];
    for i in 0..convex_hull.len() {
        let j = (i + 1) % convex_hull.len();
        temp.push(LineType::PartOfHull(convex_hull[i], convex_hull[j]));
    }

    drawing_history.push(temp);

    convex_hull
}

fn upper_hull(points: Vec<Vec2>, drawing_history: &mut Vec<Vec<LineType>>) -> Vec<Vec2> {
    if points.len() < 3 {
        return points;
    }

    let x_cor_median = median_of_medians(&points).x;
    let (left, right): (Vec<Vec2>, Vec<Vec2>) = points
        .clone()
        .into_iter()
        .partition(|point| point.x <= x_cor_median);

    println!(
        "Points:{:?}, Left: {:?}, Right: {:?}, Median: {}",
        points, left, right, x_cor_median
    );

    let upper_bridge = upper_bridge(&left, &right, x_cor_median, drawing_history);

    let mut left_hull = upper_hull(
        left.into_iter()
            .filter(|point| point.x <= upper_bridge.0.x)
            .collect(),
        drawing_history,
    );
    let right_hull = upper_hull(
        right
            .into_iter()
            .filter(|point| point.x >= upper_bridge.1.x)
            .collect(),
        drawing_history,
    );

    // Merge the left and right hulls
    left_hull.extend(right_hull);

    left_hull
}

fn upper_bridge(
    left: &[Vec2],
    right: &[Vec2],
    reference_median: f32,
    drawing_history: &mut Vec<Vec<LineType>>,
) -> (Vec2, Vec2) {
    let mut max_intersection = f32::MIN;
    let mut max_points = (Vec2::default(), Vec2::default());

    for left_point in left {
        for right_point in right {
            let intersection = calculate_intersection(left_point, right_point, reference_median);

            if intersection > max_intersection {
                max_intersection = intersection;
                max_points = (*left_point, *right_point);
            }
        }
    }

    max_points
}

fn calculate_intersection(a: &Vec2, b: &Vec2, reference_median: f32) -> f32 {
    // Calculate the slope of the line
    let slope = (b.y - a.y) / (b.x - a.x);

    // Calculate the y-intercept of the line
    let y_intercept = a.y - slope * a.x;

    // Calculate the y-coordinate of the intersection of the line with the vertical line at reference_median
    let intersection = slope * reference_median + y_intercept;

    intersection
}

pub fn median_of_medians<T: AsRef<[Vec2]>>(nums: T) -> Vec2 {
    warn!("Median of medians is wrongly implemented");
    let mut nums = nums.as_ref().to_vec();
    nums.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    nums[nums.len() / 2]
    // let nums = nums.as_ref();
    // match nums.len() {
    //     0 => panic!("No median of an empty list"),
    //     1 => nums[0],
    //     _ => median_of_medians(
    //         nums.to_vec()
    //             .chunks(5)
    //             .map(|chunk| {
    //                 let mut chunk = chunk.to_vec();
    //                 chunk.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    //                 chunk[chunk.len() / 2]
    //             })
    //             .collect::<Vec<Vec2>>(),
    //     ),
    // }
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
    fn test_graham_scan() {
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
        let hull = graham_scan(points, &mut drawing_history);

        let expected_hull = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(3.0, 0.0),
            Vec2::new(3.0, 3.0),
            Vec2::new(0.0, 3.0),
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
