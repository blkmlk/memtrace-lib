use crate::trace::Trace;

struct TraceNode {
    ip: usize,
    index: usize,
    children: Vec<TraceNode>,
}

impl TraceNode {
    pub fn new(index: usize, ip: usize) -> Self {
        Self {
            ip,
            index,
            children: Vec::new(),
        }
    }

    fn index(
        &mut self,
        mut it: impl Iterator<Item = usize>,
        on_new: &mut impl FnMut(usize, usize),
        next_idx: &mut usize,
    ) -> usize {
        let Some(ip) = it.next() else {
            return self.index;
        };

        match self.children.iter_mut().find(|c| c.ip == ip) {
            None => {
                self.children.push(TraceNode::new(*next_idx, ip));
                on_new(ip, self.index);

                *next_idx += 1;

                self.children
                    .last_mut()
                    .unwrap()
                    .index(it, on_new, next_idx)
            }
            Some(c) => c.index(it, on_new, next_idx),
        }
    }
}

pub struct TraceTree {
    root: TraceNode,
    last_index: usize,
}

impl TraceTree {
    pub fn new() -> Self {
        Self {
            root: TraceNode::new(0, 0),
            last_index: 1,
        }
    }

    pub fn index(&mut self, trace: Trace, mut on_new: impl FnMut(usize, usize)) -> usize {
        let it = trace.as_slice().iter().copied();
        let idx = self.root.index(it, &mut on_new, &mut self.last_index);

        idx
    }
}
