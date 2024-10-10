static BUILTIN_FUNCTIONS: std::sync::OnceLock<wgsl_spec::FunctionInfo> = std::sync::OnceLock::new();
static BUILTIN_TOKENS: std::sync::OnceLock<wgsl_spec::TokenInfo> = std::sync::OnceLock::new();

pub fn get_builtin_functions() -> &'static wgsl_spec::FunctionInfo {
    BUILTIN_FUNCTIONS.get_or_init(|| {
        wgsl_spec::include::functions().expect("could not load builtin function defintitions")
    })
}
pub fn get_builtin_tokens() -> &'static wgsl_spec::TokenInfo {
    BUILTIN_TOKENS.get_or_init(|| {
        wgsl_spec::include::tokens().expect("could not load builtin token defintitions")
    })
}
