use proc_macro::{TokenStream, TokenTree};
use quote::{quote, format_ident};
use syn::{braced, parenthesized, parse, parse_macro_input, token, Result, Token};
use syn::{Ident, Macro, Lit};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash};
use std::hash::Hasher;

enum EntryField {
    AssertType,
    DisplayType,
    Condition,
    Message,
    Class,
    Function,
    File,
    BeginLine,
    BeginColumn,
    MustHit,
    Id,
}

#[derive(Debug)]
enum LitOrMacro {
    Text(Lit),
    Macro(Macro),
}

impl Parse for LitOrMacro {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek2(Token![!]) {
            let m = input.parse::<Macro>()?;
            Ok(LitOrMacro::Macro(m))
        } else {
            let t = input.parse::<Lit>()?;
            Ok(LitOrMacro::Text(t))
        }
    }
}

impl LitOrMacro {
    fn as_text(&self) -> String {
        match self {
            LitOrMacro::Text(t) => {
                match t {
                    Lit::Str(lit_str) => format!("{}", lit_str.value()),
                    Lit::Bool(lit_bool) => if lit_bool.value() { "true".to_owned() } else { "false".to_owned() },
                    Lit::Int(lit_int) => lit_int.base10_digits().to_owned(),
                    _ => "".to_owned()
                }
            },
            LitOrMacro::Macro(m) => {
                match m {
                    Macro { path, bang_token:_, delimiter:_, tokens:_ } => {
                        let macro_ident = &path.segments[0].ident;
                        format!("{}!()", macro_ident.to_string())
                    }
                }
            }
        }
    }

    fn as_bool(&self) -> bool {
        match self {
            LitOrMacro::Text(t) => {
                match t {
                    Lit::Bool(lit_bool) => lit_bool.value(),
                    _ => false
                }
            },
            LitOrMacro::Macro(_) => {
                false
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct ParsableNamedField {
    pub name: Ident,
    pub colon: Token![:],
    pub value: LitOrMacro,
}

impl Parse for ParsableNamedField {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        Ok(ParsableNamedField{
            name: input.parse()?,
            colon: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl ParsableNamedField {
    #[allow(dead_code)]
    fn str_repr(&self) -> String {
        self.value.as_text()
    }
}

#[derive(Debug)]
struct ItemStruct {
    #[allow(dead_code)]
    brace_token: token::Brace,
    fields: Punctuated<ParsableNamedField, Token![,]>,
}

impl ItemStruct {
    fn str_repr(&self, s: &str) -> String {
        let mut text = "".to_owned();
        for f in &self.fields {
            let fld_name = f.name.to_string();
            if s == fld_name {
                text = f.value.as_text();
                break;
            }
        };
        text
    }
    fn bool_value(&self, s: &str) -> bool {
        let mut b = false;
        for f in &self.fields {
            let fld_name = f.name.to_string();
            if s == fld_name {
                b = f.value.as_bool();
                break;
            }
        };
        b
    }
}

impl Parse for ItemStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(ItemStruct {
            brace_token: braced!(content in input),
            fields: Punctuated::<ParsableNamedField, Token![,]>::parse_terminated(&content)?,
        })
    }
}

// -------------------------------------------------------------------------------- 
// ParsableField
//
// Used for always, sometimes, alwaysOrUnreachable
// Used for reachable, unreachable
// -------------------------------------------------------------------------------- 
#[allow(dead_code)]
#[derive(Debug)]
struct ParsableField {
    // This covers messages (Lit::Str)
    // Also includes some condition (Lit::Bool)
    // TODO: provide coverage for exprs (non-Lit bools)
    // TODO: provide coverage for serde_json:Value
    pub value: Lit,
}

impl Parse for ParsableField {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        Ok(ParsableField{
            value: input.parse()?,
        })
    }
}

impl ParsableField {
    #[allow(dead_code)]
    fn str_repr(&self) -> String {
        match &self.value {
            Lit::Str(lit_str) => format!("{}", lit_str.value()),
            Lit::Bool(lit_bool) => if lit_bool.value() { "true".to_owned() } else { "false".to_owned() },
            Lit::Int(lit_int) => lit_int.base10_digits().to_owned(),
            _ => "".to_owned()
        }
    }
}

// -------------------------------------------------------------------------------- 
// AssertWithConditionArgs
//
// Used for always!, sometimes!, alwaysOrUnreachable!
// -------------------------------------------------------------------------------- 
#[derive(Debug)]
#[allow(dead_code)]
struct AssertWithConditionArgs {
    paren_token: token::Paren,
    fields: Punctuated<ParsableField, Token![,]>,
}

impl Parse for AssertWithConditionArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(AssertWithConditionArgs {
            paren_token: parenthesized!(content in input),
            fields: Punctuated::<ParsableField, Token![,]>::parse_terminated(&content)?,
        })
    }
}

impl AssertWithConditionArgs {
    #[allow(dead_code)]
    fn str_repr(&mut self, idx: usize) -> String {
        let mut text = "".to_owned();
        let it  = &mut self.fields.iter_mut();
        match it.nth(idx) {
            Some(f) => {
                text = match &f.value {
                    Lit::Str(lit_str) => format!("{}", lit_str.value()),
                    Lit::Bool(lit_bool) => if lit_bool.value() { "true".to_owned() } else { "false".to_owned() },
                    Lit::Int(lit_int) => lit_int.base10_digits().to_owned(),
                    _ => "".to_owned()
                };
                ()
            },
            None => ()
        };
        text
    }
}

// -------------------------------------------------------------------------------- 
// ReachabilityArgs
//
// Used for reachable!, unreachable! 
// -------------------------------------------------------------------------------- 
#[derive(Debug)]
#[allow(dead_code)]
struct ReachabilityArgs {
    paren_token: token::Paren,
    fields: Punctuated<ParsableField, Token![,]>,
}

impl Parse for ReachabilityArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(ReachabilityArgs {
            paren_token: parenthesized!(content in input),
            fields: Punctuated::<ParsableField, Token![,]>::parse_terminated(&content)?,
        })
    }
}

