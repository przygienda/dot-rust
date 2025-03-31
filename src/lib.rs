// Copyright 2014-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Generate files suitable for use with [Graphviz](https://graphviz.org/)
//!
//! The `render` function generates output (e.g. an `output.dot` file) for
//! use with [Graphviz](https://graphviz.org/) by walking a labelled
//! graph. (Graphviz can then automatically lay out the nodes and edges
//! of the graph, and also optionally render the graph as an image or
//! other [output formats](https://graphviz.org/docs/outputs), such as SVG.)
//!
//! Rather than impose some particular graph data structure on clients,
//! this library exposes two traits that clients can implement on their
//! own structs before handing them over to the rendering function.
//!
//! Note: This library does not yet provide access to the full
//! expressiveness of the [DOT language](https://graphviz.org/doc/info/lang.html).
//! For example, there are many [attributes](https://graphviz.org/doc/info/attrs.html)
//! related to providing layout hints (e.g. left-to-right versus top-down, which
//! algorithm to use, etc). The current intention of this library is to
//! emit a human-readable .dot file with very regular structure suitable
//! for easy post-processing.
//!
//! # Examples
//!
//! The first example uses a very simple graph representation: a list of
//! pairs of ints, representing the edges (the node set is implicit).
//! Each node label is derived directly from the int representing the node,
//! while the edge labels are all empty strings.
//!
//! This example also illustrates how to use `Cow<[T]>` to return
//! an owned vector or a borrowed slice as appropriate: we construct the
//! node vector from scratch, but borrow the edge list (rather than
//! constructing a copy of all the edges from scratch).
//!
//! The output from this example renders five nodes, with the first four
//! forming a diamond-shaped acyclic graph and then pointing to the fifth
//! which is cyclic.
//!
//! ```rust
//! use std::borrow::Cow;
//! use std::io::Write;
//!
//! type Nd = isize;
//! type Ed = (isize,isize);
//! struct Edges(Vec<Ed>);
//!
//! pub fn render_to<W: Write>(output: &mut W) {
//!     let edges = Edges(vec!((0,1), (0,2), (1,3), (2,3), (3,4), (4,4)));
//!     dot::render(&edges, output).unwrap()
//! }
//!
//! impl<'a> dot::Labeller<'a, Nd, Ed> for Edges {
//!     fn graph_id(&'a self) -> dot::Id<'a> { dot::Id::new("example1").unwrap() }
//!
//!     fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
//!         dot::Id::new(format!("N{}", *n)).unwrap()
//!     }
//! }
//!
//! impl<'a> dot::GraphWalk<'a, Nd, Ed> for Edges {
//!     fn nodes(&self) -> dot::Nodes<'a,Nd> {
//!         // (assumes that |N| \approxeq |E|)
//!         let &Edges(ref v) = self;
//!         let mut nodes = Vec::with_capacity(v.len());
//!         for &(s,t) in v {
//!             nodes.push(s); nodes.push(t);
//!         }
//!         nodes.sort();
//!         nodes.dedup();
//!         Cow::Owned(nodes)
//!     }
//!
//!     fn edges(&'a self) -> dot::Edges<'a,Ed> {
//!         let &Edges(ref edges) = self;
//!         Cow::Borrowed(&edges[..])
//!     }
//!
//!     fn source(&self, e: &Ed) -> Nd { e.0 }
//!
//!     fn target(&self, e: &Ed) -> Nd { e.1 }
//! }
//!
//! # pub fn main() { render_to(&mut Vec::new()) }
//! ```
//!
//! ```no_run
//! # pub fn render_to<W:std::io::Write>(output: &mut W) { unimplemented!() }
//! pub fn main() {
//!     use std::fs::File;
//!     let mut f = File::create("example1.dot").unwrap();
//!     render_to(&mut f)
//! }
//! ```
//!
//! Output from first example (in `example1.dot`):
//!
//! ```ignore
//! digraph example1 {
//!     N0[label="N0"];
//!     N1[label="N1"];
//!     N2[label="N2"];
//!     N3[label="N3"];
//!     N4[label="N4"];
//!     N0 -> N1[label=""];
//!     N0 -> N2[label=""];
//!     N1 -> N3[label=""];
//!     N2 -> N3[label=""];
//!     N3 -> N4[label=""];
//!     N4 -> N4[label=""];
//! }
//! ```
//!
//! The second example illustrates using `node_label` and `edge_label` to
//! add labels to the nodes and edges in the rendered graph. The graph
//! here carries both `nodes` (the label text to use for rendering a
//! particular node), and `edges` (again a list of `(source,target)`
//! indices).
//!
//! This example also illustrates how to use a type (in this case the edge
//! type) that shares substructure with the graph: the edge type here is a
//! direct reference to the `(source,target)` pair stored in the graph's
//! internal vector (rather than passing around a copy of the pair
//! itself). Note that this implies that `fn edges(&'a self)` must
//! construct a fresh `Vec<&'a (usize,usize)>` from the `Vec<(usize,usize)>`
//! edges stored in `self`.
//!
//! Since both the set of nodes and the set of edges are always
//! constructed from scratch via iterators, we use the `collect()` method
//! from the `Iterator` trait to collect the nodes and edges into freshly
//! constructed growable `Vec` values (rather use the `into`
//! from the `IntoCow` trait as was used in the first example
//! above).
//!
//! The output from this example renders four nodes that make up the
//! Hasse-diagram for the subsets of the set `{x, y}`. Each edge is
//! labelled with the &sube; character (specified using the HTML character
//! entity `&sube`).
//!
//! ```rust
//! use std::io::Write;
//!
//! type Nd = usize;
//! type Ed<'a> = &'a (usize, usize);
//! struct Graph { nodes: Vec<&'static str>, edges: Vec<(usize,usize)> }
//!
//! pub fn render_to<W: Write>(output: &mut W) {
//!     let nodes = vec!("{x,y}","{x}","{y}","{}");
//!     let edges = vec!((0,1), (0,2), (1,3), (2,3));
//!     let graph = Graph { nodes: nodes, edges: edges };
//!
//!     dot::render(&graph, output).unwrap()
//! }
//!
//! impl<'a> dot::Labeller<'a, Nd, Ed<'a>> for Graph {
//!     fn graph_id(&'a self) -> dot::Id<'a> { dot::Id::new("example2").unwrap() }
//!     fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
//!         dot::Id::new(format!("N{}", n)).unwrap()
//!     }
//!     fn node_label<'b>(&'b self, n: &Nd) -> dot::LabelText<'b> {
//!         dot::LabelText::LabelStr(self.nodes[*n].into())
//!     }
//!     fn edge_label<'b>(&'b self, _: &Ed) -> dot::LabelText<'b> {
//!         dot::LabelText::LabelStr("&sube;".into())
//!     }
//! }
//!
//! impl<'a> dot::GraphWalk<'a, Nd, Ed<'a>> for Graph {
//!     fn nodes(&self) -> dot::Nodes<'a,Nd> { (0..self.nodes.len()).collect() }
//!     fn edges(&'a self) -> dot::Edges<'a,Ed<'a>> { self.edges.iter().collect() }
//!     fn source(&self, e: &Ed) -> Nd { e.0 }
//!     fn target(&self, e: &Ed) -> Nd { e.1 }
//! }
//!
//! # pub fn main() { render_to(&mut Vec::new()) }
//! ```
//!
//! ```no_run
//! # pub fn render_to<W:std::io::Write>(output: &mut W) { unimplemented!() }
//! pub fn main() {
//!     use std::fs::File;
//!     let mut f = File::create("example2.dot").unwrap();
//!     render_to(&mut f)
//! }
//! ```
//!
//! The third example is similar to the second, except now each node and
//! edge now carries a reference to the string label for each node as well
//! as that node's index. (This is another illustration of how to share
//! structure with the graph itself, and why one might want to do so.)
//!
//! The output from this example is the same as the second example: the
//! Hasse-diagram for the subsets of the set `{x, y}`.
//!
//! ```rust
//! use std::io::Write;
//!
//! type Nd<'a> = (usize, &'a str);
//! type Ed<'a> = (Nd<'a>, Nd<'a>);
//! struct Graph { nodes: Vec<&'static str>, edges: Vec<(usize,usize)> }
//!
//! pub fn render_to<W: Write>(output: &mut W) {
//!     let nodes = vec!("{x,y}","{x}","{y}","{}");
//!     let edges = vec!((0,1), (0,2), (1,3), (2,3));
//!     let graph = Graph { nodes: nodes, edges: edges };
//!
//!     dot::render(&graph, output).unwrap()
//! }
//!
//! impl<'a> dot::Labeller<'a, Nd<'a>, Ed<'a>> for Graph {
//!     fn graph_id(&'a self) -> dot::Id<'a> { dot::Id::new("example3").unwrap() }
//!     fn node_id(&'a self, n: &Nd<'a>) -> dot::Id<'a> {
//!         dot::Id::new(format!("N{}", n.0)).unwrap()
//!     }
//!     fn node_label<'b>(&'b self, n: &Nd<'b>) -> dot::LabelText<'b> {
//!         let &(i, _) = n;
//!         dot::LabelText::LabelStr(self.nodes[i].into())
//!     }
//!     fn edge_label<'b>(&'b self, _: &Ed<'b>) -> dot::LabelText<'b> {
//!         dot::LabelText::LabelStr("&sube;".into())
//!     }
//! }
//!
//! impl<'a> dot::GraphWalk<'a, Nd<'a>, Ed<'a>> for Graph {
//!     fn nodes(&'a self) -> dot::Nodes<'a,Nd<'a>> {
//!         self.nodes.iter().map(|s| &s[..]).enumerate().collect()
//!     }
//!     fn edges(&'a self) -> dot::Edges<'a,Ed<'a>> {
//!         self.edges.iter()
//!             .map(|&(i,j)|((i, &self.nodes[i][..]),
//!                           (j, &self.nodes[j][..])))
//!             .collect()
//!     }
//!     fn source(&self, e: &Ed<'a>) -> Nd<'a> { e.0 }
//!     fn target(&self, e: &Ed<'a>) -> Nd<'a> { e.1 }
//! }
//!
//! # pub fn main() { render_to(&mut Vec::new()) }
//! ```
//!
//! ```no_run
//! # pub fn render_to<W:std::io::Write>(output: &mut W) { unimplemented!() }
//! pub fn main() {
//!     use std::fs::File;
//!     let mut f = File::create("example3.dot").unwrap();
//!     render_to(&mut f)
//! }
//! ```
//!
//! # References
//!
//! * [Graphviz](https://graphviz.org/)
//!
//! * [DOT language](https://graphviz.org/doc/info/lang.html)

