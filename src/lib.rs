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
    fn observe(&self, incr: &Incremental) -> &Self::Item;
}

trait AsKind<T> {
    fn as_kind(&self) -> &Kind<Item=Option<T>>;
}

impl<T> AsKind<T> for Var<T> {
    fn as_kind(&self) -> &Kind<Item=Option<T>> {
        self
    }
}

impl<A1,A2,R,T> AsKind<R> for Map2<A1,A2,R,T>
    where for<'r, 's> T: Fn(&'r A1, &'s A2) -> R
{
    fn as_kind(&self) -> &Kind<Item=Option<R>> {
        self
    }
}

/// Variable Kind
struct Var<T> {
    value: T,
}
impl<T> Kind for Var<T> {
    type Item = Option<T>;
    fn recompute(&mut self, _incr: &Incremental) {}
    fn observe(&self, _incr: &Incremental) -> &Self::Item {
        &Some(self.value)
    }
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

struct Map2<A1, A2, R, F> 
    where A1: 'static ,
          A2: 'static ,
          R: 'static ,
          F: 'static + Fn(&A1, &A2)->R,
{
    a1: Box<Kind<Item=A1>>,
    a2: Box<Kind<Item=A2>>,
    f: Box<F>,
    r: Option<R>,
}
impl<A1, A2, R, F> Kind for Map2<A1, A2, R, F> 
    where A1: 'static ,
        A2: 'static ,
        R: 'static ,
        F: 'static + Fn(&A1, &A2)->R,
{
    type Item = Option<R>;
    fn recompute(&mut self, incr: &Incremental) {
        let a = self.a1.observe(incr);
        let b = self.a2.observe(incr);
        self.r = Some((self.f)(a, b))
    }
    fn observe(&self, _incr: &Incremental) -> &Self::Item {
        &self.r
    }
}
impl<A1, A2, R, F> Map2<A1, A2, R, F> 
    where A1: 'static,
          A2: 'static,
          R: 'static,
          F: 'static + Fn(&A1, &A2)->R,
{
    fn new(a1: Box<Kind<Item=A1>>, a2: Box<Kind<Item=A2>>, f: F) -> Self {
        let r = None;
        Self { a1: a1, a2: a2, f: box f, r }
    }
}
impl<A1, A2, R, F> Debug for Map2<A1, A2, R, F>
    where A1: 'static ,
          A2: 'static ,
          R: 'static ,
          F: 'static + Fn(&A1, &A2)->R,
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
}

impl<T> Incr for Box<Incr<Item=T>> {
    type Item = T;
    fn idx(&self) -> NodeIndex<u32> {
        self.idx()
    }
}

#[derive(Debug)]
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
impl<A1,A2,R,F> Incr for IncrMap2<A1,A2,R,F> {
    type Item=R;
    fn idx(&self) -> NodeIndex<u32> {
        self.idx
    }
}

#[derive(Debug)]
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
impl<T> Incr for IncrVar<T> {
    type Item = T;
    fn idx(&self) -> NodeIndex<u32> {
        self.idx
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
    fn var<T:'static >(&mut self, value: T) -> IncrVar<T> {
        let kind = Box::new(Var::new(Box::new(value) as Box<Any>));
        let id = self.node_id_counter.next().unwrap();
        let node = Node::new(kind, id);
        let idx = self.graph.add_node(node);
        IncrVar::new(idx)
    }
    fn map2<A1, A2, R, F>(&mut self, a: impl Incr<Item=A1> + 'static, b: impl Incr<Item=A2> + 'static, f:F) -> IncrMap2<A1,A2,R,F>
        where A1: 'static ,
            A2: 'static ,
            R: 'static,
            F: 'static + Fn(&A1, &A2)->R,
    {
        let a_idx = a.idx();
        let b_idx = b.idx();
        let a = self.graph[a_idx].kind;
        let b = self.graph[b_idx].kind;

        let kind = Box::new(Map2::new(a, b, f));
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
            .map(|&idx| &self.graph[idx])
            .filter(|node| node.stabilization_num < current_stabilization_num)
            .collect::<Vec<&Node>>();
        println!("{:?}", to_recompute);
        Ok(())
    }

    fn observe<A1,A2,R,F>(&self, incr: &'static impl Incr ) -> Result<&Option<R>, ()>
        where A1: 'static ,
            A2: 'static ,
            R: 'static ,
            F: 'static + Fn(&A1, &A2)->R,
    {
        let node = &self.graph[incr.idx()];
        if let Some(var) = node.kind.downcast_ref::<Var<R>>() {
            return Ok(var.observe(&self));
        }

        if let Some(fun) = (incr as &Any).downcast_ref::<IncrMap2<A1,A2,R,F>>() {
            if let Some(map2) = node.kind.downcast_ref::<Map2<A1,A2,R,F>>() {
                return Ok(map2.observe(&self))
            } 
        }

        Err(())
        
    }
}

fn test_run() {
    let mut incr = Incremental::new();
    let a = incr.var(3);
    let b = incr.var(5);
    let c = incr.map2(a, b, |a:&i32,b:&i32| {a + b});
    let n = incr.var(1);
    let d = incr.map2(n, c, |a:&i32,b:&i32| {a + b});

    assert!(incr.stabilize().is_ok());
    // assert_eq!(incr.observe(&c).unwrap(), 5);

}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_misc() {
        test_run();
    }
}