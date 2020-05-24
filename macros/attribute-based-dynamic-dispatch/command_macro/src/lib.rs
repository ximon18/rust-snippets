extern crate proc_macro;

use proc_macro::{TokenStream};
use quote::{quote, format_ident};
use syn::{self, Attribute, ImplItemMethod, AttrStyle};
use syn::visit::{self, Visit};


struct CommandFuncFinder{
    is_command_func: bool,
    commands: Vec<String>,
    func_names: Vec<String>,
}

impl CommandFuncFinder {
    fn new() -> CommandFuncFinder {
        CommandFuncFinder {
            is_command_func: false,
            commands: Vec::new(),
            func_names: Vec::new(),
        }
    }
}

impl Visit<'_> for CommandFuncFinder {
    fn visit_attribute(&mut self, node: &Attribute) {
        if self.is_command_func == false {
            if node.style == AttrStyle::Outer && node.path.is_ident("command") {
                let meta = node.parse_meta().unwrap();
                if let syn::Meta::List(list) = meta {
                    if let syn::NestedMeta::Lit(lit) = &list.nested[0] {
                        if let syn::Lit::Str(val) = lit {
                            self.commands.push(val.value());
                            self.is_command_func = true;
                        }
                    }
                }
            }
        }
    }

    fn visit_impl_item_method(&mut self, node: &ImplItemMethod) {
        self.is_command_func = false;
        visit::visit_impl_item_method(self, node);
        if self.is_command_func {
            let name = node.sig.ident.to_string();
            self.func_names.push(name);
        }
    }
}

#[proc_macro_attribute]
pub fn command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn command_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Construct a representation of the impl Rust code we're inspecting as a
    // syntax tree that we can manipulate
    let item_impl = syn::parse_macro_input!(item as syn::ItemImpl);

    // We're looking for functions with attributes like this:
    //     #[command("one")]
    //     fn handle_one(&self, ...) {
    //         ...
    //     }
    //
    // We have to check that the attribute identifier is 'command', that it has
    // a single "path" value (e.g. "one") and then capture the method name
    // ("handle_one") in this case,
    //
    // Code such as the example above appear in the syntax tree like this:
    //
    //   Tip: get a dump like this with: println!("DEBUG: {:#?}", item_impl);
    //
    //DEBUG: ItemImpl {
    //    ...
    //    items: [
    //        Method(
    //            ImplItemMethod {
    //                attrs: [
    //                    Attribute {
    //                        ...
    //                        path: Path {
    //                            ...
    //                            segments: [
    //                                PathSegment {
    //                                    ident: Ident {
    //                                        ident: "command",
    //                                        ...
    //                                    },
    //                                    ...
    //                                },
    //                            ],
    //                        },
    //                        tokens: TokenStream [
    //                            Group {
    //                                delimiter: Parenthesis,
    //                                stream: TokenStream [
    //                                    Literal { lit: Lit { kind: Str, symbol: "one", ... }, ... },
    //                                ],
    //                                ...
    //                            },
    //                        ],
    //                    },
    //                ],
    //                ...
    //                sig: Signature {
    //                    ...
    //                    ident: Ident {
    //                        ident: "handle_one",
    //                        ...
    //                    },

    // Use the syn crates visitor pattern to get the name of every command
    // function and the command it implements.
    let mut visitor = CommandFuncFinder::new();
    visitor.visit_item_impl(&item_impl);

    // Create a new method to be added to the impl
    // For each command name, see if the given command string matches and if so
    // invoke the corresponding handler method for the command.
    //
    // 1. Create k and v variables that can be referenced by the #( repetition )
    //    expression that we'll use inside the quote! macro.
    let k = &visitor.commands;
    let mut v = Vec::with_capacity(k.len());

    // 2. Populate v with the idents of the functions we want the generated code
    //    to invoke.
    for fn_name in visitor.func_names {
        v.push(format_ident!("{}", fn_name));
    }

    // 3. Generate the new method as a proc_macro2::TokenStream.
    let new_method_pm2ts = quote! {
        fn handle_command<F>(&self, command: &str, command_args: &[String], f: F) where
            F: FnOnce(&str, &[String]) {
            match command {
                #( #k => self.#v(command_args) ,)*
                _ => f(command, command_args)
            }
        }
    };

    // 4. Convert the proc_macro2::TokenStream to a proc_macro::TokenStream, and
    //    parse it out as a syntax tree.
    let new_method_pmts = new_method_pm2ts.into();
    let new_method_impl = syn::parse_macro_input!(new_method_pmts as syn::ImplItemMethod);

    // 5. Add the new method impl item to the set of items for this struct.
    let mut new_items = vec![syn::ImplItem::Method(new_method_impl)];
    new_items.extend(item_impl.items);

    // 6. Create a new struct impl with the expanded method item set.
    let new_item_impl = syn::ItemImpl {
        items: new_items,
        ..item_impl
    };

    // 7. Convert the syntax tree of the new struct impl to actual Rust code.
    let result = quote! {
        #new_item_impl
    };

    // println!("DEBUG: {}", result.to_string());
    result.into()
}