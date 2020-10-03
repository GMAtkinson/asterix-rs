use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Generate syntax for updating the fspec from the deku style syntax that would be found
/// in a ASTERIX category
///
/// This is mostly a `hack` in the true sense of the word. Although it works pretty well for
/// the well defined deku derives.
///
/// Input:
/// ```rust, ignore
/// use asterix::data_item::*;
/// #[deku(skip, cond = "is_fspec(DataSourceIdentifier::FRN_48, fspec, 0)")]
/// pub data_source_identifier: Option<DataSourceIdentifier>,
/// ```
///
/// General Output:
/// ```rust, ignore
/// use asterix::data_item::*;
/// if self.data_source_identifier.is_some() {
///     fspec[0] |= DataSourceIdentifier::FRN_48;
/// }
///
/// ```
///
/// There are a few parts that are pre-pended and appended to the end of the above statements, with
/// generation for the vec and cleaning up the fspec.
#[proc_macro_derive(UpdateFspec)]
#[doc(hidden)]
pub fn derive_answer_fn(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    // (self.name, fspec_num, FRN)
    let mut data_items: Vec<(String, String, String)> = vec![];

    if let Data::Struct(s) = input.data {
        if let Fields::Named(f) = s.fields {
            for field in f.named.iter() {
                // check if first fspec field in struct, skip
                let ident = field.ident.as_ref().unwrap();
                if ident == "fspec" {
                    continue;
                }
                // check if doc ident, we don't need that one
                for att in &field.attrs {
                    if att.path.segments[0].ident == "doc" {
                        continue;
                    }
                    // Unholy string parsing instead of AST :(
                    let s: String = format!("{}", att.tokens);
                    let s = &s[7..s.len() - 1];

                    // find FRN
                    let lpos = s.find('(').unwrap() + 1;
                    let comma = s.find(',').unwrap();
                    let fspec_pos = s.rfind(')').unwrap() - 1;

                    let frn = &s[lpos..comma];
                    let fspec = &s[fspec_pos as usize..=fspec_pos as usize];

                    data_items.push((ident.to_string(), fspec.to_string(), frn.to_string()));
                }
            }
        }
    }

    let mut quotes = quote! {};

    for data_item in data_items {
        let ident = syn::Ident::new(&data_item.0.to_string(), syn::export::Span::call_site());
        let fspec_num = data_item.1.parse::<usize>().unwrap();
        let frn = data_item.2;
        let frn = syn::parse_str::<syn::Expr>(&frn).unwrap();
        quotes = quote! {
            #quotes
            if self.#ident.is_some() {
                fspec[#fspec_num] |= #frn;
            }
        }
    }

    let expanded = quote! {
        impl #name {
            pub fn update_fspec(&mut self) {
                let mut fspec = vec![0x00; 10];
                #quotes
                trim_fspec(&mut fspec);
                add_fx(&mut fspec);
                self.fspec = fspec;
            }
        }
    };

    TokenStream::from(expanded)
}
