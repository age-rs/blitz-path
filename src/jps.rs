use std::collections::BinaryHeap;

use movingai::Coords2D;
use movingai::Map2D;
use movingai::MovingAiMap;

use crate::node::Node;
use crate::utils::{direction, distance, rewind};
use crate::Route;

#[derive(Copy, Clone)]
enum Direction {
    Vertical(i32),
    Horizontal(i32),
    Diagonal(i32, i32),
}

///Creates a new route using the JPS algorithm.
///Returns a Route struct containing the distance to the goal and number of steps needed to get there.
/// # Examples
///
/// ```
/// use std::path::Path;
///
/// let map = movingai::parser::parse_map_file(Path::new("./tests/map/maze512-32-9.map")).expect("Could not load map.");
/// let scenes = movingai::parser::parse_scen_file(Path::new("./tests/map/maze512-32-9.map.scen")).expect("Could not load scenario.");
/// let scene = &scenes[0];
///
/// let path = blitz_path::jps_path(&map, scene.start_pos, scene.goal_pos);
///
/// // using as f32 as scene.optimal_length is stored as f64,
/// // but only seems to have precision to f32
/// if let Some(path) = path {
///     assert_eq!(scene.optimal_length as f32, path.distance() as f32);  
/// }
/// ```

pub fn jps_path(map: &MovingAiMap, start: Coords2D, goal: Coords2D) -> Option<Route> {
    //Initialize open and closed lists
    let mut open = BinaryHeap::new();
    let mut closed = Vec::<Node>::new();

    //Push start node to open list
    let start_node = Node::new(0.0, distance(start, goal), start, start);
    if start == goal {
        open.push(start_node);
    } else {
        //Add start's neighbours to open list - modified as seems to be error in neighbours function
        let prev_x = start_node.position.0 - 1;
        let next_x = start_node.position.0 + 1;
        let prev_y = start_node.position.1 - 1;
        let next_y = start_node.position.1 + 1;
        for x in prev_x..=next_x {
            for y in prev_y..=next_y {
                let coords = Coords2D::from((x, y));
                let node = Node::from_parent(&start_node, coords, goal);
                open.push(node);
            }
        }

        closed.push(start_node);
    }

    //Examine the nodes
    while let Some(node_current) = open.pop() {
        //If this is the target node return the distance to get there
        if node_current.position == goal {
            //Push all remaining to closed
            for node in open {
                closed.push(node);
            }

            //Unwind
            let path = rewind(&node_current, &closed);
            let route = Route::from((node_current.g, path));
            return Some(route);
        }

        //Check if node is on closed list and continue if is
        if closed.contains(&node_current) {
            continue;
        }

        //Calculate direction
        let direction = direction(node_current.position, node_current.parent);

        if let Some(nodes) = check_jump(&node_current, map, (direction.0, direction.1), goal) {
            for node in nodes {
                open.push(node);
            }
        }

        //Push current node to closed list
        closed.push(node_current);
    }

    None
}

fn check_jump(
    parent: &Node,
    map: &MovingAiMap,
    direction: (i32, i32),
    goal: Coords2D,
) -> Option<Vec<Node>> {
    //println!("Checking: {:?}", parent.position);
    //Expand depending on direction
    //Diagonal case
    let dir = if direction.0 != 0 && direction.1 != 0 {
        Direction::Diagonal(direction.0, direction.1)
    }
    //Horizontal case
    else if direction.0 != 0 {
        Direction::Horizontal(direction.0)
    }
    //Vertical
    else {
        Direction::Vertical(direction.1)
    };

    if let Some(nodes) = expand(map, &parent, dir, goal) {
        Some(nodes)
    } else {
        None
    }
}

