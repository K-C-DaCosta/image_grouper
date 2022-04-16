use super::*;
use rayon::prelude::*;
use std::{
    cmp::{Ordering, Reverse},
    collections::BinaryHeap,
    time::Instant,
};

#[derive(Debug)]
pub struct StackFrame {
    pub idx: usize,
    pub edge_idx: usize,
    pub len: usize,
    pub printed: bool,
}

/// explicit DFS as rust iterator 
pub struct MSTIterator<'a> {
    graph: &'a HashMap<usize, Vec<usize>>,
    visited: HashSet<usize>,
    stack: Vec<StackFrame>,
}
impl<'a> MSTIterator<'a> {
    pub fn new(g: &'a HammingMST) -> Self {
        let graph = &g.graph;
        let root = g.root;
        let stack = vec![StackFrame {
            idx: root,
            edge_idx: 0,
            len: graph.get(&root).unwrap().len(),
            printed: false,
        }];
        Self {
            graph,
            visited: HashSet::new(),
            stack,
        }
    }
}
impl<'a> Iterator for MSTIterator<'a> {
    type Item = Option<usize>;
    fn next(&mut self) -> Option<Self::Item> {
        let stack = &mut self.stack;
        let visited = &mut self.visited;
        let graph = &mut self.graph; 

        let mut res = None;
        stack.pop().map(|sf| {
            let mut sf = sf;

            if sf.printed == false {
                res = Some(sf.idx)
            }

            // edges exahasted
            if sf.edge_idx >= sf.len {
                return None;
            }

            let cur_node = sf.idx;
            let edge_cursor = sf.edge_idx;
            sf.edge_idx += 1;
            sf.printed = true;
            visited.insert(cur_node);
            stack.push(sf);

            let adj_node_list = graph.get(&cur_node).expect("cur_node should always exist");
            let adj_node_children_len = adj_node_list.len();

            if edge_cursor < adj_node_children_len {
                let idx = adj_node_list[edge_cursor];
                let len = graph.get(&idx).expect("node should exist").len();
                if visited.contains(&idx) == false {
                    stack.push(StackFrame {
                        idx,
                        edge_idx: 0,
                        len,
                        printed: false,
                    });
                }
            }

            res
        })
    }
}

#[derive(Debug)]
/// contructus a minimum spanning tree were hamming distance is minimized
pub struct HammingMST {
    pub graph: HashMap<usize, Vec<usize>>,
    root: usize,
}
impl HammingMST {
    //create minimum spanning tree with kruskals algorithm
    pub fn new(nodes: &[ImageEntry]) -> Option<Self> {
        #[derive(Copy, Clone, Eq, Ord, Debug)]
        pub struct Edge {
            a: usize,
            b: usize,
            cost: u64,
        }
        impl PartialEq for Edge {
            fn eq(&self, other: &Self) -> bool {
                self.cost.eq(&other.cost)
            }
        }
        impl PartialOrd for Edge {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                self.cost.partial_cmp(&other.cost)
            }
        }

        if nodes.len() <= 1 {
            return None;
        }

        let mut graph = HashMap::<usize, Vec<usize>>::with_capacity(nodes.len());
        let mut edge_table = BinaryHeap::new();
        let mut disjoint_sets: Vec<HashSet<usize>> = vec![];

        let edge_iter = nodes
            .iter()
            .enumerate()
            .flat_map(|(a_idx, a_info)| {
                nodes
                    .iter()
                    .enumerate()
                    .map(move |(b_idx, b_info)| (a_idx, a_info, b_idx, b_info))
            })
            .filter(|(a, _, b, _)| a < b)
            .map(|(a, ai, b, bi)| {
                Reverse(Edge {
                    a,
                    b,
                    cost: perceptual::hamming_distance(ai.hash, bi.hash),
                })
            });

        for x in edge_iter {
            // println!("{:?}",x);
            edge_table.push(x);
        }

        while let Some(Reverse(edge)) = edge_table.pop() {
            // println!("{:?} popped..", edge);
            let a = edge.a;
            let b = edge.b;

            let a_in_mst = graph.contains_key(&a);
            let b_in_mst = graph.contains_key(&b);

            if a_in_mst != b_in_mst {
                if a_in_mst {
                    graph.get_mut(&a).unwrap().push(b);
                    graph.insert(b, vec![a]);

                    disjoint_sets
                        .iter_mut()
                        .find(|set| set.contains(&a))
                        .expect("set should exist")
                        .insert(b);
                } else {
                    graph.get_mut(&b).unwrap().push(a);
                    graph.insert(a, vec![b]);

                    disjoint_sets
                        .iter_mut()
                        .find(|set| set.contains(&b))
                        .expect("set should exist")
                        .insert(a);
                }
            } else if a_in_mst == false && b_in_mst == false {
                graph.insert(a, vec![b]);
                graph.insert(b, vec![a]);

                disjoint_sets.push(HashSet::new());
                disjoint_sets.last_mut().unwrap().insert(a);
                disjoint_sets.last_mut().unwrap().insert(b);
            } else {
                let mut sets_with_a_or_b_iter = disjoint_sets
                    .iter()
                    .enumerate()
                    .filter(|(_k, g)| g.contains(&a) != g.contains(&b))
                    .map(|(k, _)| k);

                let res_1 = sets_with_a_or_b_iter.next();
                let res_2 = sets_with_a_or_b_iter.next();
                if let Some((set_idx_1, set_idx_2)) = res_1.zip(res_2) {
                    let union = disjoint_sets[set_idx_1]
                        .union(&disjoint_sets[set_idx_2])
                        .map(|&a| a)
                        .collect::<HashSet<_>>();

                    if set_idx_1 > set_idx_2 {
                        disjoint_sets.remove(set_idx_1);
                        disjoint_sets.remove(set_idx_2);
                    } else {
                        disjoint_sets.remove(set_idx_2);
                        disjoint_sets.remove(set_idx_1);
                    }

                    disjoint_sets.push(union);

                    graph
                        .get_mut(&a)
                        .expect("both a and b should exist in graph")
                        .push(b);
                    graph
                        .get_mut(&b)
                        .expect("both a and b should exist in graph")
                        .push(a);
                }
            }
        }

