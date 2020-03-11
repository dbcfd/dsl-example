mod symbol;

use symbol::*;

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Group, Span};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Attribute, Error, Ident, Lit, LitStr, Result};

#[proc_macro]
pub fn meetup(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let meetup = parse_macro_input!(tokens as Meetup);

    let ts = meetup.to_token_stream();

    eprintln!("{}", ts);

    ts.into()
}

struct Meetup {
    name: Ident,
    location: LitStr,
    meetings: Vec<Meeting>,
}

impl Meetup {
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        let enum_values = self.meetings.iter().map(|m| {
            let name = m.description.clone();
            quote! {
                #name(#name)
            }
        });

        let enum_ident = format_ident!("{}Meetings", self.name);

        let enum_attendees_matches = self.meetings.iter().map(|m| {
            let name = m.description.clone();

            quote! {
                Self::#name(v) => v.attendees()
            }
        });

        let enumerated = quote! {
            pub enum #enum_ident {
                #(#enum_values),*
            }

            impl #enum_ident {
                pub fn attendees(&self) -> &'static [&'static str] {
                    match self {
                        #(#enum_attendees_matches),*
                    }
                }
            }
        };

        let name = self.name.clone();
        let location = self.location.clone();

        let meetings = self.meetings.iter().map(|m| {
            m.to_token_stream()
        });

        let meetings_instances = self.meetings.iter().map(|m| {
            let name = m.description.clone();
            quote! {
                &#enum_ident::#name(#name::default())
            }
        });

        quote! {
            #enumerated

            #(#meetings)*

            pub struct #name {
                location: &'static str,
                meetings: &'static [&'static #enum_ident],
            }

            impl #name {
                pub fn location(&self) -> &'static str {
                    self.location
                }
                pub fn meetings(&self) -> &'static [&'static #enum_ident] {
                    self.meetings
                }
            }

            impl Default for #name {
                fn default() -> Self {
                    Self {
                        location: #location,
                        meetings: &[
                            #(#meetings_instances),*
                        ]
                    }
                }
            }
        }
    }
}

impl Parse for Meetup {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        let group: Group = input.parse()?;
        let ts: TokenStream = group.stream().into();
        let params = syn::parse_macro_input::parse::<MeetupParams>(ts)?;
        Ok(Meetup {
            name: ident,
            location: params.location,
            meetings: params.meetings,
        })
    }
}

struct MeetupParams {
    location: LitStr,
    meetings: Vec<Meeting>,
}

impl Parse for MeetupParams {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut location = None;
        let mut meetings = vec![];
        while !input.is_empty() {
            let current: Ident = input.parse()?;
            let _punc: syn::token::Eq = input.parse()?;
            if current == LOCATION {
                let v: Lit = input.parse()?;
                if let Lit::Str(s) = v {
                    location = Some(s);
                } else {
                    return Err(Error::new(input.span(), "Not a string literal"));
                }
            } else if current == MEETINGS {
                let group: Group = input.parse()?;
                let ts: TokenStream = group.stream().into();
                let m = syn::parse_macro_input::parse::<Meetings>(ts)?;
                meetings.extend(m.inner);
            } else {
                return Err(Error::new(input.span(), "Meeting should be of the form `Name { location = \"\", meetings = [] }` where meetings is not empty."));
            }
            if !input.is_empty() {
                let _punc: syn::token::Comma = input.parse()?;
            }
        }

        let location = location.ok_or(Error::new(input.span(), "Meeting should be of the form `Name { location = \"\", meetings = [] }` where meetings is not empty."))?;
        if meetings.is_empty() {
            return Err(Error::new(input.span(), "Meeting should be of the form `Name { location = \"\", meetings = [] }` where meetings is not empty."));
        }

        Ok(MeetupParams {
            location: location,
            meetings: meetings,
        })
    }
}

struct Meetings {
    inner: Vec<Meeting>,
}

impl Parse for Meetings {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut meetings = vec![];
        while !input.is_empty() {
            let meeting: Meeting = input.parse()?;
            meetings.push(meeting);
            if !input.is_empty() {
                let _punc: syn::token::Comma = input.parse()?;
            }
        }

        Ok(Meetings {
            inner: meetings,
        })
    }
}

struct Meeting {
    description: Ident,
    topics: Vec<LitStr>,
    attendees: Vec<LitStr>,
}

impl Meeting {
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        let meeting_name = self.description.clone();
        let attendees = self.attendees.iter();
        let topics = if self.topics.is_empty() {
            quote! {}
        } else {
            let topics = self.topics.iter();

            quote! {
                pub fn topics(&self) -> &'static [&'static str] {
                    &[#(#topics),*]
                }
            }
        };

        quote! {
            struct #meeting_name {}

            impl Default for #meeting_name {
                fn default() -> Self {
                    Self {}
                }
            }

            impl #meeting_name {
                pub fn attendees(&self) -> &'static [&'static str] {
                    &[#(#attendees),*]
                }

                #topics
            }
        }
    }
}

impl Parse for Meeting {
    fn parse(input: ParseStream) -> Result<Self> {
        let description: Ident = input.parse()?;
        let group: Group = input.parse()?;
        let ts: TokenStream = group.stream().into();
        let params = syn::parse_macro_input::parse::<MeetingParams>(ts)?;
        Ok(Meeting {
            description: description,
            topics: params.topics,
            attendees: params.attendees,
        })
    }
}

struct MeetingParams {
    topics: Vec<LitStr>,
    attendees: Vec<LitStr>,
}

impl Parse for MeetingParams {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut topics = vec![];
        let mut attendees = vec![];
        while !input.is_empty() {
            let current: Ident = input.parse()?;
            let _punc: syn::token::Eq = input.parse()?;
            if current == TOPICS {
                let group: Group = input.parse()?;
                let ts: TokenStream = group.stream().into();
                let params = syn::parse_macro_input::parse::<LitStrGroup>(ts)?;
                topics.extend(params.inner);
            } else if current == ATTENDEES {
                let group: Group = input.parse()?;
                let ts: TokenStream = group.stream().into();
                let params = syn::parse_macro_input::parse::<LitStrGroup>(ts)?;
                attendees.extend(params.inner);
            } else {
                return Err(Error::new(input.span(), "Meeting should be of the form `Type { attendees = [], topics = [] }` where topics is optional."))
            }
            if !input.is_empty() {
                let _punc: syn::token::Comma = input.parse()?;
            }
        }
        if attendees.is_empty() {
            return Err(Error::new(input.span(), "Meeting should be of the form `Type { attendees = [], topics = [] }` where topics is optional."))
        }
        Ok(MeetingParams {
            attendees: attendees,
            topics: topics,
        })
    }
}

struct LitStrGroup {
    inner: Vec<LitStr>,
}

impl Parse for LitStrGroup {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut strs = vec![];
        while !input.is_empty() {
            let v: Lit = input.parse()?;
            if let Lit::Str(s) = v {
                strs.push(s);
            } else {
                return Err(Error::new(input.span(), "Not a string literal"));
            }
            if !input.is_empty() {
                let _punc: syn::token::Comma = input.parse()?;
            }
        }
        Ok(LitStrGroup {
            inner: strs,
        })
    }
}