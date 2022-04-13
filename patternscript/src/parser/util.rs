/*use crate::parser::*;

pub trait ParseTreePrint {
    fn print(&self, ident: usize);
}

impl ParseTreePrint for HeadData {
    fn print(&self, ident: usize) {
        for (k, v) in &self.definitions {
            print!("{}: ", k);
            v.print(ident + 4);
        }
    }
}

impl ParseTreePrint for PatternData {
    fn print(&self, ident: usize) {
        for (k, v) in &self.block {
            print!("{}: ", k);
            v.print(ident + 4);
        }
    }
}

impl ParseTreePrint for Node {
    fn print(&self, ident: usize) {
        match self {
            Node::Head(hd) => {
                print!("{:ident$}Head", "");
                hd.print(ident + 4);
            }
            Node::Pattern(pd) => {
                print!("{:ident$}Pattern", "");
                pd.print(ident + 4);
            }
            Node::Head(hd) => {
                print!("{:ident$}Head", "");
                hd.print(ident + 4);
            }
            Node::Head(hd) => {
                print!("{:ident$}Head", "");
                hd.print(ident + 4);
            }
            Node::Head(hd) => {
                print!("{:ident$}Head", "");
                hd.print(ident + 4);
            }
            Node::Head(hd) => {
                print!("{:ident$}Head", "");
                hd.print(ident + 4);
            }

            _ => {}
        }
    }
}
*/
