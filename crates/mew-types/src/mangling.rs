use mew_parse::{
    span::Spanned,
    syntax::{Expression, PathPart},
};

fn mangle_expression(expr: &Expression) -> String {
    let data: String = format!("{expr}").replace(' ', "").replace('\n', "");
    let mut result = String::new();
    let mut was_unicode = false;
    for c in data.chars() {
        if c.is_alphanumeric() {
            if was_unicode {
                result.push('_');
                was_unicode = false;
            }
            result.push(c);
        } else {
            was_unicode = true;
            let mut buf = vec![0; c.len_utf8()];
            let _ = c.encode_utf8(&mut buf);
            result.push('_');
            for item in buf {
                let str = item.to_string();
                result.push_str(&str);
            }
        }
    }
    result
}

pub fn maybe_mangle_template_args_if_needed(path_part: &PathPart) -> String {
    if path_part.template_args.is_none() || path_part.template_args.as_ref().unwrap().is_empty() {
        return path_part.name.value.clone();
    }
    mangle_template_args(path_part)
}

pub fn mangle_template_args(path_part: &PathPart) -> String {
    let name: &String = &path_part.name.replace('_', "__");
    let mut template_args = String::new();
    for template_arg in path_part.template_args.iter().flatten() {
        template_args.push('_');
        template_args.push_str(&mangle_expression(&template_arg.expression).replace('_', "__"));
    }
    format!("{name}{template_args}")
}

pub fn mangle_inline_arg_name(
    enclosing_path: &[PathPart],
    parent_path: &[PathPart],
    template_arg_name: &str,
) -> String {
    let template_arg_name = template_arg_name.replace('_', "__");
    let mut enclosing_path_result = String::new();
    for i in enclosing_path.iter() {
        enclosing_path_result.push_str(&maybe_mangle_template_args_if_needed(i).replace('_', "__"));
        enclosing_path_result.push('_');
    }
    let mut parent_path_result = String::new();
    for i in parent_path.iter() {
        parent_path_result.push_str(&maybe_mangle_template_args_if_needed(i).replace('_', "__"));
        parent_path_result.push('_');
    }

    format!("{enclosing_path_result}{parent_path_result}{template_arg_name}")
}

pub fn mangle_path(path: &mut Vec<PathPart>) {
    let mut result = Vec::new();
    let first = path.first();
    let last = path.last();
    let mut mangled_span = 0..0;
    if let (Some(first), Some(last)) = (first, last) {
        let first_name_span = first.name.span();
        let last_name_span = last.name.span();
        let mut end = last_name_span.end;
        let start = first_name_span.start;
        if let Some(last) = last.template_args.as_ref().and_then(|x| x.last()) {
            end = last.span().end;
        }
        mangled_span = start..end;
    };
    for p in path.iter_mut() {
        let mut current = String::new();
        current.push_str(p.name.replace('_', "__").as_str());
        if let Some(args) = p.template_args.as_mut() {
            for arg in args.iter_mut() {
                current.push_str("__");
                mangle_expression(&arg.expression);
                current.push_str(format!("{}", arg.expression).as_str());
            }
        }
        result.push(current);
    }
    let joined = result.join("_");
    path.clear();
    path.push(PathPart {
        name: Spanned::new(joined, mangled_span),
        template_args: None,
        inline_template_args: None,
    });
}
