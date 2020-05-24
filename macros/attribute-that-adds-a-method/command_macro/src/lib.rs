extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn;
use std::collections::HashMap;

#[proc_macro_attribute]
pub fn command_func(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn command_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Construct a representation of the impl Rust code we're inspecting as a
    // syntax tree that we can manipulate
    let item_impl = syn::parse_macro_input!(item as syn::ItemImpl);
    // println!("DEBUG: {:?}", item_impl);

    // Get the names of all the methods of the impl we're inspecting whose names
    // start with "command_"
    let mut commands = HashMap::new();
    let command_fn_names: Vec<_> = item_impl.items.iter().filter_map(|subitem| match subitem {
        syn::ImplItem::Method(val) if val.sig.ident.to_string().starts_with("command_") => {
            Some(val.sig.ident.to_string())
        },
        _ => None
    }).collect();
    for command_fn_name in &command_fn_names {
        // Skip command_unknown, that is a special handler
        if command_fn_name != "command_unknown" {
            commands.insert(
                &command_fn_name[8..],
                format_ident!("{}", command_fn_name));
        }
    }

    // Create a new method to be added to the impl
    // For each command method name, see if the given command string matches
    // and if so invoke the method for that command.
    let k = commands.keys();
    let v = commands.values();
    let new_method_pm2ts = quote! {
        fn handle_command(&self, command: &str, command_args: &[String]) {
            match command {
                #( #k => self.#v(command_args) ,)*
                _ => self.command_unknown(command, command_args)
            }
        }
    };

    // Output the original impl with the additional method added to it
    let new_method_pmts = new_method_pm2ts.into();
    let new_method_impl = syn::parse_macro_input!(new_method_pmts as syn::ImplItemMethod);
    let mut new_items = vec![syn::ImplItem::Method(new_method_impl)];
    new_items.extend(item_impl.items);

    let new_item_impl = syn::ItemImpl {
        items: new_items,
        ..item_impl
    };

    let result = quote! {
        #new_item_impl
    };

    // println!("DEBUG: {}", result.to_string());
    result.into()
}