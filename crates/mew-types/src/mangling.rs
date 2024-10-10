use mew_parse::syntax::{Expression, PathPart};

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
    return mangle_template_args(path_part);
}

pub fn mangle_template_args(path_part: &PathPart) -> String {
    let name = &path_part.name.replace('_', "__");
    let mut template_args = String::new();
    for template_arg in path_part.template_args.iter().flatten() {
        template_args.push('_');
        template_args.push_str(&mangle_expression(&template_arg.expression).replace('_', "__"));
    }
    format!("{name}{template_args}")
}

pub fn mangle_inline_arg_name(
    enclosing_path: &Vec<PathPart>,
    parent_path: &Vec<PathPart>,
    template_arg_name: &String,
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
