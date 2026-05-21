use std::fmt;

use super::ast::*;

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let alternatives = self
            .alternatives
            .iter()
            .map(|s| format!("{}", s))
            .collect::<Vec<_>>()
            .join(", ");

        write!(f, "{}", alternatives)
    }
}

impl fmt::Display for ComplexSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(head) = &self.head {
            write!(f, "{}", head)?;
        }

        let body = self
            .body
            .iter()
            .rev()
            .map(|(compound, combinator)| format!("{}{}", compound, combinator))
            .collect::<Vec<_>>()
            .join("");

        write!(f, "{}{}", body, self.last)
    }
}

impl fmt::Display for Combinator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Child => write!(f, " > "),
            Self::Descendant => write!(f, " "),
            Self::NextSibling => write!(f, " + "),
            Self::SubsequentSibling => write!(f, " ~ "),
        }
    }
}

impl fmt::Display for CompoundSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let role = match self.role.as_ref() {
            Some(r) => r,
            None => "",
        };

        let states = if !self.states.is_empty() {
            &format!(".{}", self.states.join("."))
        } else {
            ""
        };

        let attrs = self
            .attrs
            .iter()
            .map(|attr| format!("[{}]", attr))
            .collect::<Vec<_>>()
            .join("");

        let pseudo_classes = self
            .pseudo_classes
            .iter()
            .map(|pseudo_class| format!("{}", pseudo_class))
            .collect::<Vec<_>>()
            .join("");

        let display = format!("{}{}{}{}", role, states, attrs, pseudo_classes);
        if display.is_empty() {
            write!(f, "*")
        } else {
            write!(f, "{}", display)
        }
    }
}

impl fmt::Display for AttrSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Str {
                name,
                op,
                value,
                flags,
            } => {
                let flags = if flags.case_insensitive {
                    &format!(" {}", flags)
                } else {
                    ""
                };

                write!(f, "{}{}{}{}", name, op, value, flags)
            }
            Self::Int { name, op, value } => {
                let value = match value {
                    Some(v) => &format!("{}", v),
                    None => "None",
                };
                write!(f, "{}{}{}", name, op, value)
            }
            Self::Bool { name } => write!(f, "{}", name),
        }
    }
}

impl fmt::Display for StrAttrName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name => write!(f, "name"),
            Self::Description => write!(f, "description"),
            Self::States => write!(f, "states"),
            Self::StatesEnUs => write!(f, "states_en_us"),
            Self::Text => write!(f, "text"),
            Self::Actions => write!(f, "actions"),
        }
    }
}

impl fmt::Display for StringOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eq => write!(f, "="),
            Self::ContainsWord => write!(f, "~="),
            Self::Starts => write!(f, "^="),
            Self::Ends => write!(f, "$="),
            Self::Contains => write!(f, "*="),
        }
    }
}

impl fmt::Display for StrMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain(s) => write!(f, "'{}'", s.replace('\'', "\\'")),
            Self::Regex(r) => write!(f, "'{}' r", r.to_string().replace('\'', "\\'")),
        }
    }
}

impl fmt::Display for AttrFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", if self.case_insensitive { "i" } else { "" },)
    }
}

impl fmt::Display for IntAttrName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::X => write!(f, "x"),
            Self::Y => write!(f, "y"),
            Self::Width => write!(f, "width"),
            Self::Height => write!(f, "height"),
            Self::ChildrenCount => write!(f, "children_count"),
            Self::Depth => write!(f, "depth"),
        }
    }
}

impl fmt::Display for IntOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eq => write!(f, "=="),
            Self::Ne => write!(f, "!="),
            Self::Le => write!(f, "<="),
            Self::Ge => write!(f, ">="),
            Self::Lt => write!(f, "<"),
            Self::Gt => write!(f, ">"),
        }
    }
}

impl fmt::Display for BoolAttrName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AccessibleAction => write!(f, "accessible_action"),
            Self::AccessibleText => write!(f, "accessible_text"),
            Self::AccessibleSelection => write!(f, "accessible_selection"),
        }
    }
}

impl fmt::Display for PseudoClassSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scope => write!(f, ":scope"),
            Self::Has(selector) => write!(f, ":has({})", selector),
            Self::Not(selector) => write!(f, ":not({})", selector),
            Self::NthChild(n) => write!(f, ":nth-child({})", n),
            Self::NthLastChild(n) => write!(f, ":nth-last-child({})", n),
        }
    }
}

#[cfg(test)]
mod tests;
