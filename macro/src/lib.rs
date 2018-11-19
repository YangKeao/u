#![feature(extern_crate_item_prelude)]
extern crate proc_macro;
extern crate heck;
use proc_macro::*;
use heck::*;

#[proc_macro]
pub fn match_event(items: TokenStream) -> TokenStream {
    let mut event_match = r#"
        let event = self.connection.wait_for_event();
        match event {
            None => {
                break;
            }
            Some(event) => {
                let r = event.response_type() & !0x80;
                match r {
    "#.to_string();
    for token in items {
        match token {
            TokenTree::Ident(ident) => {
                event_match = format!("{}{}", event_match, format!(r#"
                    xcb::{SHOUTY_SNAKE} => {{
                        self.trigger_event(Event::{CamelCase}({CamelCase} {{}}));
                        trace!("Event {CamelCase} triggered");
                    }}
                "#, SHOUTY_SNAKE=ident.to_string(), CamelCase=ident.to_string().to_camel_case()));
            }
            _ => {
            }
        }
    }
    format!("{}{}", event_match, r#"
                _ => {}
            }
        }
    }
    "#).parse().unwrap()
}
