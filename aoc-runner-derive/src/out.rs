use crate::map::InnerMap;
use crate::utils::{to_camelcase, to_input, to_snakecase};
use crate::AOC_RUNNER;
use aoc_runner_internal::{DayParts, DayPartsBuilder};
use proc_macro as pm;
use proc_macro2 as pm2;
use quote::quote;
use std::error;
use syn::parse::{Error as ParseError, Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{LitInt, Token};

mod kw {
    use syn::custom_keyword;
    custom_keyword!(lib);
    custom_keyword!(year);
}

enum LibMacroArg {
    Year {
        type_tok: kw::year,
        _eq_tok: Token![=],
        value: LitInt,
    },
    LibRef {
        type_tok: kw::lib,
        _eq_tok: Token![=],
        value: pm2::Ident,
    },
}

impl LibMacroArg {
    fn get_from_stream(tokens: &pm::TokenStream) -> Result<impl Iterator<Item = Self>, ParseError> {
        let parser = Punctuated::<LibMacroArg, Token![,]>::parse_terminated;
        Ok(parser.parse(tokens.clone())?.into_iter())
    }
}

impl Parse for LibMacroArg {
    fn parse(input: ParseStream) -> Result<Self, ParseError> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::year) {
            Ok(LibMacroArg::Year {
                type_tok: input.parse::<kw::year>()?,
                _eq_tok: input.parse()?,
                value: input.parse()?,
            })
        } else if lookahead.peek(kw::lib) {
            Ok(LibMacroArg::LibRef {
                type_tok: input.parse::<kw::lib>()?,
                _eq_tok: input.parse()?,
                value: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug)]
struct LibInfos {
    year: u32,
}

#[derive(Debug)]
enum MainInfos {
    Ref {
        lib: pm2::Ident,
        #[allow(dead_code)]
        year: Option<u32>,
    },
    Standalone {
        year: u32,
    },
}

pub fn lib_impl(input: pm::TokenStream) -> pm::TokenStream {
    let infos = parse_lib_infos(input);
    if let Err(e) = infos {
        return e.to_compile_error().into();
    }
    let infos = infos.unwrap();

    AOC_RUNNER.with(|map| {
        let map = map.consume().expect("failed to consume map from lib");

        let year = infos.year;

        write_infos(&map, year).expect("failed to write infos from lib");

        pm::TokenStream::from(headers(&map, year))
    })
}

pub fn main_impl(input: pm::TokenStream) -> pm::TokenStream {
    let infos = parse_main_infos(input);
    // TODO: put this in upper function and properly wrap below?
    if let Err(e) = infos {
        return e.to_compile_error().into();
    }
    let infos = infos.unwrap();

    AOC_RUNNER.with(|map| {
        let map = map.consume().expect("failed to consume map from main");

        let expanded = match infos {
            MainInfos::Ref { lib, .. } => {
                let infos =
                    read_infos(lib.to_string()).expect("failed to read infos from ref main");
                body(&infos, Some(lib))
            }
            MainInfos::Standalone { year } => {
                let infos =
                    write_infos(&map, year).expect("failed to write infos from standalone main");
                let headers = headers(&map, year);
                let body = body(&infos, None);

                quote! {
                    #headers

                    #body
                }
            }
        };

        pm::TokenStream::from(expanded)
    })
}

fn headers(map: &InnerMap, year: u32) -> pm2::TokenStream {
    let traits_impl: pm2::TokenStream = map
        .keys()
        .map(|dp| {
            let snake = to_snakecase(dp);
            let camel = to_camelcase(dp);

            quote! {
                #[doc(hidden)]
                pub trait #camel {
                    fn #snake(input: ArcStr) -> Result<Box<dyn Runner>, Box<dyn Error>>;
                }
            }
        })
        .collect();

    quote! {
        pub use self::aoc_factory::*;

        #[allow(unused)]
        mod aoc_factory {
            use aoc_runner::{Runner, ArcStr};
            use std::error::Error;

            #[doc(hidden)]
            pub static YEAR : u32 = #year;

            #[doc(hidden)]
            pub struct Factory();

            #traits_impl
        }
    }
}

fn body(infos: &DayParts, lib: Option<pm2::Ident>) -> pm2::TokenStream {
    let mut days: Vec<_> = infos.iter().map(|dp| dp.day).collect();
    days.sort();
    days.dedup();

    let inputs: pm2::TokenStream = days
        .into_iter()
        .map(|d| {
            let name = to_input(d);
            let input = format!("../input/{}/day{}.txt", infos.year, d.0);

            quote! { let #name = ArcStr::from(include_str!(#input)); }
        })
        .collect();

    let body : pm2::TokenStream = infos.iter().map(|dp| {
        let identifier = to_snakecase(dp);
        let (pattern, err) = if let Some(n) = &dp.name {
            (
                format!(
                    "Day {} - Part {} - {}: {{}}\n\tgenerator: {{:?}},\n\trunner: {{:?}}\n",
                    dp.day.0, dp.part.0, n
                ),
                format! (
                    "Day {} - Part {} - {}: FAILED while {{}}:\n{{:#?}}\n",
                    dp.day.0, dp.part.0, n
                )
            )
        } else {
            (
                format!(
                    "Day {} - Part {}: {{}}\n\tgenerator: {{:?}},\n\trunner: {{:?}}\n",
                    dp.day.0, dp.part.0
                ),
                format! (
                    "Day {} - Part {}: FAILED while {{}}:\n{{:#?}}\n",
                    dp.day.0, dp.part.0
                )
            )
        };

        let input = to_input(dp.day);

        quote! {
            {
                let start_time = Instant::now();

                match Factory::#identifier(#input.clone()) {
                    Ok(runner) => {
                        let inter_time = Instant::now();

                        match runner.try_run() {
                            Ok(result) => {
                                let final_time = Instant::now();
                                println!(#pattern, result, (inter_time - start_time), (final_time - inter_time));
                            },
                            Err(e) => eprintln!(#err, "running", e)
                        }
                    },
                    Err(e) => eprintln!(#err, "generating", e)
                }
            }
        }
    }).collect();

    if let Some(lib) = lib {
        quote! {
            use #lib::*;

            fn main() {
                use aoc_runner::ArcStr;
                use std::time::{Duration, Instant};

                #inputs

                println!("Advent of code {}", YEAR);

                #body
            }
        }
    } else {
        quote! {
            fn main() {
                use aoc_runner::ArcStr;
                use std::time::{Duration, Instant};


                #inputs

                println!("Advent of code {}", YEAR);

                #body
            }
        }
    }
}

fn write_infos(map: &InnerMap, year: u32) -> Result<DayParts, Box<dyn error::Error>> {
    let mut day_parts = map
        .iter()
        .filter_map(|(dp, runner)| {
            if runner.solver.is_some() {
                Some(dp.clone())
            } else {
                None
            }
        })
        .collect::<DayPartsBuilder>()
        .with_year(year);

    day_parts.sort();

    day_parts.save()?;

    Ok(day_parts)
}

fn read_infos(crate_name: String) -> Result<DayParts, Box<dyn error::Error>> {
    DayParts::load(crate_name, None)
}

fn parse_lib_infos(infos: pm::TokenStream) -> Result<LibInfos, ParseError> {
    let args: Vec<LibMacroArg> = LibMacroArg::get_from_stream(&infos)?.collect();

    let mut year = None;
    for arg in args {
        match arg {
            LibMacroArg::Year {
                type_tok, value, ..
            } => {
                if year.is_some() {
                    return Err(ParseError::new(
                        type_tok.span,
                        "Year cannot be given multiple times!",
                    ));
                } else {
                    year = Some(value.base10_parse()?);
                }
            }
            LibMacroArg::LibRef { type_tok, .. } => {
                return Err(ParseError::new(
                    type_tok.span,
                    "'lib' is only allowed in `aoc_main`!",
                ));
            }
        }
    }
    if year.is_none() {
        let pm2_full: pm2::TokenStream = infos.into();
        return Err(ParseError::new(pm2_full.span(), "Need an argument 'year'!"));
    }
    Ok(LibInfos {
        year: year.unwrap(),
    })
}

fn parse_main_infos(infos: pm::TokenStream) -> Result<MainInfos, ParseError> {
    let args: Vec<LibMacroArg> = LibMacroArg::get_from_stream(&infos)?.collect();

    let mut year = None;
    let mut lib_ref = None;
    for arg in args {
        match arg {
            LibMacroArg::Year {
                type_tok, value, ..
            } => {
                if year.is_some() {
                    return Err(ParseError::new(
                        type_tok.span,
                        "Year cannot be given multiple times!",
                    ));
                } else {
                    year = Some(value.base10_parse()?);
                }
            }
            LibMacroArg::LibRef {
                type_tok, value, ..
            } => {
                if lib_ref.is_some() {
                    return Err(ParseError::new(
                        type_tok.span,
                        "Lib cannot be given multiple times!",
                    ));
                } else {
                    lib_ref = Some(value);
                }
            }
        }
    }

    if year.is_none() && lib_ref.is_none() {
        let pm2_full: pm2::TokenStream = infos.into();
        return Err(ParseError::new(
            pm2_full.span(),
            "Need an argument 'year' or 'lib'!",
        ));
    }

    match lib_ref {
        Some(lib_ref) => Ok(MainInfos::Ref {
            lib: lib_ref,
            year,
        }),
        None => Ok(MainInfos::Standalone {
            year: year.unwrap(),
        }),
    }
}
