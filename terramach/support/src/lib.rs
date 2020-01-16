/*
 * Terra Mach
 * Copyright [2020] Volodymyr Lykhonis
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>
 */

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(PartialWidget)]
pub fn derive_partial_widget(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics PartialWidget for #name #ty_generics #where_clause {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn clone_boxed(&self) -> Box<dyn Widget> {
                Box::new(self.clone())
            }

            fn same_content(&self, other: &BoxedWidget) -> bool {
                if let Some(one) = self.as_any().downcast_ref::<#name #ty_generics>() {
                    if let Some(other) = other.as_any().downcast_ref::<#name #ty_generics>() {
                        return one == other;
                    }
                }
                false
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(EventId)]
pub fn derive_event_id(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics Into<u8> for #name #ty_generics #where_clause {
            fn into(self) -> u8 {
                self as u8
            }
        }

        impl #impl_generics From<u8> for #name #ty_generics #where_clause {
            fn from(value: u8) -> Self {
                unsafe { std::mem::transmute(value) }
            }
        }

        impl #impl_generics From<&u8> for #name #ty_generics #where_clause {
            fn from(value: &u8) -> Self {
                Self::from(*value)
            }
        }
    };

    TokenStream::from(expanded)
}
