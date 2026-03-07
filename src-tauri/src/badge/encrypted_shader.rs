pub fn get_shader_content() -> Result<String, String> {
    Ok(include_str!("../../../LucidGlass.shader").to_string())
}
