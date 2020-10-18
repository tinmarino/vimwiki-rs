use super::{InlineElement, InlineElementContainer, Located};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Paragraph<'a> {
    pub content: InlineElementContainer<'a>,
}

impl_located_borrowed_owned!(Paragraph);

impl Paragraph<'_> {
    pub fn to_borrowed(&self) -> Paragraph {
        Paragraph {
            content: self.content.to_borrowed(),
        }
    }

    pub fn into_owned(self) -> Paragraph<'static> {
        Paragraph {
            content: self.content.into_owned(),
        }
    }
}

impl<'a> Paragraph<'a> {
    pub fn to_children(&'a self) -> Vec<Located<InlineElement<'a>>> {
        self.content.to_children()
    }
}

impl<'a> From<Vec<Located<'a, InlineElement<'a>>>> for Paragraph<'a> {
    fn from(elements: Vec<Located<'a, InlineElement<'a>>>) -> Self {
        Self::new(elements.into())
    }
}