impl ReachabilityArgs {
    #[allow(dead_code)]
    fn str_repr(&mut self, idx: usize) -> String {
        let mut text = "".to_owned();
        let it  = &mut self.fields.iter_mut();
        match it.nth(idx) {
            Some(f) => {
                text = match &f.value {
                    Lit::Str(lit_str) => format!("{}", lit_str.value()),
                    Lit::Bool(lit_bool) => if lit_bool.value() { "true".to_owned() } else { "false".to_owned() },
                    Lit::Int(lit_int) => lit_int.base10_digits().to_owned(),
                    _ => "".to_owned()
                };
                ()
            },
            None => ()
        };
        text
    }
}

// -------------------------------------------------------------------------------- 
// Local Helpers
// -------------------------------------------------------------------------------- 
fn to_hash(text: &str) -> String {
    let mut s = DefaultHasher::new();
    text.hash(&mut s);
    let lc_hash = format!("{:x}", s.finish());
    lc_hash.to_ascii_uppercase()
}

#[allow(dead_code)]
fn show_fields(fields: &Punctuated<ParsableNamedField, Token![,]>) {
    for f in fields {
        let name = &f.name;
        let value = &f.value;
        let value_repr = match value {
            LitOrMacro::Text(t) => {
                match t {
                    Lit::Str(lit_str) => format!("\"{}\"", lit_str.value()),
                    Lit::Bool(lit_bool) => if lit_bool.value() { "true".to_owned() } else { "false".to_owned() },
                    Lit::Int(lit_int) => lit_int.base10_digits().to_owned(),
                    _ => "".to_owned()
                }
            },
            LitOrMacro::Macro(m) => {
                match m {
                    Macro { path, bang_token:_, delimiter:_, tokens:_ } => {
                        let macro_ident = &path.segments[0].ident;
                        format!("{}!()", macro_ident.to_string())
                    }
                }
            }
        };
        eprintln!("Arg: {}: {}", name.to_string(), value_repr)
    }
}

