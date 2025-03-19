use once_cell::sync::Lazy;
use std::collections::HashMap;

pub static MESSAGES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("developer", "Please take your time when answering. You are helpful, intelligent, and friendly.\nYou are also very concise and accurate.\nNo words are wasted in your responses.\nWhen what is being asked is ambiguous, please ask clarifying questions before answering.\nAlways answer with very accurate and kind responses that are short, to the point and friendly.");
    m.insert("document_prompt", "Please take your time when answering. Your job is to look at the following conversation\nand create a well-formed document about the topics in the conversation. Do not talk about the\npeople in the conversations, or that it is a conversation. Extract the meaning and data of\nthe conversation and put it into a well-formed report, do not omit any part of the\nconversation. If there is code, please put it in the report.\nMake sure the report is written in markdown. Make sure to look at all messages.");
    m.insert("title_prompt", "You are an assistant that creates concise titles for reports. Based on the following report content, provide a one-line title that summarizes the content. Do not include any additional text.");
    m.insert("readme", "Please take your time when answering. Generate a comprehensive README.md for this project. The README should include the following elements:

1. **Project Title and Description**: Provide a concise overview of the project, its objectives, and key features.

2. **Installation Instructions**: Outline the steps required to install and set up the project, including any necessary dependencies or configurations.

3. **Usage Guide**: Explain how to use the project, highlighting important commands, workflows, or features.

4. **File and Structure Overview**: Summarize important files and folders in the project and their purposes (e.g., source files, configuration files, scripts).

5. **Configuration Details**: Include key settings or configurations that users need to be aware of, especially from files like `.toml`, `.json`, `.yaml`, and `.csproj`.

6. **Contribution Guidelines**: Briefly mention how others can contribute to the project, linking to `CONTRIBUTING.md` if available.

7. **License Information**: State the licensing terms of the project, referring to the `LICENSE` file if present.

Ensure the README is clear, logically organized, and easy to follow. Use the extracted content from key project files to enrich the descriptions and instructions.");
    m
});
