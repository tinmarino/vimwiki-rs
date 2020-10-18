use crate::tokens::{utils::element_path, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::{LazyRegion, Located, Position, Region};

impl<'a, T: Tokenize> Tokenize for Located<'a, T> {
    fn tokenize(&self, stream: &mut TokenStream) {
        let root = element_path();
        let mut element = TokenStream::new();
        self.as_inner().tokenize(&mut element);

        let lazy_region = do_tokenize!(&self.lazy_region());

        let self_stream = quote! {
            #root::Located::new(
                #element,
                #lazy_region,
            )
        };

        stream.extend(std::iter::once(self_stream))
    }
}

impl_tokenize!(tokenize_lazy_region, LazyRegion<'a>, 'a);
fn tokenize_lazy_region<'a>(lazy_region: &LazyRegion<'a>) -> TokenStream {
    let root = element_path();
    let inner = tokenize_region(&(*lazy_region).into());
    quote! {
        #root::LazyRegion::Owned(#inner)
    }
}

impl_tokenize!(tokenize_region, Region);
fn tokenize_region(region: &Region) -> TokenStream {
    let root = element_path();
    let start = tokenize_position(&region.start);
    let end = tokenize_position(&region.end);
    quote! {
        #root::Region {
            start: #start,
            end: #end,
        }
    }
}

impl_tokenize!(tokenize_position, Position);
fn tokenize_position(position: &Position) -> TokenStream {
    let root = element_path();
    let line = position.line;
    let column = position.column;
    quote! {
        #root::Position {
            line: #line,
            column: #column,
        }
    }
}
