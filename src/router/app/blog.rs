use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Html,
};

use tera::Context;
use tokio::fs;

use std::{path::PathBuf, sync::Arc};

use crate::{AppState, User};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct BlogArticlePreview {
    title: String,
    description: String,
}

pub async fn blog(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<Option<User>>,
) -> Result<Html<String>, StatusCode> {
    // list all directories in ./templates/articles, extract a tuple (dir_name, serde_parsed dir_name/body.json)
    let mut previews: Vec<(String, BlogArticlePreview)> = Vec::new();
    let base_path = std::path::Path::new("./templates/articles");

    let mut entries = fs::read_dir(base_path).await.unwrap();

    async fn read_and_parse<T: DeserializeOwned>(
        path: PathBuf,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path).await?;
        let parsed = serde_json::from_str(&content)?;
        Ok(parsed)
    }

    while let Some(entry) = entries.next_entry().await.unwrap() {
        println!("entry: {:?}", entry);
        let path = entry.path();

        if path.is_dir() {
            println!("path: {:?}", path);
            // Construct the path to the body.json file within this directory
            let json_path = path.join("preview.json");

            // Parse the JSON file into a BlogArticlePreview
            match read_and_parse(json_path).await {
                Ok(parsed) => {
                    // If successful, push the directory name and parsed data into the vector
                    if let Some(dir_name) = path.file_name().and_then(|s| s.to_str()) {
                        let dir_name_owned = String::from(dir_name);
                        previews.push((dir_name_owned, parsed));
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing JSON for directory {:?}: {}", path, e);
                }
            }
        }
    }

    let mut context = Context::new();
    context.insert("previews", &previews);

    let home = state.tera.render("views/blog.html", &context).unwrap();

    let mut context = Context::new();
    context.insert("view", &home);
    context.insert("current_user", &current_user);
    context.insert("with_footer", &true);
    let rendered = state.tera.render("views/main.html", &context).unwrap();

    Ok(Html(rendered))
}

#[axum::debug_handler]
pub async fn blog_by_slug(
    Path(slug): Path<String>,
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<Option<User>>,
) -> Result<Html<String>, StatusCode> {
    let template = format!("articles/{}/body.md", slug);

    match state.tera.get_template(&template) {
        Ok(_) => {
            // Template exists
            let context = Context::new();
            let article = state.tera.render(&template, &context).unwrap();

            let article_html = comrak::markdown_to_html(&article, &comrak::Options::default());
            let mut context = Context::new();
            context.insert("article", &article_html);
            let blog = state
                .tera
                .render("components/article-wrapper.html", &context)
                .unwrap();

            let mut context = Context::new();
            context.insert("view", &blog);
            context.insert("current_user", &current_user);
            context.insert("with_footer", &true);
            let rendered = state.tera.render("views/main.html", &context).unwrap();

            Ok(Html(rendered))
        }
        Err(_) => {
            // Template does not exist
            Err(StatusCode::NOT_FOUND)
        }
    }
}