#[proc_macro]
pub fn another_entry(tokens: TokenStream) -> TokenStream {

    let input = parse_macro_input!(tokens as ItemStruct);

    // show_fields(&input.fields);
    let assert_type = input.str_repr("assert_type");
    let message = input.str_repr("message");

    let uc_assert_type = assert_type.to_ascii_uppercase();
    let msg_hash = to_hash(&message);
    let entry_name = format_ident!("{}_{}", uc_assert_type, msg_hash);

    let display_type = input.str_repr("display_type");
    let condition = input.bool_value("condition");
    let must_hit = input.bool_value("must_hit");
    let id = input.str_repr("id");

    let q_text = quote!(
        #[distributed_slice(ANTITHESIS_CATALOG)]
        static #entry_name : antithesis_sdk_rust::assert::CatalogInfo = antithesis_sdk_rust::assert::CatalogInfo{
            assert_type: concat!(#assert_type),            
            display_type: concat!(#display_type),            
            condition: #condition,
            message: concat!(#message),
            class: concat!(module_path!()),
            function: concat!("maybe"),
            file: file!(),
            begin_line: line!(),
            begin_column: column!(),
            must_hit: #must_hit,
            id: concat!(#id),
        };
    );

    q_text.into()
}
// always!(true, "like this", details)

// #[proc_macro]
// pub fn always(tokens: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(tokens as AssertWithConditionArgs);
// 
//     // show_fields(&input.fields);
//     let assert_type = input.str_repr("assert_type");
//     let message = input.str_repr("message");
// 
//     let uc_assert_type = assert_type.to_ascii_uppercase();
//     let msg_hash = to_hash(&message);
//     let entry_name = format_ident!("{}_{}", uc_assert_type, msg_hash);
// 
//     let display_type = input.str_repr("display_type");
//     let condition = input.bool_value("condition");
//     let must_hit = input.bool_value("must_hit");
//     let id = input.str_repr("id");
// 
//     let q_text = quote!(
//         #[distributed_slice(ANTITHESIS_CATALOG)]
//         static #entry_name : antithesis_sdk_rust::assert::CatalogInfo = antithesis_sdk_rust::assert::CatalogInfo{
//             assert_type: concat!(#assert_type),            
//             display_type: concat!(#display_type),            
//             condition: #condition,
//             message: concat!(#message),
//             class: concat!(module_path!()),
//             function: concat!("maybe"),
//             file: file!(),
//             begin_line: line!(),
//             begin_column: column!(),
//             must_hit: #must_hit,
//             id: concat!(#id),
//         };
//     );
// 
//     q_text.into()
//     
// }

//----------------------------------------------------------------------
// Parse the TokenStream directly
//----------------------------------------------------------------------
#[proc_macro]
pub fn catalog_entry(body: TokenStream) -> TokenStream {

    let mut current_ident: Option<EntryField> = None;
    let mut assert_type: Option<String> = None;
    let mut display_type: Option<String> = None;
    let mut condition: Option<bool> = None;
    let mut message: Option<String> = None;
    let mut class: Option<String> = None;
    let mut function: Option<String> = None;
    let mut file: Option<String> = None;
    let mut begin_line: Option<String> = None;
    let mut begin_column: Option<String> = None;
    let mut must_hit: Option<bool> = None;
    let mut id: Option<String> = None;

    for token_tree in body.into_iter() {
       match token_tree {
           TokenTree::Ident(ident) => {
                let ident_text = ident.to_string();
                current_ident = match ident_text.as_str() {
                    "assert_type" => Some(EntryField::AssertType),
                    "display_type" => Some(EntryField::DisplayType),
                    "condition" => Some(EntryField::Condition),
                    "message" => Some(EntryField::Message),
                    "class" => Some(EntryField::Class),
                    "function" => Some(EntryField::Function),
                    "file" => Some(EntryField::File),
                    "begin_line" => Some(EntryField::BeginLine),
                    "begin_column" => Some(EntryField::BeginColumn),
                    "must_hit" => Some(EntryField::MustHit),
                    "id" => Some(EntryField::Id),
                    "true" => {
                        match current_ident {
                            Some(EntryField::Condition) => {
                                condition = Some(true);
                            },
                            Some(EntryField::MustHit) => {
                                must_hit = Some(true);
                            },
                            None => {
                                // was expecting a user ident (not true or false)
                                // not something like this:
                                //
                                //   true/false = <value>
                                //
                                // TODO:
                                // indicate a malformed catalog_entry!() body
                            },
                            _ => {
                                // Not expecting a 'true' or 'false' here
                                // We scanned something like this:
                                //
                                // display_type = true
                                //
                                // TODO:
                                // indicate a malformed catalog_entry!() body
                            }
                        };
                        None
                    },
                    "false" => {
                        match current_ident {
                            Some(EntryField::Condition) => {
                                condition = Some(false);
                            },
                            Some(EntryField::MustHit) => {
                                must_hit = Some(false);
                            },
                            None => {
                                // was expecting a user ident (not true or false)
                                // not something like this:
                                //
                                //   true/false = <value>
                                //
                                // TODO:
                                // indicate a malformed catalog_entry!() body
                            },
                            _ => {
                                // Not expecting a 'true' or 'false' here
                                // We scanned something like this:
                                //
                                // display_type = true
                                //
                                // TODO:
                                // indicate a malformed catalog_entry!() body
                            }
                        };
                        None
                    },
                    "line" => {
                        match current_ident {
                            Some(EntryField::BeginLine) => {
                                begin_line = Some("47".to_owned());
                            },
                            _ => ()
                        };
                        None
                    },
                    "column" => {
                        match current_ident {
                            Some(EntryField::BeginColumn) => {
                                begin_column = Some("3".to_owned());
                            },
                            _ => ()
                        };
                        None
                    }
                    _ => None,
                };
                eprintln!("Ident => {}", ident.to_string())
            },
           TokenTree::Punct(punctuation) => eprintln!("Punct => {}", punctuation.to_string()),
           TokenTree::Literal(literal) => {
                let text = literal.to_string();
                match current_ident {
                    Some(EntryField::AssertType) => assert_type = Some(text),
                    Some(EntryField::DisplayType) => display_type = Some(text),
                    Some(EntryField::Condition) => (),
                    Some(EntryField::Message) => message = Some(text),
                    Some(EntryField::Class) => class = Some(text),
                    Some(EntryField::Function) => function = Some(text),
                    Some(EntryField::File) => file = Some(text),
                    Some(EntryField::BeginLine) => begin_line = Some(text),
                    Some(EntryField::BeginColumn) => begin_column = Some(text),
                    Some(EntryField::MustHit) => (),
                    Some(EntryField::Id) => id = Some(text),
                    None => ()
                };
                // eprintln!(">> Literal => {}", literal.to_string())
            }
           _ => {}
       }
    }

    eprintln!("assert_type: {}", assert_type.unwrap_or("<not provided>".to_owned()));
    eprintln!("display_type: {}", display_type.unwrap_or("<not provided>".to_owned()));
    eprintln!("condition: {}", condition.unwrap_or(false));
    eprintln!("message: {}", message.unwrap_or("<not provided>".to_owned()));
    eprintln!("class: {}", class.unwrap_or("<not provided>".to_owned()));
    eprintln!("function: {}", function.unwrap_or("<not provided>".to_owned()));
    eprintln!("file: {}", file.unwrap_or("<not provided>".to_owned()));
    eprintln!("begin_line: {}", begin_line.unwrap_or("<not provided>".to_owned()));
    eprintln!("begin_column: {}", begin_column.unwrap_or("<not provided>".to_owned()));
    eprintln!("must_hit: {}", must_hit.unwrap_or(false));
    eprintln!("id: {}", id.unwrap_or("<not provided>".to_owned()));
    eprintln!("--------------------------------------------------------------------------------");

    // Ignore the entire macro contents
    TokenStream::new()
}


// #[proc_macro]
// pub fn catalog_macro(input: TokenStream) -> TokenStream {
// 
//     let catalog_item: CatalogEntry = parse_macro_input!(input);
//     code_gen(catalog_item)
// 
// }
// 
// fn code_gen(ce: CatalogEntry) -> TokenStream {
//     let dsp_type = ce.display_type.to_owned();
//     let msg = ce.message.to_owned();
//     let static_var_name = format!("{}_{}", dsp_type.to_ascii_uppercase(), msg.to_ascii_uppercase());
//     let mut result = TokenStream::new();
//     let var_def = quote! {
//         STATIC #static_var_name : MiniCat = MiniCat{
//             assert_type: ce.assert_type,
//             display_type: ce.display_type,
//             message: ce.message,
//         }
//     });
//     TokenStream::from(var_def)
// }


#[allow(dead_code)]
struct CatalogEntry {
    assert_type: String,
    display_type: String,
    condition: bool,
    message: String,
    class: String,
    begin_line: u32,
    begin_column: u32,
    must_hit: bool,
}



// impl Parse for CatalogEntry {
//     fn parse(input: ParseStream) -> Result<Self> {
// 
// 
//         let visibility: Visibility = input.parse()?;
//         input.parse::<Token![static]>()?;
//         input.parse::<Token![ref]>()?;
//         let name: Ident = input.parse()?;
//         input.parse::<Token![:]>()?;
//         let ty: Type = input.parse()?;
//         input.parse::<Token![=]>()?;
//         let init: Expr = input.parse()?;
//         input.parse::<Token![;]>()?;
// 
// 
// 
//         let group = syn::Group::parse(input)?;
//         eprintln!("{:?}", group);
//         Ok(CatalogEntry {
//             assert_type: "sometimes",
//             display_type: "AtLeastOnce"
//             condition: false,
//             message: "Listen for incoming client requests",
//             class: module_path!(),
//             begin_line: line!(),
//             begin_column: column!(),
//             must_hit: true,
//         })
//         // let ident = syn::Ident::parse(input)?;
//         // let _in = <Token![in]>::parse(input)?;
//         // let from = syn::LitInt::parse(input)?;
//         // let inclusive = input.peek(Token![..=]);
//         // if inclusive {
//         //     <Token![..=]>::parse(input)?;
//         // } else {
//         //     <Token![..]>::parse(input)?;
//         // }
//         // let to = syn::LitInt::parse(input)?;
//         // let content;
//         // let _braces = syn::braced!(content in input);
//         // let tt = proc_macro2::TokenStream::parse(&content)?;
// 
//         // Ok(SeqMacroInput {
//         //     from,
//         //     to,
//         //     inclusive,
//         //     tt,
//         //     ident,
//         // })
//     }
// }