#![crate_name = "dot"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]
#![doc(html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
       html_favicon_url = "https://doc.rust-lang.org/favicon.ico",
       html_root_url = "https://doc.rust-lang.org/nightly/")]

use self::LabelText::*;

use std::borrow::Cow;
use std::io::prelude::*;
use std::io;
use std::str;
use std::collections::HashMap;

/// The text for a graphviz label on a node or edge.
pub enum LabelText<'a> {
    /// This kind of label preserves the text directly as is.
    ///
    /// Occurrences of backslashes (`\`) are escaped, and thus appear
    /// as backslashes in the rendered label.
    LabelStr(Cow<'a, str>),

    /// This kind of label uses the graphviz label escString type:
    /// https://graphviz.org/docs/attr-types/escString
    ///
    /// Occurrences of backslashes (`\`) are not escaped; instead they
    /// are interpreted as initiating an escString escape sequence.
    ///
    /// Escape sequences of particular interest: in addition to `\n`
    /// to break a line (centering the line preceding the `\n`), there
    /// are also the escape sequences `\l` which left-justifies the
    /// preceding line and `\r` which right-justifies it.
    EscStr(Cow<'a, str>),

    /// This uses a graphviz [HTML string label][html]. The string is
    /// printed exactly as given, but between `<` and `>`. **No
    /// escaping is performed.**
    ///
    /// [html]: https://graphviz.org/doc/info/shapes.html#html
    HtmlStr(Cow<'a, str>),
}

/// The style for a node or edge.
/// See https://graphviz.org/doc/info/attrs.html#k:style for descriptions.
/// Note that some of these are not valid for edges.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Style {
    None,
    Solid,
    Dashed,
    Dotted,
    Bold,
    Rounded,
    Diagonals,
    Filled,
    Striped,
    Wedged,
}

impl Style {
    pub fn as_slice(self) -> &'static str {
        match self {
            Style::None => "",
            Style::Solid => "solid",
            Style::Dashed => "dashed",
            Style::Dotted => "dotted",
            Style::Bold => "bold",
            Style::Rounded => "rounded",
            Style::Diagonals => "diagonals",
            Style::Filled => "filled",
            Style::Striped => "striped",
            Style::Wedged => "wedged",
        }
    }
}


/// The direction to draw directed graphs (one rank at a time)
/// See https://graphviz.org/docs/attr-types/rankdir/ for descriptions
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RankDir {
    TopBottom,
    LeftRight,
    BottomTop,
    RightLeft,
}

impl RankDir {
    pub fn as_slice(self) -> &'static str {
        match self {
            RankDir::TopBottom => "TB",
            RankDir::LeftRight => "LR",
            RankDir::BottomTop => "BT",
            RankDir::RightLeft => "RL",
        }
    }
}

// There is a tension in the design of the labelling API.
//
// For example, I considered making a `Labeller<T>` trait that
// provides labels for `T`, and then making the graph type `G`
// implement `Labeller<Node>` and `Labeller<Edge>`. However, this is
// not possible without functional dependencies. (One could work
// around that, but I did not explore that avenue heavily.)
//
// Another approach that I actually used for a while was to make a
// `Label<Context>` trait that is implemented by the client-specific
// Node and Edge types (as well as an implementation on Graph itself
// for the overall name for the graph). The main disadvantage of this
// second approach (compared to having the `G` type parameter
// implement a Labelling service) that I have encountered is that it
// makes it impossible to use types outside of the current crate
// directly as Nodes/Edges; you need to wrap them in newtype'd
// structs. See e.g. the `No` and `Ed` structs in the examples. (In
// practice clients using a graph in some other crate would need to
// provide some sort of adapter shim over the graph anyway to
// interface with this library).
//
// Another approach would be to make a single `Labeller<N,E>` trait
// that provides three methods (graph_label, node_label, edge_label),
// and then make `G` implement `Labeller<N,E>`. At first this did not
// appeal to me, since I had thought I would need separate methods on
// each data variant for dot-internal identifiers versus user-visible
// labels. However, the identifier/label distinction only arises for
// nodes; graphs themselves only have identifiers, and edges only have
// labels.
//
// So in the end I decided to use the third approach described above.

