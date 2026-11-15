//! Positive-weight FGK trees with stable node identities and explicit sibling ranks.
use crate::{bitstream::Bits, invalid, stream_io::filled};
use std::{
    collections::VecDeque,
    io::{self, Read},
};
#[derive(Clone, Copy)]
struct Node {
    weight: u32,
    parent: Option<usize>,
    left: Option<usize>,
    right: usize,
}
pub(crate) struct Adaptive {
    nodes: Vec<Node>,
    order: Vec<usize>,
    rank: Vec<usize>,
    alphabet: usize,
    leaves: usize,
    root: usize,
    growing: bool,
    unix_updates: Option<u32>,
}
fn take(
    queue: &mut VecDeque<usize>,
    parents: &mut VecDeque<usize>,
    nodes: &[Node],
) -> io::Result<usize> {
    let leaf = match (queue.front(), parents.front()) {
        (Some(&a), Some(&b)) => nodes[a].weight <= nodes[b].weight,
        (Some(_), None) => true,
        (None, Some(_)) => false,
        _ => return Err(invalid()),
    };
    if leaf {
        queue.pop_front()
    } else {
        parents.pop_front()
    }
    .ok_or_else(invalid)
}
impl Adaptive {
    pub fn new<R: Read, P: FnMut() -> io::Result<()>>(
        alphabet: usize,
        bits: &mut Bits<R, P, true>,
    ) -> io::Result<Self> {
        if !(2..=512).contains(&alphabet) {
            return Err(invalid());
        }
        let count = alphabet * 2 - 1;
        bits.charge(count as u64)?;
        let nodes = filled(
            count,
            Node {
                weight: 1,
                parent: None,
                left: None,
                right: 0,
            },
        )?;
        let mut order = Vec::new();
        order
            .try_reserve_exact(count)
            .map_err(|_| crate::stream_io::resource())?;
        order.extend(0..alphabet);
        let rank = filled(count, 0)?;
        let mut result = Self {
            nodes,
            order,
            rank,
            alphabet,
            leaves: alphabet,
            root: count - 1,
            growing: false,
            unix_updates: None,
        };
        result.rebuild(false, bits)?;
        Ok(result)
    }
    pub fn growing<R: Read, P: FnMut() -> io::Result<()>>(
        capacity: usize,
        bits: &mut Bits<R, P, true>,
    ) -> io::Result<Self> {
        let mut result = Self::new(capacity, bits)?;
        result.order.clear();
        result.order.push(0);
        result.rank[0] = 0;
        result.leaves = 1;
        result.root = 0;
        result.nodes[0] = Node {
            weight: 1,
            parent: None,
            left: None,
            right: 0,
        };
        result.growing = true;
        Ok(result)
    }
    pub fn grow<R: Read, P: FnMut() -> io::Result<()>>(
        &mut self,
        symbol: usize,
        bits: &mut Bits<R, P, true>,
    ) -> io::Result<()> {
        if !self.growing || symbol != self.leaves || symbol >= self.alphabet {
            return Err(invalid());
        }
        bits.charge((self.order.len() + 16) as u64)?;
        let lowest = self.order[0];
        if lowest >= self.alphabet {
            return Err(invalid());
        }
        let parent = self.nodes[lowest].parent;
        let internal = self.alphabet + self.leaves - 1;
        self.nodes[internal] = Node {
            weight: self.nodes[lowest].weight,
            parent,
            left: Some(symbol),
            right: lowest,
        };
        self.nodes[lowest].parent = Some(internal);
        self.nodes[symbol] = Node {
            weight: 0,
            parent: Some(internal),
            left: None,
            right: 0,
        };
        if let Some(parent) = parent {
            if self.nodes[parent].left == Some(lowest) {
                self.nodes[parent].left = Some(internal);
            } else if self.nodes[parent].right == lowest {
                self.nodes[parent].right = internal;
            } else {
                return Err(invalid());
            }
        } else {
            self.root = internal;
        }
        self.order.insert(0, symbol);
        self.order.insert(2, internal);
        self.leaves += 1;
        for (at, &id) in self.order.iter().enumerate() {
            self.rank[id] = at;
        }
        self.update(symbol, bits)
    }
    pub fn use_unix_position_counter(&mut self) -> io::Result<()> {
        if !self.growing || self.leaves != 1 {
            return Err(invalid());
        }
        self.unix_updates = Some(0);
        Ok(())
    }
    fn rebuild<R: Read, P: FnMut() -> io::Result<()>>(
        &mut self,
        halve: bool,
        bits: &mut Bits<R, P, true>,
    ) -> io::Result<()> {
        bits.charge((self.nodes.len() * 8) as u64)?;
        let mut leaves = VecDeque::new();
        leaves
            .try_reserve(self.alphabet)
            .map_err(|_| crate::stream_io::resource())?;
        for &id in &self.order {
            if id < self.alphabet {
                let node = &mut self.nodes[id];
                if halve {
                    node.weight = node.weight.div_ceil(2);
                }
                node.parent = None;
                node.left = None;
                leaves.push_back(id);
            }
        }
        let mut leaf_order = VecDeque::new();
        leaf_order
            .try_reserve(self.leaves)
            .map_err(|_| crate::stream_io::resource())?;
        leaf_order.extend(leaves.iter().copied());
        let mut parents = VecDeque::new();
        parents
            .try_reserve(self.alphabet)
            .map_err(|_| crate::stream_io::resource())?;
        let parent_end = self.alphabet + self.leaves - 1;
        for id in self.alphabet..parent_end {
            let left = take(&mut leaves, &mut parents, &self.nodes)?;
            let right = take(&mut leaves, &mut parents, &self.nodes)?;
            self.nodes[id] = Node {
                weight: self.nodes[left].weight + self.nodes[right].weight,
                parent: None,
                left: Some(left),
                right,
            };
            self.nodes[left].parent = Some(id);
            self.nodes[right].parent = Some(id);
            parents.push_back(id);
        }
        let mut leaves = leaf_order;
        parents.clear();
        parents.extend(self.alphabet..parent_end);
        self.order.clear();
        while self.order.len() < self.leaves * 2 - 1 {
            let id = take(&mut leaves, &mut parents, &self.nodes)?;
            self.rank[id] = self.order.len();
            self.order.push(id);
        }
        Ok(())
    }
    fn exchange(&mut self, a: usize, b: usize) -> io::Result<()> {
        let pa = self.nodes[a].parent.ok_or_else(invalid)?;
        let pb = self.nodes[b].parent.ok_or_else(invalid)?;
        if pa == pb {
            let parent = &mut self.nodes[pa];
            let left = parent.left.ok_or_else(invalid)?;
            parent.left = Some(parent.right);
            parent.right = left;
        } else {
            for (parent, old, new) in [(pa, a, b), (pb, b, a)] {
                if self.nodes[parent].left == Some(old) {
                    self.nodes[parent].left = Some(new);
                } else if self.nodes[parent].right == old {
                    self.nodes[parent].right = new;
                } else {
                    return Err(invalid());
                }
            }
            self.nodes[a].parent = Some(pb);
            self.nodes[b].parent = Some(pa);
        }
        let ra = self.rank[a];
        let rb = self.rank[b];
        self.order.swap(ra, rb);
        self.rank[a] = rb;
        self.rank[b] = ra;
        Ok(())
    }
    fn update<R: Read, P: FnMut() -> io::Result<()>>(
        &mut self,
        symbol: usize,
        bits: &mut Bits<R, P, true>,
    ) -> io::Result<()> {
        if self
            .unix_updates
            .map_or(self.nodes[self.root].weight == 32768, |updates| {
                updates == 32768
            })
        {
            self.rebuild(true, bits)?;
            if self.unix_updates.is_some() {
                self.unix_updates = Some(self.nodes[self.root].weight);
            }
        }
        if let Some(updates) = &mut self.unix_updates {
            *updates += 1;
            if self.leaves == 1 {
                bits.charge(1)?;
                return Ok(());
            }
        }
        let mut id = symbol;
        for _ in 0..self.nodes.len() {
            let mut leader = self.rank[id];
            let weight = self.nodes[id].weight;
            while leader + 1 < self.order.len() {
                bits.charge(1)?;
                if self.nodes[self.order[leader + 1]].weight != weight {
                    break;
                }
                leader += 1;
            }
            let other = self.order[leader];
            bits.charge(8)?;
            if other != id {
                self.exchange(id, other)?;
            }
            self.nodes[id].weight += 1;
            if let Some(parent) = self.nodes[id].parent {
                id = parent;
            } else {
                return Ok(());
            }
        }
        Err(invalid())
    }
    pub fn decode<R: Read, P: FnMut() -> io::Result<()>>(
        &mut self,
        bits: &mut Bits<R, P, true>,
    ) -> io::Result<u16> {
        let mut id = self.root;
        for _ in 0..self.nodes.len() {
            if let Some(left) = self.nodes[id].left {
                id = if bits.get(1)? == 0 {
                    left
                } else {
                    self.nodes[id].right
                };
            } else {
                if id >= self.alphabet {
                    return Err(invalid());
                }
                self.update(id, bits)?;
                return Ok(id as u16);
            }
        }
        Err(invalid())
    }
}
