use wesl_parse::syntax::{Expression, PathPart};

fn mangle_expression(expr: &Expression) -> String {
    let data: String = format!("{expr}").replace(' ', "").replace('\n', "");
    let mut result = String::new();
    for c in data.chars() {
        if c.is_alphanumeric() {
            result.push(c);
        } else {
            let mut buf = vec![0; c.len_utf8()];
            let _ = c.encode_utf8(&mut buf);
            result.push_str("__");
            for item in buf {
                let str = item.to_string();
                result.push_str(&str.len().to_string());
                result.push_str(&str);
            }
        }
    }
    result
}

pub fn mangle_template_args(path_part: &PathPart) -> String {
    if path_part.template_args.is_none() {
        return path_part.name.value.clone();
    }
    let name = &path_part.name.replace('_', "__");
    let mut template_args = String::new();
    for template_arg in path_part.template_args.iter().flatten() {
        template_args.push('_');
        template_args.push_str(&mangle_expression(&template_arg.expression));
    }
    format!("{name}{template_args}")
}
