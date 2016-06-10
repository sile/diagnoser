use std::io;
use std::io::Write;
use graph;

pub struct DotWriter<W> {
    w: W,
}
impl<W> DotWriter<W>
    where W: Write
{
    pub fn new(w: W) -> Self {
        DotWriter { w: w }
    }

    pub fn write(&mut self, graph: &graph::Graph) -> io::Result<()> {
        try!(write!(self.w, "digraph g {{\n"));
        for node in graph.nodes.values() {
            try!(write!(self.w, "{} [label={:?}];\n", node.id, node.label()));
        }
        for node in graph.nodes.values() {
            for (kind, id) in node.content.link_nodes() {
                try!(write!(self.w,
                            "{} -> {} [label={:?}];\n",
                            id,
                            node.id,
                            kind.label()));
            }
        }
        for edge in graph.edges.values() {
            try!(write!(self.w,
                        "{} -> {} [label={:?}];\n",
                        edge.producer,
                        edge.consumer,
                        edge.kind.label()));
        }
        try!(write!(self.w, "}}\n"));
        Ok(())
    }
}
