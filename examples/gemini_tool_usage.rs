use anthropic_sdk::{
    GeminiClient, GeminiContent, GeminiFunctionCall, GeminiFunctionResponse, GeminiPart, GeminiTool,
};
use anyhow::Result;
use serde_json::json;
use std::env;

// Simulates external API calls with static data
async fn get_dummy_theater_data(location: &str, movie: Option<&str>) -> serde_json::Value {
    json!({
        "name": "find_theaters",
        "content": {
            "movie": movie.unwrap_or(""),
            "theaters": [{
                "name": "AMC Mountain View 16",
                "address": "2000 W El Camino Real, Mountain View, CA 94040"
            }, {
                "name": "Regal Edwards 14",
                "address": "245 Castro St, Mountain View, CA 94040"
            }]
        }
    })
}

async fn get_dummy_movie_data(location: &str, description: &str) -> serde_json::Value {
    json!({
        "name": "find_movies",
        "content": {
            "movies": [{
                "title": "Barbie",
                "genre": "Comedy",
                "rating": "PG-13"
            }, {
                "title": "Dune: Part Two",
                "genre": "Science Fiction",
                "rating": "PG-13"
            }]
        }
    })
}

async fn get_dummy_showtime_data(
    location: &str,
    movie: &str,
    theater: &str,
    date: &str,
) -> serde_json::Value {
    json!({
        "name": "get_showtimes",
        "content": {
            "showtimes": [
                "10:30 AM",
                "1:15 PM",
                "4:00 PM",
                "7:30 PM",
                "10:45 PM"
            ]
        }
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");

    // Initialize the client with configuration
    let mut client = GeminiClient::new()
        .auth(&api_key)
        .model("gemini-pro")
        .temperature(0.7)
        .max_output_tokens(1000)
        .stream(true);

    // Create function declarations for all three tools
    let find_movies = GeminiClient::function_declaration(
        "find_movies",
        "find movie titles currently playing in theaters based on any description, genre, title words, etc.",
        json!({
            "type": "OBJECT",
            "properties": {
                "location": {
                    "type": "STRING",
                    "description": "The city and state, e.g. San Francisco, CA or a zip code e.g. 95616"
                },
                "description": {
                    "type": "STRING",
                    "description": "Any kind of description including category or genre, title words, attributes, etc."
                }
            },
            "required": ["description"]
        }),
    );

    let find_theaters = GeminiClient::function_declaration(
        "find_theaters",
        "find theaters based on location and optionally movie title which is currently playing in theaters",
        json!({
            "type": "OBJECT",
            "properties": {
                "location": {
                    "type": "STRING",
                    "description": "The city and state, e.g. San Francisco, CA or a zip code e.g. 95616"
                },
                "movie": {
                    "type": "STRING",
                    "description": "Any movie title"
                }
            },
            "required": ["location"]
        }),
    );

    let get_showtimes = GeminiClient::function_declaration(
        "get_showtimes",
        "Find the start times for movies playing in a specific theater",
        json!({
            "type": "OBJECT",
            "properties": {
                "location": {
                    "type": "STRING",
                    "description": "The city and state, e.g. San Francisco, CA or a zip code e.g. 95616"
                },
                "movie": {
                    "type": "STRING",
                    "description": "Any movie title"
                },
                "theater": {
                    "type": "STRING",
                    "description": "Name of the theater"
                },
                "date": {
                    "type": "STRING",
                    "description": "Date for requested showtime"
                }
            },
            "required": ["location", "movie", "theater", "date"]
        }),
    );

    // Create tools array with all function declarations
    let tools = vec![GeminiTool {
        function_declarations: vec![find_movies, find_theaters, get_showtimes],
    }];

    // Set up the client with our tools
    client = client.tools(tools);

    // Create the conversation flow
    let contents = vec![
        // Initial user query
        GeminiContent {
            role: Some("user".to_string()),
            parts: vec![GeminiPart::Text {
                text: "Which theaters in Mountain View show Barbie movie?".to_string(),
            }],
        },
        // Model's function call
        GeminiContent {
            role: Some("model".to_string()),
            parts: vec![GeminiPart::FunctionCall {
                function_call: GeminiFunctionCall {
                    name: "find_theaters".to_string(),
                    args: json!({
                        "location": "Mountain View, CA",
                        "movie": "Barbie"
                    }),
                },
            }],
        },
        // Function response with theater data
        GeminiContent {
            role: Some("user".to_string()),
            parts: vec![GeminiPart::FunctionResponse {
                function_response: GeminiFunctionResponse {
                    name: "find_theaters".to_string(),
                    response: get_dummy_theater_data("Mountain View, CA", Some("Barbie")).await,
                },
            }],
        },
    ];

    // Process the response using streaming
    println!("Generating response...");
    let res = client.generate(contents).await?;

    // Print the response
    dbg!("{}", res);

    Ok(())
}
