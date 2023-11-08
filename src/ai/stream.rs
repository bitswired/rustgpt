use axum::Error;
use reqwest::header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest_eventsource::{Event as ReqwestEvent, EventSource as ReqwestEventSource};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use crate::data::model::ChatMessagePair;

// Define a struct to represent a model.
#[derive(Serialize, Deserialize, Debug)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}

// Define a struct to represent the list of models.
#[derive(Serialize, Deserialize, Debug)]
struct ModelList {
    object: String,
    data: Vec<Model>,
}

pub async fn list_engines(api_key: &str) -> Result<Vec<Model>, reqwest::Error> {
    let client = reqwest::Client::new();
    let res: ModelList = client
        .get("https://api.openai.com/v1/models")
        .bearer_auth(api_key)
        .send()
        .await?
        .json()
        .await?;

    Ok(res.data)
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

pub enum GenerationEvent {
    Text(String),
    End(String),
}

pub async fn generate_sse_stream(
    api_key: &str,
    model: &str,
    messages: Vec<ChatMessagePair>,
    sender: mpsc::Sender<Result<GenerationEvent, Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Your OpenAI API key

    // The API endpoint for chat completions
    let url = "https://api.openai.com/v1/chat/completions";

    let system_message = json!({"role": "system", "content": "You are a helpful assistant."});
    let system_message_iter = std::iter::once(Some(system_message));

    // Create an iterator over the messages
    let messages_iter = messages.iter().flat_map(|msg| {
        let user_message = Some(json!({
            "role": "user",
            "content": msg.human_message
        }));

        let ai_message = msg.ai_message.as_ref().map(|ai_msg| {
            json!({
                "role": "assistant",
                "content": ai_msg
            })
        });

        std::iter::once(user_message).chain(std::iter::once(ai_message))
    });

    println!("Model is: {}", model);
    println!("Model is: {}", model);
    println!("Model is: {}", model);
    println!("Model is: {}", model);

    // Chain the system message with the user and AI messages, filter out the Nones, and collect into a Vec<Value>
    let body_messages = system_message_iter
        .chain(messages_iter)
        .flatten() // This removes any None values
        .collect::<Vec<Value>>();

    // Prepare the request body
    let body = json!({
        "model": model,
        // "model": "gpt-4",
        "messages": body_messages,
        "stream": true
    });

    println!("body: {}", body);

    // Create a client
    let client = reqwest::Client::new();

    // Create a request
    let request = client
        .post(url)
        .header(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key))?,
        )
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .body(body.to_string());

    // Start streaming
    let mut stream = ReqwestEventSource::new(request)?;

    // Handle streaming events
    while let Some(event) = stream.next().await {
        match event {
            Ok(ReqwestEvent::Open) => println!("Connection Open!"),
            Ok(ReqwestEvent::Message(message)) => {
                if message.data.trim() == "[DONE]" {
                    println!("Stream completed.");
                    stream.close();
                    if sender
                        // .send(Ok(Event::default()
                        //     .data(r#"<div id="sse-listener" hx-swap-oob="true"></div>"#)))
                        .send(Ok(GenerationEvent::End(
                            r#"<div id="sse-listener" hx-swap-oob="true"></div>"#.to_string(),
                        )))
                        .await
                        .is_err()
                    {
                        break; // Receiver has dropped, stop sending.
                    }
                    break;
                } else {
                    let m: Value = serde_json::from_str(&message.data).unwrap();
                    if let Some(text) = m["choices"][0]["delta"]["content"].as_str() {
                        // let text = text.to_string().replace(' ', "&nbsp;");
                        // // print debug text
                        // println!("text: {:?}", text);
                        // println!("text: {}", text);

                        // if sender.send(Ok(Event::default().data(text))).await.is_err() {
                        if sender
                            .send(Ok(GenerationEvent::Text(text.to_string())))
                            .await
                            .is_err()
                        {
                            break; // Receiver has dropped, stop sending.
                        }
                    }
                }
            }
            Err(err) => {
                println!("Error: {}", err);
                stream.close();
                if sender.send(Err(axum::Error::new(err))).await.is_err() {
                    break; // Receiver has dropped, stop sending.
                }
            }
            _ => (),
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_something_async() {
    // Create a channel for sending SSE events
    let (sender, receiver) = mpsc::channel(10);

    // Convert the receiver end into a Stream
    let mut stream = ReceiverStream::new(receiver);

    // Call the function that generates the SSE stream
    let generator_handle = tokio::spawn(async move {
        generate_sse_stream("Hello".to_string(), sender)
            .await
            .unwrap();
    });

    while let Some(event) = stream.next().await {
        match event {
            Ok(sse_event) => {
                println!("Received event: {:?}", sse_event)
            }
            Err(e) => {}
        }
    }

    // Convert the Stream into an SSE response
    let sse_response = Sse::new(stream);

    // You can then test the SSE response by consuming the events from `sse_response`
    // For this test, we're just going to drop the SSE response
    drop(sse_response);

    // Wait for the generator to finish
    let result = generator_handle.await;

    // Assert that the SSE event generation function completed successfully
    assert!(result.is_ok());
}

// #[tokio::test]
// async fn test_request_to_example_com() {
//     let response = reqwest::get("http://example.com").await;

//     // Check if the request was successful
//     assert!(response.is_ok(), "Request should be successful");

//     let response = response.unwrap();

//     // Check if the HTTP status code is 200 OK
//     assert_eq!(
//         response.status(),
//         reqwest::StatusCode::OK,
//         "Response status should be 200 OK"
//     );

//     // Optionally, you can check the contents of the response
//     let body = response.text().await;
//     assert!(body.is_ok(), "Should be able to read the response body");

//     // Here you can perform more detailed checks on the `body` if necessary,
//     // such as checking for specific content.
// }
