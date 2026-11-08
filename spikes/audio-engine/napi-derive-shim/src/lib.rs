//! 摸底实验 T-1：`#[napi]` 的替身。
//!
//! 它不生成任何 JS 绑定，只把被标注的代码原样交回去，
//! 并且递归去掉里面参数、字段、方法上残留的 `#[napi(...)]`（那些位置不允许出现属性宏）。

use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

#[proc_macro_attribute]
pub fn napi(_attr: TokenStream, item: TokenStream) -> TokenStream {
    strip_napi_attrs(item)
}

fn strip_napi_attrs(stream: TokenStream) -> TokenStream {
    let mut out = Vec::new();
    let mut tokens = stream.into_iter().peekable();
    while let Some(token) = tokens.next() {
        if let TokenTree::Punct(p) = &token {
            if p.as_char() == '#' {
                if let Some(TokenTree::Group(g)) = tokens.peek() {
                    if g.delimiter() == Delimiter::Bracket && starts_with_napi(g) {
                        tokens.next();
                        continue;
                    }
                }
            }
        }
        match token {
            TokenTree::Group(g) => {
                let mut stripped = Group::new(g.delimiter(), strip_napi_attrs(g.stream()));
                stripped.set_span(g.span());
                out.push(TokenTree::Group(stripped));
            }
            other => out.push(other),
        }
    }
    out.into_iter().collect()
}

fn starts_with_napi(group: &Group) -> bool {
    matches!(group.stream().into_iter().next(), Some(TokenTree::Ident(i)) if i.to_string() == "napi")
}
