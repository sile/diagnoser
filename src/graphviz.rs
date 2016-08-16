use std::io;
use std::fs::File;
use std::path::Path;

use ty;

macro_rules! try_renderln{
    ($r:expr, $fmt:expr) => {
        try!(writeln!($r.writer, $fmt));
    };
    ($r:expr, $fmt:expr, $($arg:tt)*) => {
        try!(writeln!($r.writer, $fmt, $($arg)*));
    }
}

pub struct Renderer<'a, W> {
    writer: W,
    graph: &'a ty::Graph,
}
impl<'a, W> Renderer<'a, W>
    where W: io::Write
{
    pub fn new(writer: W, graph: &'a ty::Graph) -> Self {
        Renderer {
            writer: writer,
            graph: graph,
        }
    }
    pub fn render(mut self) -> io::Result<W> {
        try_renderln!(self, "digraph g {{");
        try!(self.render_nodes());
        try!(self.render_edges());
        try_renderln!(self, "}}");
        Ok(self.writer)
    }

    fn render_nodes(&mut self) -> io::Result<()> {
        for n in self.graph.nodes().values() {
            try_renderln!(self, "  {} [label={:?}];", n.id, n.ty.label());
            for (kind, c) in n.ty.get_children() {
                try_renderln!(self,
                              "  {} -> {} [label={:?}, style=dotted];",
                              n.id,
                              c,
                              kind);
            }
        }
        Ok(())
    }
    fn render_edges(&mut self) -> io::Result<()> {
        for e in self.graph.edges().values() {
            if let Some(ref label) = e.label {
                try_renderln!(self,
                              "  {} -> {} [label={:?}];",
                              e.match_from,
                              e.match_to,
                              label);
            } else {
                try_renderln!(self, "  {} -> {};", e.match_from, e.match_to);
            }
        }
        Ok(())
    }
}

pub fn for_dot_file<P: AsRef<Path>>(filepath: P, graph: &ty::Graph) -> io::Result<Renderer<File>> {
    let writer = try!(File::create(filepath));
    Ok(Renderer::new(writer, graph))
}
