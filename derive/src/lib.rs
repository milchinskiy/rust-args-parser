//! derive macros for rust-args-parser
//! all parsing is delegated to the core crate

use proc_macro::TokenStream;
use proc_macro2::TokenStream as Tokens;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Field, Fields, FieldsNamed, GenericArgument, Ident, LitBool,
    LitChar, LitInt, LitStr, Path, PathArguments, Type,
};

#[proc_macro_derive(Parser, attributes(arg, command))]
pub fn parser(input: TokenStream) -> TokenStream {
    expand(parse_macro_input!(input as DeriveInput), true, false).unwrap_or_else(syn::Error::into_compile_error).into()
}

#[proc_macro_derive(Args, attributes(arg, command))]
pub fn args(input: TokenStream) -> TokenStream {
    expand(parse_macro_input!(input as DeriveInput), false, false).unwrap_or_else(syn::Error::into_compile_error).into()
}

#[proc_macro_derive(Subcommand, attributes(arg, command))]
pub fn subcommand(input: TokenStream) -> TokenStream {
    expand(parse_macro_input!(input as DeriveInput), false, true).unwrap_or_else(syn::Error::into_compile_error).into()
}

#[derive(Default)]
struct Group {
    name: Option<LitStr>,
    mode: Option<LitStr>,
}

#[derive(Default)]
struct CommandMeta {
    name: Option<LitStr>,
    help: Option<LitStr>,
    aliases: Vec<LitStr>,
    groups: Vec<Group>,
    validate: Option<Path>,
    validate_try: Option<Path>,
    handler: Option<Path>,
    handler_try: Option<Path>,
    context: Option<Type>,
}

#[derive(Default)]
struct FieldMeta {
    name: Option<LitStr>,
    positional: bool,
    long: Option<Option<LitStr>>,
    short: Option<Option<LitChar>>,
    help: Option<LitStr>,
    value_name: Option<LitStr>,
    env: Option<LitStr>,
    default_value: Option<LitStr>,
    group: Option<LitStr>,
    required: Option<bool>,
    min: Option<usize>,
    max: Option<usize>,
    action: Option<LitStr>,
    validator: Option<Path>,
    validator_try: Option<Path>,
    subcommand: bool,
    flatten: bool,
    on_flag: Option<Path>,
    on_flag_try: Option<Path>,
    on_value: Option<Path>,
    on_value_try: Option<Path>,
}

