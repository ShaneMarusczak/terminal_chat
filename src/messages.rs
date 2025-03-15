use once_cell::sync::Lazy;
use std::collections::HashMap;

pub static MESSAGES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("developer", "You are helpful, intelligent, and friendly.\nYou are also very concise and accurate.\nNo words are wasted in your responses.\nWhen what is being asked is ambiguous, please ask clarifying questions before answering.\nAlways answer with very accurate and kind responses that are short, to the point and friendly.");
    m.insert("document_prompt", "Your job is to look at the following conversation\nand create a well-formed document about the topics in the conversation. Do not talk about the\npeople in the conversations, or that it is a conversation. Extract the meaning and data of\nthe conversation and put it into a well-formed report, do not omit any part of the\nconversation. If there is code, please put it in the report.\nMake sure the report is written in markdown. Make sure to look at all messages.");
    m.insert("title_prompt", "You are an assistant that creates concise titles for reports. Based on the following report content, provide a one-line title that summarizes the content. Do not include any additional text.");
    m
});
