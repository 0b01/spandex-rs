#![feature(conservative_impl_trait)]
extern crate petgraph;
use petgraph::algo::{Cycle, toposort};
use petgraph::graph::Graph;
use petgraph::graph::NodeIndex;

use std::marker::PhantomData;
use std::fmt::{Debug, Formatter, self};

/// Kind
trait Kind: Debug {
}

/// Variable Kind
struct Var<T> {
    value: T,
}
impl<'a, T> Kind for Var<T> {
}
impl<T> Var<T> {
    fn new(value: T) -> Self {
        Var {
            value,
        }
    }
}
impl<T> Debug for Var<T> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Var")
    }
}

struct Map2<A1, A2, R, F: Fn(A1, A2)->R> 
{
    a1: Incr<A1>,
    a2: Incr<A2>,
    f: Box<F>
}
impl<A1, A2, R, F:Fn(A1, A2)->R> Kind for Map2<A1, A2, R, F> 
{

}
impl<A1, A2, R, F:Fn(A1, A2)->R> Map2<A1, A2, R, F> 
{
    fn new(a1: Incr<A1>, a2: Incr<A2>, f: Box<F>) -> Self {
        Self { a1, a2, f }
    }
}
impl<A1, A2, R, F:Fn(A1,A2)->R> Debug for Map2<A1, A2, R, F> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Map2")
    }
}

//--------------------------------------------------------

/// Node
#[derive(Debug)]
struct Node<'a> {
    kind: Box<Kind + 'a>,
    pub stabilization_num: usize,
    id: usize,
}
impl<'a> Node<'a> {
    fn new(kind: Box<Kind + 'a>, id: usize) -> Self {
        Node {
            kind,
            stabilization_num: 0,
            id,
        }
    }
}

#[derive(Debug)]
struct Incr<T> {
    pub idx: NodeIndex<u32>,
    _type: PhantomData<T>,
}
impl<T> Incr<T> {
    fn new(idx: NodeIndex<u32>) -> Self {
        Incr {
            idx,
            _type: PhantomData,
        }
    }
}

struct Id {id: usize}
impl Iterator for Id {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        self.id += 1;
        Some(self.id)
    }
}
impl Id {
    fn new()->Self{Self{id:0}}
    fn none()->usize{0}
}

/// Incremental
struct Incremental<'a> {
    graph: Graph<Node<'a>, u32>,
    node_id_counter: Id,
    stabilization_num_counter: Id,
}
impl<'a> Incremental<'a> {
    fn new() -> Self {
        Incremental {
            graph: Graph::new(),
            node_id_counter: Id::new(),
            stabilization_num_counter: Id::new(),
        }
    }
    fn var<T: 'a>(&mut self, value: T) -> Incr<T> {
        let kind = Box::new(Var::new(value));
        let id = self.node_id_counter.next().unwrap();
        let node = Node::new(kind, id);
        let idx = self.graph.add_node(node);
        Incr::new(idx)
    }
    fn map2<A1:'a,A2:'a,R:'a,F:'a+Fn(A1,A2)->R>(&mut self, a: Incr<A1>, b: Incr<A2>, f:Box<F>) -> Incr<R>
    {
        let a_idx = a.idx;
        let b_idx = b.idx;

        let kind = Box::new(Map2::new(a, b, f));
        let id = self.node_id_counter.next().unwrap();
        let node = Node::new(kind, id);
        let idx = self.graph.add_node(node);

        self.graph.add_edge(a_idx, idx, 1);
        self.graph.add_edge(b_idx, idx, 1);
        Incr::new(idx)
    }
    fn stabilize(&mut self) -> Result<(), Cycle<NodeIndex>> {
        let current_stabilization_num = self.stabilization_num_counter.next().unwrap();
        let nodes_idx = toposort(&self.graph, None)?;
        let nodes = nodes_idx.iter()
            .map(|&idx| &self.graph[idx])
            .map(|node| {println!("{:?}, {}", node, current_stabilization_num); node })
            .filter(|node| node.stabilization_num < current_stabilization_num)
            .collect::<Vec<&Node>>();
        println!("{:?}", nodes);
        Ok(())
    }
}

fn test_run() {
    let mut incr = Incremental::new();
    let a = incr.var(3);
    let b = incr.var(5);
    let c = incr.map2(a, b, Box::new(|a:i32,b:i32| {a + b}));
    let n = incr.var(1);
    let d = incr.map2(n, c, Box::new(|a:i32,b:i32| {a + b}));
    incr.stabilize();
    
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_misc() {
        test_run();
    }
}