fn doc_text(attrs: &[Attribute]) -> Option<LitStr> {
    let lines: Vec<String> = attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .filter_map(|a| match &a.meta {
            syn::Meta::NameValue(nv) => match &nv.value {
                syn::Expr::Lit(e) => match &e.lit {
                    syn::Lit::Str(s) => Some(s.value().trim().to_string()),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        })
        .filter(|line| !line.is_empty())
        .collect();
    if lines.is_empty() {
        None
    } else {
        Some(LitStr::new(&lines.join(" "), proc_macro2::Span::call_site()))
    }
}

fn command_meta(attrs: &[Attribute]) -> syn::Result<CommandMeta> {
    let mut out = CommandMeta::default();
    for attr in attrs.iter().filter(|a| a.path().is_ident("command")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                out.name = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("help") {
                out.help = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("alias") {
                out.aliases.push(meta.value()?.parse()?);
            } else if meta.path.is_ident("validate") {
                out.validate = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("validate_try") {
                out.validate_try = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("handler") {
                out.handler = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("handler_try") {
                out.handler_try = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("context") {
                out.context = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("group") {
                let mut group = Group::default();
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("name") {
                        group.name = Some(nested.value()?.parse()?);
                    } else if nested.path.is_ident("mode") {
                        group.mode = Some(nested.value()?.parse()?);
                    } else {
                        return Err(nested.error("expected group name or mode"));
                    }
                    Ok(())
                })?;
                if group.name.is_none() || group.mode.is_none() {
                    return Err(meta.error("group requires name and mode"));
                }
                out.groups.push(group);
            } else {
                return Err(meta.error("unknown command attribute"));
            }
            Ok(())
        })?;
    }
    if out.validate.is_some() && out.validate_try.is_some() {
        return Err(syn::Error::new(proc_macro2::Span::call_site(), "choose validate or validate_try"));
    }
    if out.handler.is_some() && out.handler_try.is_some() {
        return Err(syn::Error::new(proc_macro2::Span::call_site(), "choose handler or handler_try"));
    }
    Ok(out)
}

fn field_meta(field: &Field) -> syn::Result<FieldMeta> {
    let mut out = FieldMeta::default();
    for attr in field.attrs.iter().filter(|a| a.path().is_ident("arg") || a.path().is_ident("command")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                out.name = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("positional") {
                out.positional = true;
            } else if meta.path.is_ident("short") {
                out.short = Some(if meta.input.peek(syn::Token![=]) { Some(meta.value()?.parse()?) } else { None });
            } else if meta.path.is_ident("long") {
                out.long = Some(if meta.input.peek(syn::Token![=]) { Some(meta.value()?.parse()?) } else { None });
            } else if meta.path.is_ident("help") {
                out.help = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("value_name") {
                out.value_name = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("env") {
                out.env = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("default_value") {
                out.default_value = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("group") {
                out.group = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("required") {
                out.required = Some(if meta.input.peek(syn::Token![=]) {
                    let b: LitBool = meta.value()?.parse()?;
                    b.value
                } else {
                    true
                });
            } else if meta.path.is_ident("min") {
                let n: LitInt = meta.value()?.parse()?;
                out.min = Some(n.base10_parse()?);
            } else if meta.path.is_ident("max") {
                let n: LitInt = meta.value()?.parse()?;
                out.max = Some(n.base10_parse()?);
            } else if meta.path.is_ident("action") {
                out.action = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("value_parser") {
                out.validator = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("value_parser_try") {
                out.validator_try = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("subcommand") {
                out.subcommand = true;
            } else if meta.path.is_ident("flatten") {
                out.flatten = true;
            } else if meta.path.is_ident("on_flag") {
                out.on_flag = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("on_flag_try") {
                out.on_flag_try = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("on_value") {
                out.on_value = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("on_value_try") {
                out.on_value_try = Some(meta.value()?.parse()?);
            } else {
                return Err(meta.error("unknown argument attribute"));
            }
            Ok(())
        })?;
    }
    if out.validator.is_some() && out.validator_try.is_some() {
        return Err(syn::Error::new_spanned(field, "choose value_parser or value_parser_try"));
    }
    if out.on_flag.is_some() && out.on_flag_try.is_some() {
        return Err(syn::Error::new_spanned(field, "choose on_flag or on_flag_try"));
    }
    if out.on_value.is_some() && out.on_value_try.is_some() {
        return Err(syn::Error::new_spanned(field, "choose on_value or on_value_try"));
    }
    Ok(out)
}

fn kebab(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::new();
    for (i, ch) in chars.iter().enumerate() {
        if *ch == '_' {
            out.push('-');
        } else {
            if ch.is_uppercase()
                && i > 0
                && chars[i - 1] != '_'
                && chars[i - 1] != '-'
                && (chars[i - 1].is_lowercase()
                    || chars[i - 1].is_ascii_digit()
                    || (i + 1 < chars.len() && chars[i + 1].is_lowercase()))
            {
                out.push('-');
            }
            out.extend(ch.to_lowercase());
        }
    }
    out
}

fn literal(value: &str) -> LitStr {
    LitStr::new(value, proc_macro2::Span::call_site())
}

fn inner(ty: &Type, container: &str) -> Option<Type> {
    let Type::Path(p) = ty else { return None };
    let seg = p.path.segments.last()?;
    if seg.ident != container {
        return None;
    }
    let PathArguments::AngleBracketed(a) = &seg.arguments else { return None };
    if a.args.len() != 1 {
        return None;
    }
    match a.args.first()? {
        GenericArgument::Type(t) => Some(t.clone()),
        _ => None,
    }
}

fn is_type(ty: &Type, name: &str) -> bool {
    match ty {
        Type::Path(p) => matches!(
            p.path.segments.last(),
            Some(s) if s.ident == name && matches!(&s.arguments, PathArguments::None)
        ),
        _ => false,
    }
}

fn decode_value(ty: &Type, value: Tokens, name: &LitStr) -> Tokens {
    if is_type(ty, "OsString") {
        quote! { ::std::result::Result::<_, ::rust_args_parser::Error>::Ok(::rust_args_parser::derive_api::os_string(#value)) }
    } else if is_type(ty, "PathBuf") {
        quote! { ::std::result::Result::<_, ::rust_args_parser::Error>::Ok(::rust_args_parser::derive_api::path_buf(#value)) }
    } else {
        quote! { ::rust_args_parser::derive_api::parse_value::<#ty>(#value, #name) }
    }
}

struct FieldPlan {
    augment: Vec<Tokens>,
    init: Vec<Tokens>,
    needs_view: bool,
}

fn plan_fields(fields: &FieldsNamed, ctx_ty: &Tokens) -> syn::Result<FieldPlan> {
    let mut plan = FieldPlan { augment: vec![], init: vec![], needs_view: false };
    for field in &fields.named {
        let ident = field.ident.as_ref().ok_or_else(|| syn::Error::new_spanned(field, "expected named field"))?;
        let attr = field_meta(field)?;
        let base = ident.to_string().trim_start_matches("r#").to_string();
        let key = attr.name.clone().unwrap_or_else(|| literal(&base));
        let opt_ty = inner(&field.ty, "Option");
        let vec_ty = inner(&field.ty, "Vec");
        let elem_ty = opt_ty.as_ref().or(vec_ty.as_ref()).unwrap_or(&field.ty);

        if attr.subcommand || attr.flatten {
            if attr.subcommand && attr.flatten {
                return Err(syn::Error::new_spanned(field, "subcommand and flatten are mutually exclusive"));
            }
            if attr.positional
                || attr.short.is_some()
                || attr.long.is_some()
                || attr.default_value.is_some()
                || attr.env.is_some()
                || attr.validator.is_some()
                || attr.validator_try.is_some()
                || attr.name.is_some()
                || attr.value_name.is_some()
                || attr.help.is_some()
                || attr.group.is_some()
                || attr.required.is_some()
                || attr.min.is_some()
                || attr.max.is_some()
                || attr.action.is_some()
                || attr.on_flag.is_some()
                || attr.on_flag_try.is_some()
                || attr.on_value.is_some()
                || attr.on_value_try.is_some()
            {
                return Err(syn::Error::new_spanned(field, "subcommand/flatten fields cannot have option attributes"));
            }
            if attr.subcommand {
                let sub_ty = opt_ty.as_ref().unwrap_or(&field.ty);
                plan.augment.push(quote! { let cmd = <#sub_ty as ::rust_args_parser::Subcommand<#ctx_ty>>::augment_subcommands(cmd); });
                let extract = quote! { <#sub_ty as ::rust_args_parser::Subcommand<#ctx_ty>>::from_matches(m, path)? };
                let init = if opt_ty.is_some() {
                    extract
                } else {
                    quote! { #extract.ok_or_else(|| ::rust_args_parser::Error::User("missing required subcommand".into()))? }
                };
                plan.init.push(quote! { #ident: #init });
            } else {
                let ty = &field.ty;
                plan.augment.push(quote! { let cmd = <#ty as ::rust_args_parser::Args<#ctx_ty>>::augment(cmd); });
                plan.init.push(quote! { #ident: <#ty as ::rust_args_parser::Args<#ctx_ty>>::from_matches(m, path)? });
            }
            continue;
        }
        plan.needs_view = true;

        let is_count = attr.action.as_ref().map(|s| s.value() == "count").unwrap_or(false);
        if attr.action.is_some() && !is_count {
            return Err(syn::Error::new_spanned(field, "only action = \"count\" is supported"));
        }
        let is_flag = is_type(&field.ty, "bool") || is_count;
        if is_flag && (attr.on_value.is_some() || attr.on_value_try.is_some()) {
            return Err(syn::Error::new_spanned(field, "flags use on_flag/on_flag_try"));
        }
        if !is_flag && (attr.on_flag.is_some() || attr.on_flag_try.is_some()) {
            return Err(syn::Error::new_spanned(field, "values and positionals use on_value/on_value_try"));
        }
        if is_count && (opt_ty.is_some() || vec_ty.is_some() || is_type(&field.ty, "bool")) {
            return Err(syn::Error::new_spanned(field, "count requires a scalar integer field"));
        }
        if attr.positional && is_flag {
            return Err(syn::Error::new_spanned(field, "flags cannot be positional"));
        }
        if !attr.positional && (attr.min.is_some() || attr.max.is_some()) {
            return Err(syn::Error::new_spanned(field, "min and max apply only to positional Vec fields"));
        }
        if attr.positional && vec_ty.is_none() && (attr.min.is_some() || attr.max.is_some()) {
            return Err(syn::Error::new_spanned(field, "positional bounds require Vec<T>"));
        }
        if attr.positional
            && (attr.short.is_some()
                || attr.long.is_some()
                || attr.env.is_some()
                || attr.default_value.is_some()
                || attr.group.is_some())
        {
            return Err(syn::Error::new_spanned(field, "positional arguments cannot have option attributes"));
        }
        if is_flag && attr.default_value.is_some() {
            return Err(syn::Error::new_spanned(
                field,
                "flag/count defaults are not supported; use env or a value option",
            ));
        }
        if is_flag && attr.value_name.is_some() {
            return Err(syn::Error::new_spanned(field, "flags/counts do not take values or value_name"));
        }
        if is_flag && (attr.validator.is_some() || attr.validator_try.is_some()) {
            return Err(syn::Error::new_spanned(field, "value validators cannot be attached to flags"));
        }
        if attr.min.is_some() != attr.max.is_some() {
            return Err(syn::Error::new_spanned(field, "specify both min and max"));
        }
        if let (Some(min), Some(max)) = (attr.min, attr.max) {
            if max == 0 || min > max {
                return Err(syn::Error::new_spanned(field, "positional bounds require 0 <= min <= max and max > 0"));
            }
        }

        let pos_name = attr
            .value_name
            .clone()
            .or_else(|| attr.name.clone())
            .unwrap_or_else(|| literal(&base.to_ascii_uppercase()));
        let missing_label = if attr.positional {
            pos_name.value()
        } else if let Some(Some(value)) = &attr.long {
            format!("--{}", value.value())
        } else if let Some(Some(value)) = &attr.short {
            format!("-{}", value.value())
        } else if attr.short.is_some() && attr.long.is_none() {
            format!("-{}", base.chars().next().unwrap_or('?'))
        } else {
            format!("--{}", kebab(&base))
        };
        let missing = literal(&format!(
            "missing required {} '{missing_label}'",
            if attr.positional { "positional" } else { "option" }
        ));
        let required = attr.required.unwrap_or(opt_ty.is_none() && vec_ty.is_none() && !is_flag);
        let doc = attr.help.clone().or_else(|| doc_text(&field.attrs));
        let mut builder: Tokens;
        if attr.positional {
            builder = if let Some(on) = &attr.on_value_try {
                quote! { ::rust_args_parser::PosSpec::new_try(#pos_name, #on) }
            } else if let Some(on) = &attr.on_value {
                quote! { ::rust_args_parser::PosSpec::new(#pos_name, #on) }
            } else {
                quote! { ::rust_args_parser::PosSpec::new(#pos_name, |_, _: &mut _| {}) }
            };
            if let (Some(min), Some(max)) = (attr.min, attr.max) {
                builder = quote! { #builder.range(#min, #max) };
            } else if vec_ty.is_some() {
                builder = quote! { #builder.many() };
            } else if required {
                builder = quote! { #builder.required() };
            }
        } else {
            builder = if is_flag {
                if let Some(on) = &attr.on_flag_try {
                    quote! { ::rust_args_parser::OptSpec::flag_try(#key, #on) }
                } else if let Some(on) = &attr.on_flag {
                    quote! { ::rust_args_parser::OptSpec::flag(#key, #on) }
                } else {
                    quote! { ::rust_args_parser::OptSpec::flag(#key, |_: &mut _| {}) }
                }
            } else if let Some(on) = &attr.on_value_try {
                quote! { ::rust_args_parser::OptSpec::value_try(#key, #on) }
            } else if let Some(on) = &attr.on_value {
                quote! { ::rust_args_parser::OptSpec::value(#key, #on) }
            } else {
                quote! { ::rust_args_parser::OptSpec::value(#key, |_, _: &mut _| {}) }
            };
            let long = match &attr.long {
                Some(Some(value)) => Some(value.clone()),
                Some(None) => Some(literal(&kebab(&base))),
                None if attr.short.is_none() => Some(literal(&kebab(&base))),
                None => None,
            };
            if let Some(long) = long {
                builder = quote! { #builder.long(#long) };
            }
            if let Some(short) = &attr.short {
                let c = match short {
                    Some(value) => value.clone(),
                    None => {
                        let first =
                            base.chars().next().ok_or_else(|| syn::Error::new_spanned(field, "empty field name"))?;
                        LitChar::new(first, proc_macro2::Span::call_site())
                    }
                };
                builder = quote! { #builder.short(#c) };
            }
            if let Some(mv) = &attr.value_name {
                builder = quote! { #builder.metavar(#mv) };
            }
            if let Some(env) = &attr.env {
                builder = quote! { #builder.env(#env) };
            }
            if let Some(default) = &attr.default_value {
                builder = quote! { #builder.default(#default) };
            }
            if let Some(group) = &attr.group {
                builder = quote! { #builder.group(#group) };
            }
            if vec_ty.is_some() || is_count {
                builder = quote! { #builder.repeatable() };
            }
        }
        if let Some(help) = &doc {
            builder = quote! { #builder.help(#help) };
        }
        if let Some(validator) = &attr.validator {
            builder = quote! { #builder.validator(#validator) };
        }
        if let Some(validator) = &attr.validator_try {
            builder = quote! { #builder.validator_try(#validator) };
        }
        plan.augment.push(if attr.positional {
            quote! { let cmd = cmd.pos(#builder); }
        } else {
            quote! { let cmd = cmd.opt(#builder); }
        });

        let init = if is_count {
            let ty = &field.ty;
            quote! {
                {
                    let count = view.flag_count(#key);
                    if #required && count == 0 { return Err(::rust_args_parser::Error::User(#missing.into())); }
                    <#ty as ::std::convert::TryFrom<usize>>::try_from(count)
                        .map_err(|_| ::rust_args_parser::Error::User(format!("flag count overflow for '{}'", #key)))?
                }
            }
        } else if is_flag {
            quote! {
                {
                    let present = view.is_set(#key);
                    if #required && !present { return Err(::rust_args_parser::Error::User(#missing.into())); }
                    present
                }
            }
        } else {
            let value = if attr.positional {
                quote! { view.pos_one(#pos_name) }
            } else {
                quote! { view.value(#key) }
            };
            let values = if attr.positional {
                quote! { view.pos_many(#pos_name) }
            } else {
                quote! { view.values(#key) }
            };
            let parse = decode_value(elem_ty, quote!(value), if attr.positional { &pos_name } else { &key });
            if let Some(t) = vec_ty.as_ref() {
                let parse = decode_value(t, quote!(value.as_os_str()), if attr.positional { &pos_name } else { &key });
                quote! {
                    {
                        let values = #values;
                        if #required && values.map_or(true, |xs| xs.is_empty()) {
                            return Err(::rust_args_parser::Error::User(#missing.into()));
                        }
                        values.unwrap_or(&[]).iter()
                            .map(|value| #parse)
                            .collect::<::rust_args_parser::Result<::std::vec::Vec<#t>>>()?
                    }
                }
            } else if opt_ty.is_some() {
                quote! {
                    {
                        let parsed = #value.map(|value| #parse).transpose()?;
                        if #required && parsed.is_none() {
                            return Err(::rust_args_parser::Error::User(#missing.into()));
                        }
                        parsed
                    }
                }
            } else if required {
                quote! {
                    match #value {
                        Some(value) => #parse?,
                        None => return Err(::rust_args_parser::Error::User(#missing.into())),
                    }
                }
            } else {
                quote! {
                    match #value {
                        Some(value) => #parse?,
                        None => ::std::default::Default::default(),
                    }
                }
            }
        };
        plan.init.push(quote! { #ident: #init });
    }
    Ok(plan)
}

fn commands(cmd: &CommandMeta, attrs: &[Attribute], plan: &FieldPlan) -> syn::Result<Tokens> {
    // Outer command metadata wins over help text from flattened Args.
    let mut steps = plan.augment.clone();
    if let Some(help) = cmd.help.clone().or_else(|| doc_text(attrs)) {
        steps.push(quote! { let cmd = cmd.help(#help); });
    }
    for group in &cmd.groups {
        let name = group.name.as_ref().expect("checked group name");
        let mode = match group.mode.as_ref().expect("checked group mode").value().as_str() {
            "xor" => quote! { ::rust_args_parser::GroupMode::Xor },
            "req_one" => quote! { ::rust_args_parser::GroupMode::ReqOne },
            _ => {
                return Err(syn::Error::new_spanned(group.mode.as_ref().unwrap(), "group mode must be xor or req_one"))
            }
        };
        steps.push(quote! { let cmd = cmd.group(#name, #mode); });
    }
    if let Some(path) = &cmd.validate {
        steps.push(quote! { let cmd = cmd.validator(#path); });
    }
    if let Some(path) = &cmd.validate_try {
        steps.push(quote! { let cmd = cmd.validator_try(#path); });
    }
    if let Some(path) = &cmd.handler {
        steps.push(quote! { let cmd = cmd.handler(#path); });
    }
    if let Some(path) = &cmd.handler_try {
        steps.push(quote! { let cmd = cmd.handler_try(#path); });
    }
    Ok(quote! { #(#steps)* cmd })
}

fn named(input: &DeriveInput) -> syn::Result<&FieldsNamed> {
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => Ok(fields),
            _ => Err(syn::Error::new_spanned(&data.fields, "derive supports only structs with named fields")),
        },
        _ => Err(syn::Error::new_spanned(input, "Parser/Args derive requires a struct")),
    }
}

/// A handler-bearing container is bound to its declared callback context.
/// Containers without handlers remain generic and can be reused in any context.
fn context_impl(declared: Option<&Type>, has_hooks: bool) -> (Tokens, Tokens) {
    if let Some(ty) = declared {
        (quote! {}, quote! { #ty })
    } else if has_hooks {
        (quote! {}, quote! { () })
    } else {
        (quote! { <__RappCtx: ?Sized> }, quote! { __RappCtx })
    }
}

fn expand(input: DeriveInput, is_parser: bool, is_sub: bool) -> syn::Result<Tokens> {
    if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
        return Err(syn::Error::new_spanned(&input.generics, "generic derived types are not yet supported"));
    }
    let ident = &input.ident;
    if is_sub {
        return expand_enum(&input, ident);
    }
    let fields = named(&input)?;
    let cmd = command_meta(&input.attrs)?;
    let field_hooks = fields.named.iter().try_fold(false, |found, field| {
        let a = field_meta(field)?;
        Ok::<bool, syn::Error>(
            found || a.on_flag.is_some() || a.on_flag_try.is_some() || a.on_value.is_some() || a.on_value_try.is_some(),
        )
    })?;
    let (impl_generics, ctx_ty) =
        context_impl(cmd.context.as_ref(), field_hooks || cmd.handler.is_some() || cmd.handler_try.is_some());
    let plan = plan_fields(fields, &ctx_ty)?;
    let spec_steps = commands(&cmd, &input.attrs, &plan)?;
    let values = &plan.init;
    let view_decl = if plan.needs_view {
        quote! { let view = m.at(path); }
    } else {
        quote! {}
    };
    let args_impl = quote! {
        impl #impl_generics ::rust_args_parser::Args<#ctx_ty> for #ident {
            fn augment(cmd: ::rust_args_parser::CmdSpec<'static, #ctx_ty>) -> ::rust_args_parser::CmdSpec<'static, #ctx_ty> {
                #spec_steps
            }
            fn from_matches(m: &::rust_args_parser::Matches, path: &[&str]) -> ::rust_args_parser::Result<Self> {
                let _ = (m, path);
                #view_decl
                Ok(Self { #(#values),* })
            }
        }
    };
    if !is_parser {
        return Ok(args_impl);
    }
    let cmd_name = cmd.name.unwrap_or_else(|| literal(&kebab(&ident.to_string())));
    let parser_ctx = cmd.context.map_or_else(|| quote! { () }, |ty| quote! { #ty });
    Ok(quote! {
        #args_impl
        impl ::rust_args_parser::Parser for #ident {
            type Context = #parser_ctx;
            fn command() -> ::rust_args_parser::CmdSpec<'static, Self::Context> {
                <Self as ::rust_args_parser::Args<Self::Context>>::augment(::rust_args_parser::CmdSpec::new(#cmd_name))
            }
            fn from_matches(m: &::rust_args_parser::Matches) -> ::rust_args_parser::Result<Self> {
                <Self as ::rust_args_parser::Args<Self::Context>>::from_matches(m, &[])
            }
        }
    })
}

fn expand_enum(input: &DeriveInput, ident: &Ident) -> syn::Result<Tokens> {
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new_spanned(input, "Subcommand derive requires an enum"));
    };
    let mut make = Vec::new();
    let mut arms = Vec::new();
    let enum_meta = command_meta(&input.attrs)?;
    let mut has_hooks = false;
    for variant in &data.variants {
        let meta = command_meta(&variant.attrs)?;
        has_hooks |= meta.handler.is_some() || meta.handler_try.is_some();
        if let Fields::Named(named) = &variant.fields {
            for field in &named.named {
                let a = field_meta(field)?;
                has_hooks |=
                    a.on_flag.is_some() || a.on_flag_try.is_some() || a.on_value.is_some() || a.on_value_try.is_some();
            }
        }
    }
    let (impl_generics, ctx_ty) = context_impl(enum_meta.context.as_ref(), has_hooks);
    for variant in &data.variants {
        let variant_ident = &variant.ident;
        let meta = command_meta(&variant.attrs)?;
        let cmd_name = meta.name.clone().unwrap_or_else(|| literal(&kebab(&variant_ident.to_string())));
        let aliases = &meta.aliases;
        let (steps, value, needs_view, needs_path) = match &variant.fields {
            Fields::Unit => {
                let plan = FieldPlan { augment: vec![], init: vec![], needs_view: false };
                (commands(&meta, &variant.attrs, &plan)?, quote! { Self::#variant_ident }, false, false)
            }
            Fields::Named(named) => {
                let plan = plan_fields(named, &ctx_ty)?;
                let steps = commands(&meta, &variant.attrs, &plan)?;
                let values = &plan.init;
                (steps, quote! { Self::#variant_ident { #(#values),* } }, plan.needs_view, !named.named.is_empty())
            }
            Fields::Unnamed(tuple) if tuple.unnamed.len() == 1 => {
                let ty = &tuple.unnamed.first().expect("one field").ty;
                let plan = FieldPlan {
                    augment: vec![quote! { let cmd = <#ty as ::rust_args_parser::Args<#ctx_ty>>::augment(cmd); }],
                    init: vec![],
                    needs_view: false,
                };
                (
                    commands(&meta, &variant.attrs, &plan)?,
                    quote! { Self::#variant_ident(<#ty as ::rust_args_parser::Args<#ctx_ty>>::from_matches(m, path)?) },
                    false,
                    true,
                )
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    &variant.fields,
                    "variants must be unit, named, or one-field tuple of Args",
                ))
            }
        };
        make.push(quote! {
            let cmd = cmd.subcmd({
                let cmd = ::rust_args_parser::CmdSpec::new(#cmd_name);
                #(let cmd = cmd.alias(#aliases);)*
                #steps
            });
        });
        let view_decl = if needs_view {
            quote! { let view = m.at(path); }
        } else {
            quote! {}
        };
        let path_decl = if needs_path {
            quote! { let path = &leaf[..path.len() + 1]; }
        } else {
            quote! {}
        };
        arms.push(quote! {
            Some(#cmd_name) => {
                #path_decl
                #view_decl
                Ok(Some(#value))
            },
        });
    }
    Ok(quote! {
        impl #impl_generics ::rust_args_parser::Subcommand<#ctx_ty> for #ident {
            fn augment_subcommands(cmd: ::rust_args_parser::CmdSpec<'static, #ctx_ty>) -> ::rust_args_parser::CmdSpec<'static, #ctx_ty> {
                #(#make)*
                cmd
            }
            fn from_matches(m: &::rust_args_parser::Matches, path: &[&str]) -> ::rust_args_parser::Result<Option<Self>> {
                let leaf = m.leaf_path();
                match leaf.get(path.len()).copied() {
                    #(#arms)*
                    _ => Ok(None),
                }
            }
        }
    })
}