fn forced_horizontal(
    map: &MovingAiMap,
    check_node: &Node,
    direction: i32,
    goal: Coords2D,
) -> Option<Vec<Node>> {
    let next_x = (check_node.position.0 as i32 + direction) as usize;
    let up_y = (check_node.position.1 as i32 - 1) as usize;
    let down_y = (check_node.position.1 as i32 + 1) as usize;

    let mut nodes = Vec::new();

    //Check if blocked up
    if (!map.is_traversable(Coords2D::from((check_node.position.0, up_y))))
        && (map.is_traversable(Coords2D::from((next_x, up_y))))
    {
        let jump_point = Coords2D::from((next_x, up_y));
        let jump_node = Node::from_parent(&check_node, jump_point, goal);
        nodes.push(jump_node);
    }

    //Check if blocked down
    if (!map.is_traversable(Coords2D::from((check_node.position.0, down_y))))
        && (map.is_traversable(Coords2D::from((next_x, down_y))))
    {
        let jump_point = Coords2D::from((next_x, down_y));
        let jump_node = Node::from_parent(&check_node, jump_point, goal);
        nodes.push(jump_node);
    }

    if !nodes.is_empty() {
        Some(nodes)
    } else {
        None
    }
}

fn forced_vertical(
    map: &MovingAiMap,
    check_node: &Node,
    direction: i32,
    goal: Coords2D,
) -> Option<Vec<Node>> {
    let next_y = (check_node.position.1 as i32 + direction) as usize;
    let left_x = (check_node.position.0 as i32 - 1) as usize;
    let right_x = (check_node.position.0 as i32 + 1) as usize;

    let mut nodes = Vec::new();

    //Check if blocked left
    if (!map.is_traversable(Coords2D::from((left_x, check_node.position.1))))
        && (map.is_traversable(Coords2D::from((left_x, next_y))))
    {
        let jump_point = Coords2D::from((left_x, next_y));
        let jump_node = Node::from_parent(&check_node, jump_point, goal);
        nodes.push(jump_node);
    }

    //Check if blocked right
    if (!map.is_traversable(Coords2D::from((right_x, check_node.position.1))))
        && (map.is_traversable(Coords2D::from((right_x, next_y))))
    {
        let jump_point = Coords2D::from((right_x, next_y));
        let jump_node = Node::from_parent(&check_node, jump_point, goal);
        nodes.push(jump_node);
    }

    if !nodes.is_empty() {
        Some(nodes)
    } else {
        None
    }
}

fn expand(
    map: &MovingAiMap,
    start_node: &Node,
    direction: Direction,
    goal: Coords2D,
) -> Option<Vec<Node>> {
    let mut current = *start_node;
    let mut nodes = Vec::new();
    loop {
        //Check if goal
        if current.position == goal {
            nodes.push(current);

            return Some(nodes);
        }

        //Check blocked
        if !map.is_traversable(current.position) {
            return None;
        }

        //Otherwise Expand depending on direction
        let dir;
        match direction {
            Direction::Vertical(vert) => {
                dir = (0, vert);
                //Check for forced neighbours
                if let Some(mut vert_nodes) = forced_vertical(map, &current, vert, goal) {
                    nodes.append(&mut vert_nodes);
                }
            }
            Direction::Horizontal(hor) => {
                dir = (hor, 0);
                //Check for forced neighbours
                if let Some(mut hor_nodes) = forced_horizontal(map, &current, hor, goal) {
                    nodes.append(&mut hor_nodes);
                }
            }
            Direction::Diagonal(hor, vert) => {
                dir = (hor, vert);
                //Expand horizontally
                if let Some(mut hor_nodes) = expand(map, &current, Direction::Horizontal(hor), goal)
                {
                    nodes.append(&mut hor_nodes);
                }
                //Expand vertically
                if let Some(mut vert_nodes) = expand(map, &current, Direction::Vertical(vert), goal)
                {
                    nodes.append(&mut vert_nodes);
                }
            }
        }

        let next_x = (current.position.0 as i32 + dir.0) as usize;
        let next_y = (current.position.1 as i32 + dir.1) as usize;
        let next_position = Coords2D::from((next_x, next_y));

        //If forced neighbours found return them along with this node and next on to continue checking in this direction
        if !nodes.is_empty() {
            let next_node = Node::from_parent(&current, next_position, goal);
            nodes.push(current);
            nodes.push(next_node);

            return Some(nodes);
        }

        //Else move onto next tile
        current = Node::from_parent(start_node, next_position, goal);
    }
}
