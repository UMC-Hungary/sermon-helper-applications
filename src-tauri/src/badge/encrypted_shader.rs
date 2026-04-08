include!(concat!(env!("OUT_DIR"), "/shader_content.rs"));

pub fn get_shader_content() -> Result<String, String> {
    Ok(SHADER_CONTENT.to_string())
}
