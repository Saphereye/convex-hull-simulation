use std::cmp::Ordering;
use std::hash::Hash;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use std::collections::HashSet;

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
}

/// # Implementation of the [Jarvis March](https://en.wikipedia.org/wiki/Gift_wrapping_algorithm) algorithm
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
            if orientation(&points[p], &points[r], &points[q]) == 2 {
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

        drawing_history.push(temp);
    }

    // // Add final hull lines to drawing history
    // for i in 0..hull.len() - 1 {
    //     drawing_history.push(vec![LineType::PartOfHull(hull[i], hull[i + 1])]);
    // }
    // Connect the last point with the first one
    drawing_history.push(vec![LineType::PartOfHull(hull[hull.len() - 1], hull[0])]);

    hull
}

// To find orientation of ordered triplet (p, q, r).
// The function returns following values
// 0 --> p, q and r are colinear
// 1 --> Clockwise
// 2 --> Counterclockwise
fn orientation(p: &Vec2, q: &Vec2, r: &Vec2) -> i32 {
    let val = (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y);

    if val == 0.0 {
        return 0; // colinear
    }
    if val > 0.0 {
        return 1; // clockwise
    }
    return 2; // counterclockwise
}

pub fn kirk_patrick_seidel(
    points: Vec<Vec2>,
    drawing_history: &mut Vec<Vec<LineType>>,
) -> Vec<Vec2> {
    todo!("Kirk Patrick Seidel algorithm is not implemented yet");
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
            Vec2::new(0.0, 0.0),
            Vec2::new(3.0, 0.0),
            Vec2::new(3.0, 3.0),
            Vec2::new(0.0, 3.0),
        ];

        assert_eq!(hull, expected_hull);
    }
}
