#![feature(conservative_impl_trait)]
extern crate daggy;
use daggy::Dag;
use daggy::NodeIndex;
use std::rc::Rc;
use std::cell::RefCell;

/// Kind
trait Kind {
}

/// Variable Kind
struct Var<T> {
    value: T,
    set_at: usize,
}
impl<'a, T> Kind for Var<T> {
}
impl<T> Var<T> {
    fn new(value: T, set_at: usize) -> Self {
        Var {
            value,
            set_at,
        }
    }
}

struct Map2<A1, A2, F> {
    a1: A1,
    a2: A2,
    f: F
}
impl<A1, A2, F> Kind for Map2<A1, A2, F> {}
impl<A1, A2, F> Map2<A1, A2, F> {
    fn new(a1: A1, a2: A2, f: F) -> Self {
        Self { a1, a2, f }
    }
}

//--------------------------------------------------------

/// Node
struct Node<'a> {
    kind: Box<Kind + 'a>,
    height: i32,
}

impl<'a> Node<'a> {
    fn new(kind: Box<Kind + 'a>) -> Self {
        Node {
            kind,
            height: -1,
        }
    }
}

struct Incr {
    pub idx: NodeIndex<usize>,
}

impl Incr {
    fn new(idx: NodeIndex<usize>) -> Self {
        Incr {
            idx,
        }
    }
}

struct Id {id: usize}
impl Iterator for Id {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        let ret = self.id;
        self.id += 1;
        Some(ret)
    }
}
impl Id {
    fn new()->Self{Self{id:0}}
    fn none()->usize{0}
}


/// Incremental
struct Incremental<'a> {
    graph: Dag<Node<'a>, u32, usize>,
    node_id_counter: Id,
    stabilization_num_counter: Id,
    stabilization_num: usize,
}

impl<'a> Incremental<'a> {
    fn new() -> Self {
        Incremental {
            graph: Dag::new(),
            node_id_counter: Id::new(),
            stabilization_num_counter: Id::new(),
            stabilization_num: 0,
        }
    }
    fn var<T: 'a>(&mut self, value: T) -> Node<'a> {
        let node = Node::new(Box::new(Var::new(value, self.stabilization_num)));
        let idx = self.graph.add_node(node);
        self.graph[idx]
    }
    fn map2<F:'a, A1, A2, R>(&mut self, a: Node<'a>, b: Node<'a>, f: F) -> Node<'a>
        where F: Fn(A1, A2) -> R
    {
        let node = Node::new(Box::new(Map2::new(a, b, f)));
        let idx = self.graph.add_node(node);
        self.graph[idx]
    }
}

fn test() {
    let mut incr = Incremental::new();
    let a = incr.var(3);
    let b = incr.var(5);
    let c = incr.map2(a, b, |a:i32,b:i32| a + b);
    
}