/// `Id` is a Graphviz `ID`.
pub struct Id<'a> {
    name: Cow<'a, str>,
}

impl<'a> Id<'a> {
    /// Creates an `Id` named `name`.
    ///
    /// The caller must ensure that the input conforms to an
    /// identifier format: it must be a non-empty string made up of
    /// alphanumeric or underscore characters, not beginning with a
    /// digit (i.e. the regular expression `[a-zA-Z_][a-zA-Z_0-9]*`).
    ///
    /// (Note: this format is a strict subset of the `ID` format
    /// defined by the DOT language.  This function may change in the
    /// future to accept a broader subset, or the entirety, of DOT's
    /// `ID` format.)
    ///
    /// Passing an invalid string (containing spaces, brackets,
    /// quotes, ...) will return an empty `Err` value.
    pub fn new<Name: Into<Cow<'a, str>>>(name: Name) -> Result<Id<'a>, ()> {
        let name = name.into();
        {
            let mut chars = name.chars();
            match chars.next() {
                Some(c) if is_letter_or_underscore(c) => {}
                _ => return Err(()),
            }
            if !chars.all(is_constituent) {
                return Err(())
            }
        }
        return Ok(Id{ name: name });

        fn is_letter_or_underscore(c: char) -> bool {
            in_range('a', c, 'z') || in_range('A', c, 'Z') || c == '_'
        }
        fn is_constituent(c: char) -> bool {
            is_letter_or_underscore(c) || in_range('0', c, '9')
        }
        fn in_range(low: char, c: char, high: char) -> bool {
            low as usize <= c as usize && c as usize <= high as usize
        }
    }

    pub fn as_slice(&'a self) -> &'a str {
        &*self.name
    }

    pub fn name(self) -> Cow<'a, str> {
        self.name
    }
}

/// Each instance of a type that implements `Label<C>` maps to a
/// unique identifier with respect to `C`, which is used to identify
/// it in the generated .dot file. They can also provide more
/// elaborate (and non-unique) label text that is used in the graphviz
/// rendered output.

