#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    pub alternatives: Vec<ComplexSelector>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComplexSelector {
    pub leading_combinator: Option<Combinator>,
    pub first: CompoundSelector,
    pub tail: Vec<(Combinator, CompoundSelector)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Combinator {
    Child,
    Descendant,
    NextSibling,
    SubsequentSibling,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompoundSelector {
    pub role: Option<String>,
    pub states: Vec<String>,
    pub attrs: Vec<AttrSelector>,
    pub pseudo_classes: Vec<PseudoClassSelector>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttrSelector {
    Str {
        name: StrAttrName,
        op: StringOp,
        value: String,
        flags: AttrFlags,
    },
    Int {
        name: IntAttrName,
        op: IntOp,
        value: Option<i32>,
    },
    Bool {
        name: BoolAttrName,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrAttrName {
    Name,
    Description,
    States,
    StatesEnUs,
    Text,
    Actions,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StringOp {
    Eq,
    ContainsWord,
    Starts,
    Ends,
    Contains,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttrFlags {
    pub case_insensitive: bool,
    pub regex: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntAttrName {
    X,
    Y,
    Width,
    Height,
    ChildrenCount,
    Depth,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntOp {
    Eq,
    Ne,
    Le,
    Ge,
    Lt,
    Gt,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoolAttrName {
    AccessibleAction,
    AccessibleText,
    AccessibleSelection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PseudoClassSelector {
    Has(Box<Selector>),
    Not(Box<Selector>),
    NthChild(NthFormula),
    NthLastChild(NthFormula),
    NthOfType(NthFormula),
    NthLastOfType(NthFormula),
}

/// This is just 'an + b'
/// e.g. ':nth-child(2*n + 1)' for odd-numbered childs
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NthFormula {
    pub a: i32,
    pub b: i32,
}

impl NthFormula {
    pub fn new(a: i32, b: i32) -> Self {
        Self { a, b }
    }

    pub fn matches(&self, index: usize) -> bool {
        if self.a == 0 {
            return (index as i32) == self.b;
        }
        if self.b == 0 {
            return (index as i32) % self.a == 0;
        }
        let n = ((index as i32) - self.b) / self.a;
        n >= 0 && (index as i32) == self.a * n + self.b
    }

    #[allow(dead_code)]
    pub fn positions(&self, count: usize) -> Vec<usize> {
        (0..=count).filter(|&i| self.matches(i)).collect()
    }
}