        Some(Self { graph, root: 0 })
    }

    /// create minimum spanning tree with prims algorithm
    pub fn new_prims(nodes: &[ImageEntry]) -> Option<Self> {
        #[derive(Eq, Ord)]
        pub struct HeapKey {
            a_idx: usize,
            b_idx: usize,
            dist: u64,
        }
        impl PartialEq for HeapKey {
            fn eq(&self, other: &Self) -> bool {
                self.dist.eq(&other.dist)
            }
        }
        impl PartialOrd for HeapKey {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                self.dist.partial_cmp(&other.dist)
            }
        }

        let mut visited_list = Vec::with_capacity(nodes.len());
        let mut visited_table = HashSet::<usize>::with_capacity(nodes.len());
        let mut graph = HashMap::new();

        if nodes.len() <= 1 {
            return None;
        }

        visited_list.push(0);
        visited_table.insert(0);
        graph.insert(0, vec![]);
        let mut nodes_left = nodes.len() - 1;

        while nodes_left > 0 {
            let lowest_cost_edge = visited_list
                .iter()
                .map(|&idx| (idx, nodes[idx].hash))
                .flat_map(|(vidx, vhash)| {
                    nodes
                        .iter()
                        .enumerate()
                        .filter(|(adj_idx, _)| visited_table.contains(adj_idx) == false)
                        .map(move |(adj_idx, _)| (vidx, vhash, adj_idx, nodes[adj_idx].hash))
                })
                .map(|(vidx, vhash, adj_idx, adj_hash)| {
                    (vidx, adj_idx, perceptual::hamming_distance(vhash, adj_hash))
                })
                .par_bridge()
                .min_by_key(|&(_, _, dist)| dist);
            if let Some((vidx, adj_idx, _dist)) = lowest_cost_edge {
                visited_list.push(adj_idx);
                visited_table.insert(adj_idx);
                graph.insert(adj_idx, vec![]);
                graph.get_mut(&vidx).map(|adj_nodes| {
                    adj_nodes.push(adj_idx);
                });
                nodes_left -= 1;
            }
        }
        Some(Self { graph, root: 0 })
    }

    pub fn iter<'a>(&'a self)->MSTIterator<'a>{
        MSTIterator::new(&self)
    }



    /// do a dfs on the tree
    pub fn dfs_preorder_iterative<CB: FnMut(&Self, &StackFrame)>(&self, mut call_back: CB) {
        let graph = &self.graph;
        let mut visited = HashSet::<usize>::new();
        let mut stack: Vec<StackFrame> = Vec::new();

        let root = self.root;

        stack.push(StackFrame {
            idx: root,
            edge_idx: 0,
            len: self.graph.get(&root).unwrap().len(),
            printed: false,
        });

        while let Some(sf) = stack.pop() {
            let mut sf = sf;

            if sf.printed == false {
                call_back(self, &sf);
            }

            // edges exahasted
            if sf.edge_idx >= sf.len {
                continue;
            }

            let cur_node = sf.idx;
            let edge_cursor = sf.edge_idx;
            sf.edge_idx += 1;
            sf.printed = true;
            visited.insert(cur_node);
            stack.push(sf);

            let adj_node_list = graph.get(&cur_node).expect("cur_node should always exist");
            let adj_node_children_len = adj_node_list.len();

            if edge_cursor < adj_node_children_len {
                let idx = adj_node_list[edge_cursor];
                let len = graph.get(&idx).expect("node should exist").len();
                if visited.contains(&idx) == false {
                    stack.push(StackFrame {
                        idx,
                        edge_idx: 0,
                        len,
                        printed: false,
                    });
                }
            }
        }
    }
}


pub fn iteratively_improve_tour(max_iterations:u64, max_time:u128, circuit:&mut Vec<usize>, nodes:&Vec<ImageEntry>){
    let len = nodes.len(); 
    let t0 = Instant::now();
    let mut iterations = 0; 
    
    let calc_cost = |c:&Vec<usize>|->u64 {
        let mut cost = 0; 
        for i in 0..len-1 {
            let hash_i = nodes[c[i]].hash;
            let hash_j = nodes[c[i+1]].hash; 
            cost+=perceptual::hamming_distance(hash_i,hash_j);
        }
        cost
    };

    let mut cost = calc_cost(&circuit);

    let before = cost; 
    
    while t0.elapsed().as_millis() < max_time && iterations < max_iterations{
        let a = fastrand::usize(0..len);
        let b = fastrand::usize(0..len);
        if a != b {
            circuit.swap(a, b);
            let new_cost = calc_cost(&circuit);
            if new_cost < cost {
               
                cost = new_cost;
            }else{
                circuit.swap(a, b);
            }
        }
        iterations+=1; 
    }

    println!("iteratively improved by: [before = {}, after = {}]",before, cost);

}
