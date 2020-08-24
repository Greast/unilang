use core::iter::FromIterator;
use std::collections::{BTreeSet, BTreeMap, LinkedList};
use std::ptr::null_mut;

#[derive(Debug, PartialEq, Eq)]
struct Node<T> {
    empty: BTreeSet<*const Self>,
    keys: BTreeMap<T, BTreeSet<*const Self>>,
}

impl<T> Node<T> {
    fn remap(&mut self, map: &BTreeMap<*const Self, *const Self>) {
        let closure = |ptr: *const Self| *map.get(&ptr).unwrap_or(&ptr);

        self.empty = self.empty.iter().map(Deref::deref).map(closure).collect();
        self.keys.iter_mut().for_each(|(_, set)| {
            set = set.drain().map(closure).collect();
        });
    }
}

impl<T> Default for Node<T>
where
    T: Ord,
{
    fn default() -> Self {
        Node {
            empty: Vec::new(),
            keys: BTreeMap::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct NFA<T> {
    graph: LinkedList<Node<T>>,
}

impl<T> Clone for NFA<T>
where
    T: Ord + Clone,
{
    fn clone(&self) -> Self {
        let mut graph = self
            .graph
            .iter()
            .map(|node| Node {
                empty: node.empty.clone(),
                keys: node.keys.clone(),
            })
            .collect::<LinkedList<_>>();

        let table = self
            .graph
            .iter()
            .map(|x| x as *const _)
            .zip(graph.iter().map(|x| x as *const _))
            .collect::<BTreeMap<_, _>>();

        graph.iter_mut().for_each(move |node| {
            node.remap(&table);
        });

        Self { graph }
    }
}

impl<T> Default for NFA<T>
where
    T: Ord,
{
    fn default() -> Self {
        let mut nfa = Self {
            graph: LinkedList::default(),
        };
        let node = Node::default();
        nfa.graph.push_back(node);
        nfa
    }
}

impl<T> FromIterator<T> for NFA<T>
where
    T: Ord,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        iter.into_iter().fold(Self::default(), Self::append)
    }
}

//Wrappers for the start and accept state.
impl<T> NFA<T> {
    pub fn start(&self) -> &Node<T> {
        self.graph.front().unwrap()
    }
    pub fn start_mut(&mut self) -> &mut Node<T> {
        self.graph.front_mut().unwrap()
    }
    pub fn accept(&self) -> &Node<T> {
        self.graph.back().unwrap()
    }
    pub fn accept_mut(&mut self) -> &mut Node<T> {
        self.graph.back_mut().unwrap()
    }
}

//Essential nfa functions.
impl<T> NFA<T>
where
    T: Ord,
{
    fn append(mut self, value: T) -> Self {
        let node = Node::default();
        let old = self.accept_mut() as *mut Node<T>;
        self.graph.push_back(node);
        let ptr = self.accept_mut() as *mut _;
        unsafe {
            (*old).keys.entry(value).or_default().push(ptr);
        }
        self
    }

    pub fn plus(mut self) -> Self {
        let ptr = self.start() as *const _;
        self.accept_mut().empty.push(ptr);
        self
    }

    pub fn or(mut self, mut rhs: Self) -> Self {
        let mut graph = LinkedList::default();
        //Place the new start node at the start of the linked list...
        graph.push_back(Node {
            empty: vec![self.start(), rhs.start()],
            keys: Default::default(),
        });
        graph.append(&mut self.graph);
        graph.append(&mut rhs.graph);
        //And the accepting node at the end.
        graph.push_back(Node {
            empty: vec![self.accept(), rhs.accept()],
            keys: Default::default(),
        });

        NFA { graph }
    }

    pub fn and(mut self, mut rhs: Self) -> Self {
        self.accept_mut().empty.push(rhs.start());
        self.graph.append(&mut rhs.graph);
        self
    }

    pub fn optional(mut self) -> Self {
        let ptr = self.accept() as *const _;
        self.start_mut().empty.push(ptr);
        self
    }
}

impl<T> NFA<T>
where
    T: Ord + Clone,
{
    pub fn star(mut self) -> Self {
        self.plus().optional()
    }

    pub fn range(mut self, from: usize, to: Option<usize>) -> Self {
        let head = (0..from)
            .into_iter()
            .map(|_| self.clone())
            .fold(self.clone(), Self::and);
        let tail = if let Some(x) = to {
            (from..x)
                .into_iter()
                .map(|_| self.clone())
                .map(Self::optional)
                .fold(self.clone(), Self::and)
        } else {
            self.clone().star()
        };
        head.and(tail)
    }
}