/// The graph instance is responsible for providing the DOT compatible
/// identifiers for the nodes and (optionally) rendered labels for the nodes and
/// edges, as well as an identifier for the graph itself.
pub trait Labeller<'a,N,E> {
    /// Must return a DOT compatible identifier naming the graph.
    fn graph_id(&'a self) -> Id<'a>;

    /// A list of attributes to apply to the graph
    fn graph_attrs(&'a self) -> HashMap<&str, &str> {
        HashMap::default()
    }

    /// Maps `n` to a unique identifier with respect to `self`. The
    /// implementer is responsible for ensuring that the returned name
    /// is a valid DOT identifier.
    fn node_id(&'a self, n: &N) -> Id<'a>;

    /// Maps `n` to one of the [graphviz `shape` names][1]. If `None`
    /// is returned, no `shape` attribute is specified.
    ///
    /// [1]: https://graphviz.org/doc/info/shapes.html
    fn node_shape(&'a self, _node: &N) -> Option<LabelText<'a>> {
        None
    }

    /// Maps `n` to a label that will be used in the rendered output.
    /// The label need not be unique, and may be the empty string; the
    /// default is just the output from `node_id`.
    fn node_label(&'a self, n: &N) -> LabelText<'a> {
        LabelStr(self.node_id(n).name())
    }

    /// Maps `e` to a label that will be used in the rendered output.
    /// The label need not be unique, and may be the empty string; the
    /// default is in fact the empty string.
    fn edge_label(&'a self, e: &E) -> LabelText<'a> {
        let _ignored = e;
        LabelStr("".into())
    }

    /// Maps `n` to a style that will be used in the rendered output.
    fn node_style(&'a self, _n: &N) -> Style {
        Style::None
    }

    /// Return an explicit rank dir to use for directed graphs.
    ///
    /// Return 'None' to use the default (generally "TB" for directed graphs).
    fn rank_dir(&'a self) -> Option<RankDir> {
        None
    }

    /// Maps `n` to one of the [graphviz `color` names][1]. If `None`
    /// is returned, no `color` attribute is specified.
    ///
    /// [1]: https://graphviz.gitlab.io/_pages/doc/info/colors.html
    fn node_color(&'a self, _node: &N) -> Option<LabelText<'a>> {
        None
    }

    /// Maps `n` to a set of arbritrary node attributes.
    fn node_attrs(&'a self, _n: &N) -> HashMap<&str, &str> {
        HashMap::default()
    }

    /// Maps `e` to arrow style that will be used on the end of an edge.
    /// Defaults to default arrow style.
    fn edge_end_arrow(&'a self, _e: &E) -> Arrow {
        Arrow::default()
    }

    /// Maps `e` to arrow style that will be used on the end of an edge.
    /// Defaults to default arrow style.
    fn edge_start_arrow(&'a self, _e: &E) -> Arrow {
        Arrow::default()
    }

    /// Maps `e` to a style that will be used in the rendered output.
    fn edge_style(&'a self, _e: &E) -> Style {
        Style::None
    }

    /// Maps `e` to one of the [graphviz `color` names][1]. If `None`
    /// is returned, no `color` attribute is specified.
    ///
    /// [1]: https://graphviz.gitlab.io/_pages/doc/info/colors.html
    fn edge_color(&'a self, _e: &E) -> Option<LabelText<'a>> {
        None
    }

    /// Maps `e` to a set of arbritrary edge attributes.
    fn edge_attrs(&'a self, _e: &E) -> HashMap<&str, &str> {
        HashMap::default()
    }

    /// The kind of graph, defaults to `Kind::Digraph`.
    #[inline]
    fn kind(&self) -> Kind {
        Kind::Digraph
    }
}

/// Escape tags in such a way that it is suitable for inclusion in a
/// Graphviz HTML label.
pub fn escape_html(s: &str) -> String {
    s
        .replace("&", "&amp;")
        .replace("\"", "&quot;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
}

impl<'a> LabelText<'a> {
    pub fn label<S:Into<Cow<'a, str>>>(s: S) -> LabelText<'a> {
        LabelStr(s.into())
    }

    pub fn escaped<S:Into<Cow<'a, str>>>(s: S) -> LabelText<'a> {
        EscStr(s.into())
    }

    pub fn html<S: Into<Cow<'a, str>>>(s: S) -> LabelText<'a> {
        HtmlStr(s.into())
    }

    fn escape_ascii_char(c: char) -> String {
        if c.is_ascii() || c.is_control() || c.is_whitespace() {
            c.escape_default().to_string()
        } else {
            String::from(c)
        }
    }

    fn escape_char<F>(c: char, mut f: F)
        where F: FnMut(char)
    {
        match c {
            // not escaping \\, since Graphviz escString needs to
            // interpret backslashes; see EscStr above.
            '\\' => f(c),
            _ => for c in Self::escape_ascii_char(c).chars() {
                f(c)
            },
        }
    }
    fn escape_str(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for c in s.chars() {
            LabelText::escape_char(c, |c| out.push(c));
        }
        out
    }

    fn escape_default(s: &str) -> String {
        let mut buf = String::new();
        for c in s.chars() {
            buf.push_str(Self::escape_ascii_char(c).as_str());
        }
        buf
    }

    /// Renders text as string suitable for a label in a .dot file.
    /// This includes quotes or suitable delimeters.
    pub fn to_dot_string(&self) -> String {
        match self {
            &LabelStr(ref s) => format!("\"{}\"", LabelText::escape_default(s)),
            &EscStr(ref s) => format!("\"{}\"", LabelText::escape_str(&s[..])),
            &HtmlStr(ref s) => format!("<{}>", s),
        }
    }

    /// Decomposes content into string suitable for making EscStr that
    /// yields same content as self.  The result obeys the law
    /// render(`lt`) == render(`EscStr(lt.pre_escaped_content())`) for
    /// all `lt: LabelText`.
    fn pre_escaped_content(self) -> Cow<'a, str> {
        match self {
            EscStr(s) => s,
            LabelStr(s) => if s.contains('\\') {
                LabelText::escape_default(&*s).into()
            } else {
                s
            },
            HtmlStr(s) => s,
        }
    }

    /// Puts `prefix` on a line above this label, with a blank line separator.
    pub fn prefix_line(self, prefix: LabelText) -> LabelText<'static> {
        prefix.suffix_line(self)
    }

    /// Puts `suffix` on a line below this label, with a blank line separator.
    pub fn suffix_line(self, suffix: LabelText) -> LabelText<'static> {
        let mut prefix = self.pre_escaped_content().into_owned();
        let suffix = suffix.pre_escaped_content();
        prefix.push_str(r"\n\n");
        prefix.push_str(&suffix[..]);
        EscStr(prefix.into())
    }
}


/// This structure holds all information that can describe an arrow connected to
/// either start or end of an edge.
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Arrow {
    pub arrows: Vec<ArrowShape>,
}

use self::ArrowShape::*;

impl Arrow {
    /// Return `true` if this is a default arrow.
    fn is_default(&self) -> bool {
        self.arrows.is_empty()
    }

    /// Arrow constructor which returns a default arrow
    pub fn default() -> Arrow {
        Arrow {
            arrows: vec![],
        }
    }

    /// Arrow constructor which returns an empty arrow
    pub fn none() -> Arrow {
        Arrow {
            arrows: vec![NoArrow],
        }
    }

    /// Arrow constructor which returns a regular triangle arrow, without modifiers
    pub fn normal() -> Arrow {
        Arrow {
            arrows: vec![ArrowShape::normal()]
        }
    }

    /// Arrow constructor which returns an arrow created by a given ArrowShape.
    pub fn from_arrow(arrow: ArrowShape) -> Arrow {
        Arrow {
            arrows: vec![arrow],
        }
    }

    /// Function which converts given arrow into a renderable form.
    pub fn to_dot_string(&self) -> String {
        let mut cow = String::new();
        for arrow in &self.arrows {
            cow.push_str(&arrow.to_dot_string());
        };
        cow
    }
}


impl Into<Arrow> for [ArrowShape; 2] {
    fn into(self) -> Arrow {
        Arrow {
            arrows: vec![self[0], self[1]],
        }
    }
}
impl Into<Arrow> for [ArrowShape; 3] {
    fn into(self) -> Arrow {
        Arrow {
            arrows: vec![self[0], self[1], self[2]],
        }
    }
}
impl Into<Arrow> for [ArrowShape; 4] {
    fn into(self) -> Arrow {
        Arrow {
            arrows: vec![self[0], self[1], self[2], self[3]],
        }
    }
}

/// Arrow modifier that determines if the shape is empty or filled.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum Fill {
    Open,
    Filled,
}

impl Fill {
    pub fn as_slice(self) -> &'static str {
        match self {
            Fill::Open => "o",
            Fill::Filled => "",
        }
    }
}

/// Arrow modifier that determines if the shape is clipped.
/// For example `Side::Left` means only left side is visible.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
    Both,
}

impl Side {
    pub fn as_slice(self) -> &'static str {
        match self {
            Side::Left  => "l",
            Side::Right => "r",
            Side::Both  => "",
        }
    }
}


/// This enumeration represents all possible arrow edge
/// as defined in [graphviz documentation](https://graphviz.org/doc/info/arrows.html).
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum ArrowShape {
    /// No arrow will be displayed
    NoArrow,
    /// Arrow that ends in a triangle. Basically a normal arrow.
    /// NOTE: there is error in official documentation, this supports both fill and side clipping
    Normal(Fill, Side),
    /// Arrow ending in a small square box
    Box(Fill, Side),
    /// Arrow ending in a three branching lines also called crow's foot
    Crow(Side),
    /// Arrow ending in a curve
    Curve(Side),
    /// Arrow ending in an inverted curve
    ICurve(Fill, Side),
    /// Arrow ending in an diamond shaped rectangular shape.
    Diamond(Fill, Side),
    /// Arrow ending in a circle.
    Dot(Fill),
    /// Arrow ending in an inverted triangle.
    Inv(Fill, Side),
    /// Arrow ending with a T shaped arrow.
    Tee(Side),
    /// Arrow ending with a V shaped arrow.
    Vee(Side),
}
impl ArrowShape {
    /// Constructor which returns no arrow.
    pub fn none() -> ArrowShape {
        ArrowShape::NoArrow
    }

    /// Constructor which returns normal arrow.
    pub fn normal() -> ArrowShape {
        ArrowShape::Normal(Fill::Filled, Side::Both)
    }

    /// Constructor which returns a regular box arrow.
    pub fn boxed() -> ArrowShape {
        ArrowShape::Box(Fill::Filled, Side::Both)
    }

    /// Constructor which returns a regular crow arrow.
    pub fn crow() -> ArrowShape {
        ArrowShape::Crow(Side::Both)
    }

    /// Constructor which returns a regular curve arrow.
    pub fn curve() -> ArrowShape {
        ArrowShape::Curve(Side::Both)
    }

    /// Constructor which returns an inverted curve arrow.
    pub fn icurve() -> ArrowShape {
        ArrowShape::ICurve(Fill::Filled, Side::Both)
    }

    /// Constructor which returns a diamond arrow.
    pub fn diamond() -> ArrowShape {
        ArrowShape::Diamond(Fill::Filled, Side::Both)
    }

    /// Constructor which returns a circle shaped arrow.
    pub fn dot() -> ArrowShape {
        ArrowShape::Diamond(Fill::Filled, Side::Both)
    }

