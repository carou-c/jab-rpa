use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Selector {
    pub alternatives: Vec<ComplexSelector>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
// Right-to-left
pub struct ComplexSelector {
    pub head: Option<Combinator>,
    pub body: Vec<(CompoundSelector, Combinator)>,
    pub last: CompoundSelector,
}

impl ComplexSelector {
    pub fn is_relative(&self) -> bool {
        if self.head.is_some() {
            return true;
        }
        self.body.iter().any(|(compound, _)| {
            compound
                .pseudo_classes
                .contains(&PseudoClassSelector::Scope)
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Combinator {
    Child,
    Descendant,
    NextSibling,
    SubsequentSibling,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompoundSelector {
    pub role: Option<String>,
    pub attrs: Vec<AttrSelector>,
    pub pseudo_classes: Vec<PseudoClassSelector>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttrSelector {
    Str {
        name: StrAttrName,
        op: StringOp,
        value: StrMatcher,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StrAttrName {
    Name,
    Description,
    States,
    StatesEnUs,
    Text,
    Actions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StringOp {
    Eq,
    ContainsWord,
    Starts,
    Ends,
    Contains,
}

#[derive(Debug, Clone)]
pub enum StrMatcher {
    Plain(String),
    Regex(Regex),
}

impl PartialEq for StrMatcher {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Plain(s), Self::Plain(o)) => s == o,
            (Self::Regex(s), Self::Regex(o)) => s.to_string() == o.to_string(),
            _ => false,
        }
    }
}

impl Eq for StrMatcher {}

impl std::hash::Hash for StrMatcher {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        match self {
            Self::Plain(s) => s.hash(state),
            Self::Regex(r) => r.to_string().hash(state),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttrFlags {
    pub case_insensitive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntAttrName {
    X,
    Y,
    Width,
    Height,
    ChildrenCount,
    Depth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntOp {
    Eq,
    Ne,
    Le,
    Ge,
    Lt,
    Gt,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoolAttrName {
    AccessibleAction,
    AccessibleText,
    AccessibleSelection,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PseudoClassSelector {
    Scope,
    Has(Box<Selector>),
    Not(Box<Selector>),
    RequireState(String),
    ExcludeState(String),
    NthChild(i32),
    NthLastChild(i32),
}

#[cfg(test)]
mod tests;
