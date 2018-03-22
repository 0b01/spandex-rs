#![feature(universal_impl_trait)]
#![feature(box_syntax)]

extern crate petgraph;
use petgraph::algo::{Cycle, toposort};
use petgraph::graph::Graph;
use petgraph::graph::NodeIndex;

use std::marker::PhantomData;
use std::fmt::{Debug, Formatter, self};

use std::any::Any;

// use std::rc::Rc;
// use std::cell::RefCell;

/// Kind
trait Kind: Debug {
    type Item;
    fn recompute(&mut self, incr: &Incremental);
    fn observe(&self, incr: &Incremental) -> Self::Item;
}

// trait AsKind<T> {
//     fn as_kind(&self) -> &Kind<Item=Option<T>>;
// }

// impl<T> AsKind<T> for Var<T> {
//     fn as_kind(&self) -> &Kind<Item=Option<T>> {
//         self
//     }
// }

// impl<A1,A2,R,T> AsKind<R> for Map2<A1,A2,R,T>
//     where T: Fn(A1, A2) -> R
// {
//     fn as_kind(&self) -> &Kind<Item=Option<R>> {
//         self
//     }
// }

/// Variable Kind
struct Var<T>
    where T: 'static + Clone
{
    value: T,
}
impl<T: Clone> Kind for Var<T> {
    type Item = Option<T>;
    fn recompute(&mut self, _incr: &Incremental) {}
    fn observe(&self, _incr: &Incremental) -> Self::Item {
        Some(self.value.clone())
    }
}

impl<T: Clone> Var<T> {
    fn new(value: T) -> Self {
        Var {
            value: value,
        }
    }
}
impl<T: Clone> Debug for Var<T> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Var")
    }
}

struct Map2<'a, A1, A2, R, F> 
    where A1: 'static,
          A2: 'static,
          R: 'static + Clone,
          F: 'static + Fn(A1, A2)->R,
{
    a1: Box<Incr<Item=A1>>,
    a2: Box<Incr<Item=A2>>,
    f: Box<F>,
    r: Option<R>,
    lifetime: PhantomData<&'a F>,
}
impl<'a, A1, A2, R, F> Kind for Map2<'a, A1, A2, R, F> 
    where A1: 'static,
        A2: 'static,
        R: 'static + Clone,
        F: 'static + Fn(A1, A2)->R,
{
    type Item = Option<R>;
    fn recompute(&mut self, incr: &Incremental) {
        let a1 = self.a1.clone();
        let a2 = self.a2.clone();
        let a = incr.observe(a1).unwrap();
        let b = incr.observe(a2).unwrap();
        self.r = Some((self.f)(a, b))
    }
    fn observe(&self, _incr: &Incremental) -> Self::Item {
        self.r.clone()
    }
}
impl<'a, A1, A2, R, F> Map2<'a, A1, A2, R, F> 
    where A1: 'static,
          A2: 'static,
          R: 'static + Clone,
          F: 'static + Fn(A1, A2)->R,
{
    fn new(a1: Box<Incr<Item=A1>>, a2: Box<Incr<Item=A2>>, f: F) -> Self {
        let r = None;
        Self { a1: a1, a2: a2, f: box f, r , lifetime: PhantomData }
    }
}
impl<'a, A1, A2, R, F> Debug for Map2<'a, A1, A2, R, F>
    where A1: 'static,
          A2: 'static,
          R: 'static + Clone,
          F: 'static + Fn(A1, A2)->R,
{
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Map2")
    }
}

//--------------------------------------------------------


/// Node
#[derive(Debug)]
struct Node {
    kind: Box<Any>,
    pub stabilization_num: usize,
    id: usize,
}
impl Node {
    fn new(kind: Box<Any>, id: usize) -> Self {
        Node {
            kind,
            stabilization_num: 0,
            id,
        }
    }
}

trait Incr {
    type Item;
    fn idx(&self) -> NodeIndex<u32>;
    fn extract<'a>(&self, data: &'a Node, incr: &Incremental) -> Option<Self::Item>;
}

impl<T> Clone for Box<Incr<Item=T>> {
    fn clone(&self) -> Self {
        self.clone()
    }
}

impl<T> Incr for Box<Incr<Item=T>> {
    type Item = T;
    fn idx(&self) -> NodeIndex<u32> {
        self.idx()
    }
    fn extract<'a>(&self, data: &'a Node, incr: &Incremental) -> Option<Self::Item> {
        self.extract(data, incr)
    }
}


