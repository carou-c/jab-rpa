#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    pub alternatives: Vec<ComplexSelector>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComplexSelector {
    pub leading_combinator: Combinator,
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

#[allow(clippy::enum_variant_names)]
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
    NthChild(i32),
    NthLastChild(i32),
}
