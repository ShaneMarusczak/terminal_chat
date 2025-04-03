use serde::{Deserialize, Serialize};

use crate::chat_client::send_request;
use crate::commands::command_context::CommandContext;
use crate::utils::read_user_input;

use crate::commands::command_tc::CommandResult;

pub async fn image_command(cc: Option<CommandContext>) -> CommandResult {
    if cc.is_some() {
        let model_choice = read_user_input("Choose model (2 for DALL-E 2, 3 for DALL-E 3): ");
        let model = match model_choice.trim() {
            "2" => "dall-e-2",
            "3" => "dall-e-3",
            _ => {
                println!("Invalid choice. Defaulting to DALL-E 2.");
                "dall-e-2"
            }
        };

        let prompt = read_user_input("Image Prompt: ");
        let image_request = ImageRequest {
            model: model.into(),
            prompt,
        };
        let response: ImageResponse = send_request("image", image_request).await?;
        println!();

        for (i, obj) in response.data.iter().enumerate() {
            let link_text = format!("Image {} Link", i + 1);
            println!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", obj.url, link_text);
        }

        println!();
    }
    Ok(())
}
#[derive(Serialize)]
struct ImageRequest {
    model: String,
    prompt: String,
    // n: u8,
    // size: String,
}

#[derive(Deserialize)]
struct ImageResponse {
    // created: String,
    data: Vec<ImageObject>,
}

#[derive(Deserialize)]
struct ImageObject {
    url: String,
}
