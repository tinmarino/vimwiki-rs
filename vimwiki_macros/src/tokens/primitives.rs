use crate::tokens::{utils::vendor_path, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::{borrow::Cow, path::Path};
use vimwiki::vendor::{chrono::NaiveDate, uriparse::URI};

// Implement primitives that already implement ToTokens via quote crate
impl_tokenize!(bool);
impl_tokenize!(i32);
impl_tokenize!(u32);
impl_tokenize!(f32);
impl_tokenize!(f64);

impl_tokenize!(tokenize_str, str);
impl_tokenize!(tokenize_str, String);
pub fn tokenize_str(ctx: &TokenizeContext, s: &str) -> TokenStream {
    ctx.quote_str(s)
}

impl_tokenize!(tokenize_cow_str, Cow<'a, str>, 'a);
pub fn tokenize_cow_str(ctx: &TokenizeContext, inner: &str) -> TokenStream {
    let inner_t = ctx.quote_str(inner);
    if ctx.verbatim {
        quote! { ::std::borrow::Cow::Borrowed(#inner_t) }
    } else {
        quote! { ::std::borrow::Cow::Owned(#inner_t) }
    }
}

impl_tokenize!(tokenize_cow_path, Cow<'a, Path>, 'a);
pub fn tokenize_cow_path(ctx: &TokenizeContext, path: &Path) -> TokenStream {
    let inner = path.to_str().expect("Unable to translate path to str");
    let inner_t = ctx.quote_str(inner);

    if ctx.verbatim {
        quote! { ::std::borrow::Cow::Borrowed(::std::path::Path::new(#inner_t)) }
    } else {
        quote! {
            ::std::borrow::Cow::Owned(
                <
                    ::std::path::PathBuf as
                    ::std::convert::From<::std::string::String>
                >::from(
                    #inner_t
                )
            )
        }
    }
}

impl_tokenize!(tokenize_naive_date, NaiveDate);
fn tokenize_naive_date(
    _ctx: &TokenizeContext,
    naive_date: &NaiveDate,
) -> TokenStream {
    use vimwiki::vendor::chrono::Datelike;
    let root = vendor_path();
    let year = naive_date.year();
    let month = naive_date.month();
    let day = naive_date.day();
    quote! { #root::chrono::NaiveDate::from_ymd(#year, #month, #day) }
}

impl_tokenize!(tokenize_uri, URI<'a>, 'a);
fn tokenize_uri(_ctx: &TokenizeContext, uri: &URI) -> TokenStream {
    let root = vendor_path();
    let uri_string = uri.to_string();
    quote! {
        {
            use ::std::convert::TryFrom;
            #root::uriparse::URI::try_from(#uri_string)
                .expect("Failed to parse URI").into_owned()
        }
    }
}

impl_tokenize!(tokenize_path, Path);
fn tokenize_path(_ctx: &TokenizeContext, path: &Path) -> TokenStream {
    // TODO: Support cases where pathbuf cannot be converted back to Rust str
    let t = path.to_str().expect("Path cannot be converted to &str");
    quote! {
        ::std::path::Path::new(#t)
    }
}