#[derive(Debug, Clone)]
struct IncrVar<T> {
    pub idx: NodeIndex<u32>,
    _type: PhantomData<T>,
}
impl<T> IncrVar<T> {
    fn new(idx: NodeIndex<u32>) -> Self {
        IncrVar {
            idx,
            _type: PhantomData,
        }
    }
}
impl<T> Incr for IncrVar<T>
    where T: 'static + Clone
{
    type Item = T;
    fn idx(&self) -> NodeIndex<u32> {
        self.idx
    }
    fn extract<'a, 'b>(&self, data: &'a Node, incr: &Incremental) -> Option<Self::Item> {
        if let Some(var) = data.kind.downcast_ref::<Var<T>>() {
            return var.observe(incr);
        }
        None
    }
}

#[derive(Debug, Clone)]
struct IncrMap2<A1,A2,R,F> {
    pub idx: NodeIndex<u32>,
    _arg1type: PhantomData<A1>,
    _arg2type: PhantomData<A2>,
    _rettype: PhantomData<R>,
    _fntype: PhantomData<F>,
}
impl<A1,A2,R,F> IncrMap2<A1,A2,R,F> {
    fn new(idx: NodeIndex<u32>) -> Self {
        IncrMap2 {
            idx,
            _arg1type: PhantomData,
            _arg2type: PhantomData,
            _rettype: PhantomData,
            _fntype: PhantomData,
        }
    }
}
impl<A1,A2,R,F> Incr for IncrMap2<A1,A2,R,F>
    where A1: 'static,
          A2: 'static,
          R: 'static + Clone,
          F: 'static + Fn(A1, A2)->R,
{
    type Item=R;
    fn idx(&self) -> NodeIndex<u32> {
        self.idx
    }

    fn extract<'a>(&self, data: &'a Node, incr: &Incremental) -> Option<Self::Item> {
        if let Some(map2) = data.kind.downcast_ref::<Map2<A1,A2,R,F>>() {
            return map2.observe(incr)
        } 
        None
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
struct Incremental {
    graph: Graph<Node, u32>,
    node_id_counter: Id,
    stabilization_num_counter: Id,
}

impl Incremental {
    fn new() -> Self {
        Incremental {
            graph: Graph::new(),
            node_id_counter: Id::new(),
            stabilization_num_counter: Id::new(),
        }
    }
    fn var<T:'static + Clone>(&mut self, value: T) -> IncrVar<T> {
        let kind = Box::new(Var::new(value));
        let id = self.node_id_counter.next().unwrap();
        let node = Node::new(kind, id);
        let idx = self.graph.add_node(node);
        IncrVar::new(idx)
    }
    fn map2<A1, A2, R, F>(&mut self, a: impl Incr<Item=A1> + 'static, b: impl Incr<Item=A2> + 'static, f:F) -> IncrMap2<A1,A2,R,F>
        where A1: 'static,
            A2: 'static,
            R: 'static + Clone,
            F: 'static + Fn(A1, A2)->R,
    {
        let a_idx = a.idx();
        let b_idx = b.idx();

        let kind = Box::new(Map2::new(box a, box b, f));
        let id = self.node_id_counter.next().unwrap();
        let node = Node::new(kind, id);
        let idx = self.graph.add_node(node);

        self.graph.add_edge(a_idx, idx, 1);
        self.graph.add_edge(b_idx, idx, 1);
        IncrMap2::new(idx)
    }

    fn stabilize(&mut self) -> Result<(), Cycle<NodeIndex>> {
        let current_stabilization_num = self.stabilization_num_counter.next().unwrap();
        let nodes_idx = toposort(&self.graph, None)?;
        let to_recompute = nodes_idx.iter()
            .filter_map(|&idx| {
                let node = &self.graph[idx];
                if node.stabilization_num < current_stabilization_num {
                    Some(node)
                } else {
                    None
                }
            })
            .collect::<Vec<&Node>>();
        to_recompute.iter().map(|node| node.kind.recompute());
        Ok(())
    }

    fn observe<'a, R: 'static>(&self, incr: impl Incr<Item=R> + 'a ) -> Option<R>
    {
        let node = &self.graph[incr.idx()];
        incr.extract(node, self)
    }
}

fn test_run() {
    let mut incr = Incremental::new();
    let a = incr.var(3);
    let b = incr.var(5);
    let c = incr.map2(a, b, |a:i32,b:i32| {a + b});
    let n = incr.var(1);
    let d = incr.map2(n, c, |a:i32,b:i32| {a + b});

    assert!(incr.stabilize().is_ok());
    assert_eq!(incr.observe(d), Some(5));

}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_misc() {
        test_run();
    }
}
