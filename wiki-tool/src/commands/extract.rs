use std::path::Path;

use super::Context;

/// Run the extract command.
pub fn run(ctx: &Context, file_path: &Path) -> wiki_tool::Result<()> {
    let full_path = if file_path.is_relative() {
        ctx.project_root.join(file_path)
    } else {
        file_path.to_path_buf()
    };

    if !full_path.exists() {
        return Err(wiki_tool::WikiToolError::FileNotFound(
            full_path.display().to_string(),
        ));
    }

    let text = wiki_tool::extract::extract_text(&full_path)?;

    if ctx.json {
        let result = serde_json::json!({
            "file": file_path.to_string_lossy(),
            "content": text,
            "length": text.len(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print!("{}", text);
    }

    Ok(())
}