    /// Constructor which returns an inverted triangle arrow.
    pub fn inv() -> ArrowShape {
        ArrowShape::Inv(Fill::Filled, Side::Both)
    }

    /// Constructor which returns a T shaped arrow.
    pub fn tee() -> ArrowShape {
        ArrowShape::Tee(Side::Both)
    }

    /// Constructor which returns a V shaped arrow.
    pub fn vee() -> ArrowShape {
        ArrowShape::Vee(Side::Both)
    }

    /// Function which renders given ArrowShape into a String for displaying.
    pub fn to_dot_string(&self) -> String {
        let mut res = String::new();
        match *self {
            Box(fill, side) | ICurve(fill, side)| Diamond(fill, side) |
            Inv(fill, side) | Normal(fill, side)=> {
                res.push_str(fill.as_slice());
                match side {
                    Side::Left | Side::Right => res.push_str(side.as_slice()),
                    Side::Both => {},
                };
            },
            Dot(fill)       => res.push_str(fill.as_slice()),
            Crow(side) | Curve(side) | Tee(side)
            | Vee(side) => {
                match side {
                    Side::Left | Side::Right => res.push_str(side.as_slice()),
                    Side::Both => {},
                }
            }
            NoArrow => {},
        };
        match *self {
            NoArrow         => res.push_str("none"),
            Normal(_, _)    => res.push_str("normal"),
            Box(_, _)       => res.push_str("box"),
            Crow(_)         => res.push_str("crow"),
            Curve(_)        => res.push_str("curve"),
            ICurve(_, _)    => res.push_str("icurve"),
            Diamond(_, _)   => res.push_str("diamond"),
            Dot(_)          => res.push_str("dot"),
            Inv(_, _)       => res.push_str("inv"),
            Tee(_)          => res.push_str("tee"),
            Vee(_)          => res.push_str("vee"),
        };
        res
    }
}

pub type Nodes<'a,N> = Cow<'a,[N]>;
pub type Edges<'a,E> = Cow<'a,[E]>;

/// Graph kind determines if `digraph` or `graph` is used as keyword
/// for the graph.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Kind {
    Digraph,
    Graph,
}

impl Kind {
    /// The keyword to use to introduce the graph.
    /// Determines which edge syntax must be used, and default style.
    fn keyword(&self) -> &'static str {
        match *self {
            Kind::Digraph => "digraph",
            Kind::Graph => "graph"
        }
    }

    /// The edgeop syntax to use for this graph kind.
    fn edgeop(&self) -> &'static str {
        match *self {
            Kind::Digraph => "->",
            Kind::Graph => "--",
        }
    }
}

// (The type parameters in GraphWalk should be associated items,
// when/if Rust supports such.)

