use super::*;
#[derive(Debug)]
pub struct StackFrame {
    pub idx: usize,
    pub edge_idx: usize,
    pub len: usize,
    pub printed: bool,
}

#[derive(Debug)]
pub struct HammingMST {
    pub graph: HashMap<usize, Vec<usize>>,
}
impl HammingMST {
    pub fn new(nodes: &[ImageEntry]) -> Option<Self> {
        let mut visited_list = Vec::new();
        let mut visited_table = HashSet::<usize>::new();
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
        Some(Self { graph })
    }

    pub fn hamiltonian_circuit<CB: Fn(&StackFrame)>(&self, call_back: CB) {
        let graph = &self.graph;
        let mut stack: Vec<StackFrame> = Vec::new();

        stack.push(StackFrame {
            idx: 0,
            edge_idx: 0,
            len: self.graph.get(&0).unwrap().len(),
            printed: false,
        });

        while let Some(sf) = stack.pop() {
            let mut sf = sf; 

            if sf.printed == false {
                call_back(&sf);
            }

            // edges exahasted
            if sf.edge_idx >= sf.len {
                continue;
            }

            let cur_node = sf.idx;
            let edge_cursor = sf.edge_idx;
            sf.edge_idx += 1;
            sf.printed = true;
            stack.push(sf);

            let adj_node_list = graph.get(&cur_node).expect("cur_node should always exist");
            let adj_node_children_len = adj_node_list.len();

            if edge_cursor < adj_node_children_len {
                let idx = adj_node_list[edge_cursor];
                let len = graph.get(&idx).expect("node should exist").len();

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