/// GraphWalk is an abstraction over a graph = (nodes,edges)
/// made up of node handles `N` and edge handles `E`, where each `E`
/// can be mapped to its source and target nodes.
///
/// The lifetime parameter `'a` is exposed in this trait (rather than
/// introduced as a generic parameter on each method declaration) so
/// that a client impl can choose `N` and `E` that have substructure
/// that is bound by the self lifetime `'a`.
///
/// The `nodes` and `edges` method each return instantiations of
/// `Cow<[T]>` to leave implementers the freedom to create
/// entirely new vectors or to pass back slices into internally owned
/// vectors.
pub trait GraphWalk<'a, N: Clone, E: Clone> {
    /// Returns all the nodes in this graph.
    fn nodes(&'a self) -> Nodes<'a, N>;
    /// Returns all of the edges in this graph.
    fn edges(&'a self) -> Edges<'a, E>;
    /// The source node for `edge`.
    fn source(&'a self, edge: &E) -> N;
    /// The target node for `edge`.
    fn target(&'a self, edge: &E) -> N;
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RenderOption {
    NoEdgeLabels,
    NoNodeLabels,
    NoEdgeStyles,
    NoEdgeColors,
    NoNodeStyles,
    NoNodeColors,
    NoArrows,
}

/// Returns vec holding all the default render options.
pub fn default_options() -> Vec<RenderOption> {
    vec![]
}

/// Renders graph `g` into the writer `w` in DOT syntax.
/// (Simple wrapper around `render_opts` that passes a default set of options.)
pub fn render<'a,
              N: Clone + 'a,
              E: Clone + 'a,
              G: Labeller<'a, N, E> + GraphWalk<'a, N, E>,
              W: Write>
    (g: &'a G,
     w: &mut W)
     -> io::Result<()> {
    render_opts(g, w, &[])
}

/// Renders graph `g` into the writer `w` in DOT syntax.
/// (Main entry point for the library.)
pub fn render_opts<'a,
                   N: Clone + 'a,
                   E: Clone + 'a,
                   G: Labeller<'a, N, E> + GraphWalk<'a, N, E>,
                   W: Write>
    (g: &'a G,
     w: &mut W,
     options: &[RenderOption])
     -> io::Result<()> {
    fn writeln<W: Write>(w: &mut W, arg: &[&str]) -> io::Result<()> {
        for &s in arg {
            write!(w, "{}", s)?;
        }
        write!(w, "\n")
    }

    fn indent<W: Write>(w: &mut W) -> io::Result<()> {
        w.write_all(b"    ")
    }

    writeln(w, &[g.kind().keyword(), " ", g.graph_id().as_slice(), " {"])?;
    if g.kind() == Kind::Digraph {
        if let Some(rankdir) = g.rank_dir() {
            indent(w)?;
            writeln(w, &["rankdir=\"", rankdir.as_slice(), "\";"])?;
        }
    }

    for (name, value) in g.graph_attrs().iter() {
        writeln(w, &[name, "=", value])?;
    }
    for n in g.nodes().iter() {
        let colorstring;

        indent(w)?;
        let id = g.node_id(n);

        let escaped = &g.node_label(n).to_dot_string();
        let shape;

        let mut text = vec![id.as_slice()];

        if !options.contains(&RenderOption::NoNodeLabels) {
            text.push("[label=");
            text.push(escaped);
            text.push("]");
        }

        let style = g.node_style(n);
        if !options.contains(&RenderOption::NoNodeStyles) && style != Style::None {
            text.push("[style=\"");
            text.push(style.as_slice());
            text.push("\"]");
        }

        let color = g.node_color(n);
        if !options.contains(&RenderOption::NoNodeColors) {
            if let Some(c) = color {
                colorstring = c.to_dot_string();
                text.push("[color=");
                text.push(&colorstring);
                text.push("]");
            }
        }

        if let Some(s) = g.node_shape(n) {
            shape = s.to_dot_string();
            text.push("[shape=");
            text.push(&shape);
            text.push("]");
        }

        let node_attrs = g.node_attrs(n).iter().map(|(name, value)| format!("[{name}={value}]")).collect::<Vec<String>>();
        text.extend(node_attrs.iter().map(|s| s as &str));

        text.push(";");
        writeln(w, &text)?;
    }

    for e in g.edges().iter() {
        let colorstring;
        let escaped_label = &g.edge_label(e).to_dot_string();
        let start_arrow = g.edge_start_arrow(e);
        let end_arrow = g.edge_end_arrow(e);
        let start_arrow_s = start_arrow.to_dot_string();
        let end_arrow_s = end_arrow.to_dot_string();

        indent(w)?;
        let source = g.source(e);
        let target = g.target(e);
        let source_id = g.node_id(&source);
        let target_id = g.node_id(&target);

        let mut text = vec![source_id.as_slice(), " ",
                            g.kind().edgeop(), " ",
                            target_id.as_slice()];

        if !options.contains(&RenderOption::NoEdgeLabels) {
            text.push("[label=");
            text.push(escaped_label);
            text.push("]");
        }

        let style = g.edge_style(e);
        if !options.contains(&RenderOption::NoEdgeStyles) && style != Style::None {
            text.push("[style=\"");
            text.push(style.as_slice());
            text.push("\"]");
        }

        let color = g.edge_color(e);
        if !options.contains(&RenderOption::NoEdgeColors) {
            if let Some(c) = color {
                colorstring = c.to_dot_string();
                text.push("[color=");
                text.push(&colorstring);
                text.push("]");
            }
        }

        if !options.contains(&RenderOption::NoArrows) &&
            (!start_arrow.is_default() || !end_arrow.is_default()) {
            text.push("[");
            if !end_arrow.is_default() {
                text.push("arrowhead=\"");
                text.push(&end_arrow_s);
                text.push("\"");
            }
            if !start_arrow.is_default() {
                text.push(" dir=\"both\" arrowtail=\"");
                text.push(&start_arrow_s);
                text.push("\"");
            }

            text.push("]");
        }
        let edge_attrs = g.edge_attrs(e).iter().map(|(name, value)| format!("[{name}={value}]")).collect::<Vec<String>>();
        text.extend(edge_attrs.iter().map(|s| s as &str));
        text.push(";");
        writeln(w, &text)?;
    }

    writeln(w, &["}"])
}

#[cfg(test)]
mod tests {
    use self::NodeLabels::*;
    use super::{Id, Labeller, Nodes, Edges, GraphWalk, render, Style, Kind, RankDir};
    use super::LabelText::{self, LabelStr, EscStr, HtmlStr};
    use super::{Arrow, ArrowShape, Side};
    use std::io;
    use std::io::prelude::*;

    /// each node is an index in a vector in the graph.
    type Node = usize;
    struct Edge {
        from: usize,
        to: usize,
        label: &'static str,
        style: Style,
        start_arrow: Arrow,
        end_arrow: Arrow,
        color: Option<&'static str>,
    }

    fn edge(from: usize, to: usize, label: &'static str, style: Style, color: Option<&'static str>) -> Edge {
        Edge {
            from: from,
            to: to,
            label: label,
            style: style,
            start_arrow: Arrow::default(),
            end_arrow: Arrow::default(),
            color: color,

        }
    }

    fn edge_with_arrows(from: usize, to: usize, label: &'static str, style:Style,
        start_arrow: Arrow, end_arrow: Arrow, color: Option<&'static str>) -> Edge {
        Edge {
            from: from,
            to: to,
            label: label,
            style: style,
            start_arrow: start_arrow,
            end_arrow: end_arrow,
            color: color,
        }
    }


    struct LabelledGraph {
        /// The name for this graph. Used for labelling generated `digraph`.
        name: &'static str,

        /// Each node is an index into `node_labels`; these labels are
        /// used as the label text for each node. (The node *names*,
        /// which are unique identifiers, are derived from their index
        /// in this array.)
        ///
        /// If a node maps to None here, then just use its name as its
        /// text.
        node_labels: Vec<Option<&'static str>>,

        node_styles: Vec<Style>,

        /// Each edge relates a from-index to a to-index along with a
        /// label; `edges` collects them.
        edges: Vec<Edge>,
    }

    // A simple wrapper around LabelledGraph that forces the labels to
    // be emitted as EscStr.
    struct LabelledGraphWithEscStrs {
        graph: LabelledGraph,
    }

    enum NodeLabels<L> {
        AllNodesLabelled(Vec<L>),
        UnlabelledNodes(usize),
        SomeNodesLabelled(Vec<Option<L>>),
    }

    type Trivial = NodeLabels<&'static str>;

    impl NodeLabels<&'static str> {
        fn into_opt_strs(self) -> Vec<Option<&'static str>> {
            match self {
                UnlabelledNodes(len) => vec![None; len],
                AllNodesLabelled(lbls) => lbls.into_iter().map(|l| Some(l)).collect(),
                SomeNodesLabelled(lbls) => lbls.into_iter().collect(),
            }
        }

        fn len(&self) -> usize {
            match self {
                &UnlabelledNodes(len) => len,
                &AllNodesLabelled(ref lbls) => lbls.len(),
                &SomeNodesLabelled(ref lbls) => lbls.len(),
            }
        }
    }

    impl LabelledGraph {
        fn new(name: &'static str,
               node_labels: Trivial,
               edges: Vec<Edge>,
               node_styles: Option<Vec<Style>>)
               -> LabelledGraph {
            let count = node_labels.len();
            LabelledGraph {
                name: name,
                node_labels: node_labels.into_opt_strs(),
                edges: edges,
                node_styles: match node_styles {
                    Some(nodes) => nodes,
                    None => vec![Style::None; count],
                },
            }
        }
    }

    impl LabelledGraphWithEscStrs {
        fn new(name: &'static str,
               node_labels: Trivial,
               edges: Vec<Edge>)
               -> LabelledGraphWithEscStrs {
            LabelledGraphWithEscStrs { graph: LabelledGraph::new(name, node_labels, edges, None) }
        }
    }

    fn id_name<'a>(n: &Node) -> Id<'a> {
        Id::new(format!("N{}", *n)).unwrap()
    }

    impl<'a> Labeller<'a, Node, &'a Edge> for LabelledGraph {
        fn graph_id(&'a self) -> Id<'a> {
            Id::new(&self.name[..]).unwrap()
        }
        fn node_id(&'a self, n: &Node) -> Id<'a> {
            id_name(n)
        }
        fn node_label(&'a self, n: &Node) -> LabelText<'a> {
            match self.node_labels[*n] {
                Some(ref l) => LabelStr((*l).into()),
                None => LabelStr(id_name(n).name()),
            }
        }
        fn edge_label(&'a self, e: &&'a Edge) -> LabelText<'a> {
            LabelStr(e.label.into())
        }
        fn node_style(&'a self, n: &Node) -> Style {
            self.node_styles[*n]
        }
        fn edge_style(&'a self, e: &&'a Edge) -> Style {
            e.style
        }
        fn edge_color(&'a self, e: &&'a Edge) -> Option<LabelText<'a>>
        {
            match e.color {
                Some(l) => {
                    Some(LabelStr((*l).into()))
                },
                None => None,
            }
        }
        fn edge_end_arrow(&'a self, e: &&'a Edge) -> Arrow {
            e.end_arrow.clone()
        }

        fn edge_start_arrow(&'a self, e: &&'a Edge) -> Arrow {
            e.start_arrow.clone()
        }
    }

    impl<'a> Labeller<'a, Node, &'a Edge> for LabelledGraphWithEscStrs {
        fn graph_id(&'a self) -> Id<'a> {
            self.graph.graph_id()
        }
        fn node_id(&'a self, n: &Node) -> Id<'a> {
            self.graph.node_id(n)
        }
        fn node_label(&'a self, n: &Node) -> LabelText<'a> {
            match self.graph.node_label(n) {
                LabelStr(s) | EscStr(s) | HtmlStr(s) => EscStr(s),
            }
        }
        fn node_color(&'a self, n: &Node) -> Option<LabelText<'a>> {
            match self.graph.node_color(n) {
                Some(LabelStr(s)) | Some(EscStr(s)) | Some(HtmlStr(s)) => Some(EscStr(s)),
                None => None,
            }
        }
        fn edge_label(&'a self, e: &&'a Edge) -> LabelText<'a> {
            match self.graph.edge_label(e) {
                LabelStr(s) | EscStr(s) | HtmlStr(s) => EscStr(s),
            }
        }
        fn edge_color(&'a self, e: &&'a Edge) -> Option<LabelText<'a>> {
            match self.graph.edge_color(e) {
                Some(LabelStr(s)) | Some(EscStr(s)) | Some(HtmlStr(s)) => Some(EscStr(s)),
                None => None,
            }
        }
    }

    impl<'a> GraphWalk<'a, Node, &'a Edge> for LabelledGraph {
        fn nodes(&'a self) -> Nodes<'a, Node> {
            (0..self.node_labels.len()).collect()
        }
        fn edges(&'a self) -> Edges<'a, &'a Edge> {
            self.edges.iter().collect()
        }
        fn source(&'a self, edge: &&'a Edge) -> Node {
            edge.from
        }
        fn target(&'a self, edge: &&'a Edge) -> Node {
            edge.to
        }
    }

    impl<'a> GraphWalk<'a, Node, &'a Edge> for LabelledGraphWithEscStrs {
        fn nodes(&'a self) -> Nodes<'a, Node> {
            self.graph.nodes()
        }
        fn edges(&'a self) -> Edges<'a, &'a Edge> {
            self.graph.edges()
        }
        fn source(&'a self, edge: &&'a Edge) -> Node {
            edge.from
        }
        fn target(&'a self, edge: &&'a Edge) -> Node {
            edge.to
        }
    }

    fn test_input(g: LabelledGraph) -> io::Result<String> {
        let mut writer = Vec::new();
        render(&g, &mut writer).unwrap();
        let mut s = String::new();
        Read::read_to_string(&mut &*writer, &mut s)?;
        Ok(s)
    }

    // All of the tests use raw-strings as the format for the expected outputs,
    // so that you can cut-and-paste the content into a .dot file yourself to
    // see what the graphviz visualizer would produce.

    #[test]
    fn empty_graph() {
        let labels: Trivial = UnlabelledNodes(0);
        let r = test_input(LabelledGraph::new("empty_graph", labels, vec![], None));
        assert_eq!(r.unwrap(),
r#"digraph empty_graph {
}
"#);
    }

    #[test]
    fn single_node() {
        let labels: Trivial = UnlabelledNodes(1);
        let r = test_input(LabelledGraph::new("single_node", labels, vec![], None));
        assert_eq!(r.unwrap(),
r#"digraph single_node {
    N0[label="N0"];
}
"#);
    }

    #[test]
    fn single_node_with_style() {
        let labels: Trivial = UnlabelledNodes(1);
        let styles = Some(vec![Style::Dashed]);
        let r = test_input(LabelledGraph::new("single_node", labels, vec![], styles));
        assert_eq!(r.unwrap(),
r#"digraph single_node {
    N0[label="N0"][style="dashed"];
}
"#);
    }

    #[test]
    fn single_edge() {
        let labels: Trivial = UnlabelledNodes(2);
        let result = test_input(LabelledGraph::new("single_edge",
                                                   labels,
                                                   vec![edge(0, 1, "E", Style::None, None)],
                                                   None));
        assert_eq!(result.unwrap(),
r#"digraph single_edge {
    N0[label="N0"];
    N1[label="N1"];
    N0 -> N1[label="E"];
}
"#);
    }

    #[test]
    fn single_edge_with_style() {
        let labels: Trivial = UnlabelledNodes(2);
        let result = test_input(LabelledGraph::new("single_edge",
                                                   labels,
                                                   vec![edge(0, 1, "E", Style::Bold, Some("red"))],
                                                   None));
        assert_eq!(result.unwrap(),
r#"digraph single_edge {
    N0[label="N0"];
    N1[label="N1"];
    N0 -> N1[label="E"][style="bold"][color="red"];
}
"#);
    }

    #[test]
    fn test_some_labelled() {
        let labels: Trivial = SomeNodesLabelled(vec![Some("A"), None]);
        let styles = Some(vec![Style::None, Style::Dotted]);
        let result = test_input(LabelledGraph::new("test_some_labelled",
                                                   labels,
                                                   vec![edge(0, 1, "A-1", Style::None, None)],
                                                   styles));
        assert_eq!(result.unwrap(),
r#"digraph test_some_labelled {
    N0[label="A"];
    N1[label="N1"][style="dotted"];
    N0 -> N1[label="A-1"];
}
"#);
    }

    #[test]
    fn single_cyclic_node() {
        let labels: Trivial = UnlabelledNodes(1);
        let r = test_input(LabelledGraph::new("single_cyclic_node",
                                              labels,
                                              vec![edge(0, 0, "E", Style::None, None)],
                                              None));
        assert_eq!(r.unwrap(),
r#"digraph single_cyclic_node {
    N0[label="N0"];
    N0 -> N0[label="E"];
}
"#);
    }

    #[test]
    fn hasse_diagram() {
        let labels = AllNodesLabelled(vec!("{x,y}", "{x}", "{y}", "{}"));
        let r = test_input(LabelledGraph::new("hasse_diagram",
                                              labels,
                                              vec![edge(0, 1, "", Style::None, Some("green")),
                                                   edge(0, 2, "", Style::None, Some("blue")),
                                                   edge(1, 3, "", Style::None, Some("red")),
                                                   edge(2, 3, "", Style::None, Some("black"))],
                                              None));
        assert_eq!(r.unwrap(),
r#"digraph hasse_diagram {
    N0[label="{x,y}"];
    N1[label="{x}"];
    N2[label="{y}"];
    N3[label="{}"];
    N0 -> N1[label=""][color="green"];
    N0 -> N2[label=""][color="blue"];
    N1 -> N3[label=""][color="red"];
    N2 -> N3[label=""][color="black"];
}
"#);
    }

    #[test]
    fn utf8_diagram() {
        let labels = AllNodesLabelled(vec!("Λ", "ι"));
        let r = test_input(LabelledGraph::new("utf8_diagram",
                                              labels,
                                              vec![edge(0, 1, "☕", Style::None, None)],
                                              None));
        assert_eq!(r.unwrap(),
r#"digraph utf8_diagram {
    N0[label="Λ"];
    N1[label="ι"];
    N0 -> N1[label="☕"];
}
"#);
    }

    #[test]
    fn left_aligned_text() {
        let labels = AllNodesLabelled(vec!(
            "if test {\
           \\l    branch1\
           \\l} else {\
           \\l    branch2\
           \\l}\
           \\lafterward\
           \\l",
            "branch1",
            "branch2",
            "afterward"));

        let mut writer = Vec::new();

        let g = LabelledGraphWithEscStrs::new("syntax_tree",
                                              labels,
                                              vec![edge(0, 1, "then", Style::None, None),
                                                   edge(0, 2, "else", Style::None, None),
                                                   edge(1, 3, ";", Style::None, None),
                                                   edge(2, 3, ";", Style::None, None)]);

        render(&g, &mut writer).unwrap();
        let mut r = String::new();
        Read::read_to_string(&mut &*writer, &mut r).unwrap();

        assert_eq!(r,
r#"digraph syntax_tree {
    N0[label="if test {\l    branch1\l} else {\l    branch2\l}\lafterward\l"];
    N1[label="branch1"];
    N2[label="branch2"];
    N3[label="afterward"];
    N0 -> N1[label="then"];
    N0 -> N2[label="else"];
    N1 -> N3[label=";"];
    N2 -> N3[label=";"];
}
"#);
    }

    #[test]
    fn simple_id_construction() {
        let id1 = Id::new("hello");
        match id1 {
            Ok(_) => {}
            Err(..) => panic!("'hello' is not a valid value for id anymore"),
        }
    }

    #[test]
    fn test_some_arrow() {
        let labels: Trivial = SomeNodesLabelled(vec![Some("A"), None]);
        let styles = Some(vec![Style::None, Style::Dotted]);
        let start  = Arrow::default();
        let end    = Arrow::from_arrow(ArrowShape::crow());
        let result = test_input(LabelledGraph::new("test_some_labelled",
                                                   labels,
                                                   vec![edge_with_arrows(0, 1, "A-1", Style::None, start, end, None)],
                                                   styles));
        assert_eq!(result.unwrap(),
r#"digraph test_some_labelled {
    N0[label="A"];
    N1[label="N1"][style="dotted"];
    N0 -> N1[label="A-1"][arrowhead="crow"];
}
"#);
    }

    #[test]
    fn test_some_arrows() {
        let labels: Trivial = SomeNodesLabelled(vec![Some("A"), None]);
        let styles = Some(vec![Style::None, Style::Dotted]);
        let start  = Arrow::from_arrow(ArrowShape::tee());
        let end    = Arrow::from_arrow(ArrowShape::Crow(Side::Left));
        let result = test_input(LabelledGraph::new("test_some_labelled",
                                                   labels,
                                                   vec![edge_with_arrows(0, 1, "A-1", Style::None, start, end, None)],
                                                   styles));
        assert_eq!(result.unwrap(),
r#"digraph test_some_labelled {
    N0[label="A"];
    N1[label="N1"][style="dotted"];
    N0 -> N1[label="A-1"][arrowhead="lcrow" dir="both" arrowtail="tee"];
}
"#);
    }

    #[test]
    fn badly_formatted_id() {
        let id2 = Id::new("Weird { struct : ure } !!!");
        match id2 {
            Ok(_) => panic!("graphviz id suddenly allows spaces, brackets and stuff"),
            Err(..) => {}
        }
    }

    type SimpleEdge = (Node, Node);

    struct DefaultStyleGraph {
        /// The name for this graph. Used for labelling generated graph
        name: &'static str,
        nodes: usize,
        edges: Vec<SimpleEdge>,
        kind: Kind,
        rankdir: Option<RankDir>,
    }

    impl DefaultStyleGraph {
        fn new(name: &'static str,
               nodes: usize,
               edges: Vec<SimpleEdge>,
               kind: Kind)
               -> DefaultStyleGraph {
            assert!(!name.is_empty());
            DefaultStyleGraph {
                name: name,
                nodes: nodes,
                edges: edges,
                kind: kind,
                rankdir: None,
            }
        }

        fn with_rankdir(self, rankdir: Option<RankDir>) -> Self {
            Self {
                rankdir,
                ..self
            }
        }
    }

    impl<'a> Labeller<'a, Node, &'a SimpleEdge> for DefaultStyleGraph {
        fn graph_id(&'a self) -> Id<'a> {
            Id::new(&self.name[..]).unwrap()
        }
        fn node_id(&'a self, n: &Node) -> Id<'a> {
            id_name(n)
        }
        fn kind(&self) -> Kind {
            self.kind
        }
        fn rank_dir(&self) -> Option<RankDir> {
            self.rankdir
        }
    }

    impl<'a> GraphWalk<'a, Node, &'a SimpleEdge> for DefaultStyleGraph {
        fn nodes(&'a self) -> Nodes<'a, Node> {
            (0..self.nodes).collect()
        }
        fn edges(&'a self) -> Edges<'a, &'a SimpleEdge> {
            self.edges.iter().collect()
        }
        fn source(&'a self, edge: &&'a SimpleEdge) -> Node {
            edge.0
        }
        fn target(&'a self, edge: &&'a SimpleEdge) -> Node {
            edge.1
        }
    }

    fn test_input_default(g: DefaultStyleGraph) -> io::Result<String> {
        let mut writer = Vec::new();
        render(&g, &mut writer).unwrap();
        let mut s = String::new();
        Read::read_to_string(&mut &*writer, &mut s)?;
        Ok(s)
    }

    #[test]
    fn default_style_graph() {
        let r = test_input_default(
            DefaultStyleGraph::new("g", 4,
                                   vec![(0, 1), (0, 2), (1, 3), (2, 3)],
                                   Kind::Graph));
        assert_eq!(r.unwrap(),
r#"graph g {
    N0[label="N0"];
    N1[label="N1"];
    N2[label="N2"];
    N3[label="N3"];
    N0 -- N1[label=""];
    N0 -- N2[label=""];
    N1 -- N3[label=""];
    N2 -- N3[label=""];
}
"#);
    }

    #[test]
    fn default_style_digraph() {
        let r = test_input_default(
            DefaultStyleGraph::new("di", 4,
                                   vec![(0, 1), (0, 2), (1, 3), (2, 3)],
                                   Kind::Digraph));
        assert_eq!(r.unwrap(),
r#"digraph di {
    N0[label="N0"];
    N1[label="N1"];
    N2[label="N2"];
    N3[label="N3"];
    N0 -> N1[label=""];
    N0 -> N2[label=""];
    N1 -> N3[label=""];
    N2 -> N3[label=""];
}
"#);
    }

    #[test]
    fn digraph_with_rankdir() {
        let r = test_input_default(
            DefaultStyleGraph::new("di", 4, vec![(0, 1), (0, 2)],
                                   Kind::Digraph)
                .with_rankdir(Some(RankDir::LeftRight)));
        assert_eq!(
            r.unwrap(),
            r#"digraph di {
    rankdir="LR";
    N0[label="N0"];
    N1[label="N1"];
    N2[label="N2"];
    N3[label="N3"];
    N0 -> N1[label=""];
    N0 -> N2[label=""];
}
"#
        );
    }
}
