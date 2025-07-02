use serde::{Deserialize, Serialize};
use serde_json::json;

wit_bindgen::generate!({
    world: "weather-service",
    path: "wit",
});

struct Component;

// Weather tool input schema
#[derive(Deserialize)]
struct GetWeatherInput {
    location: String,
    #[serde(default)]
    units: WeatherUnits,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum WeatherUnits {
    Celsius,
    Fahrenheit,
}

impl Default for WeatherUnits {
    fn default() -> Self {
        WeatherUnits::Fahrenheit
    }
}

// Weather response
#[derive(Serialize)]
struct WeatherInfo {
    location: String,
    temperature: i32,
    unit: String,
    conditions: String,
    humidity: u8,
    wind_speed: u8,
    wind_direction: String,
}

impl exports::spin::mcp_trigger::mcp_types::Guest for Component {
    fn handle_request(
        request: exports::spin::mcp_trigger::mcp_types::Request,
    ) -> exports::spin::mcp_trigger::mcp_types::Response {
        use exports::spin::mcp_trigger::mcp_types::*;
        
        match request {
            Request::ToolsList => {
                Response::ToolsList(vec![
                    Tool {
                        name: "get_weather".to_string(),
                        description: "Get current weather information for a location".to_string(),
                        input_schema: json!({
                            "type": "object",
                            "properties": {
                                "location": {
                                    "type": "string",
                                    "description": "The city or location to get weather for"
                                },
                                "units": {
                                    "type": "string",
                                    "enum": ["celsius", "fahrenheit"],
                                    "description": "Temperature units (default: fahrenheit)",
                                    "default": "fahrenheit"
                                }
                            },
                            "required": ["location"]
                        }).to_string(),
                    },
                    Tool {
                        name: "get_forecast".to_string(),
                        description: "Get weather forecast for the next 5 days".to_string(),
                        input_schema: json!({
                            "type": "object",
                            "properties": {
                                "location": {
                                    "type": "string",
                                    "description": "The city or location to get forecast for"
                                },
                                "days": {
                                    "type": "integer",
                                    "description": "Number of days to forecast (1-5)",
                                    "minimum": 1,
                                    "maximum": 5,
                                    "default": 3
                                }
                            },
                            "required": ["location"]
                        }).to_string(),
                    }
                ])
            }
            
            Request::ToolsCall(params) => {
                match params.name.as_str() {
                    "get_weather" => handle_get_weather(&params.arguments),
                    "get_forecast" => handle_get_forecast(&params.arguments),
                    _ => Response::ToolsCall(ToolResult::Error(Error {
                        code: -32602,
                        message: format!("Unknown tool: {}", params.name),
                        data: None,
                    }))
                }
            }
            
            Request::ResourcesList => {
                // This example doesn't provide resources
                Response::ResourcesList(vec![])
            }
            
            Request::PromptsList => {
                Response::PromptsList(vec![
                    Prompt {
                        name: "weather_report".to_string(),
                        description: Some("Generate a detailed weather report".to_string()),
                        arguments: vec![
                            PromptArgument {
                                name: "location".to_string(),
                                description: Some("The location for the weather report".to_string()),
                                required: true,
                            }
                        ],
                    }
                ])
            }
            
            Request::PromptsGet(params) => {
                if params.name == "weather_report" {
                    let args: serde_json::Value = serde_json::from_str(&params.arguments)
                        .unwrap_or(json!({}));
                    let location = args.get("location")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown location");
                    
                    Response::PromptsGet(vec![
                        PromptMessage {
                            role: "system".to_string(),
                            content: "You are a professional meteorologist providing detailed weather reports.".to_string(),
                        },
                        PromptMessage {
                            role: "user".to_string(),
                            content: format!("Please provide a detailed weather report for {}, including current conditions, forecast, and any weather advisories.", location),
                        }
                    ])
                } else {
                    Response::Error(Error {
                        code: -32602,
                        message: format!("Unknown prompt: {}", params.name),
                        data: None,
                    })
                }
            }
            
            Request::Ping => Response::Pong,
            
            _ => Response::Error(Error {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            })
        }
    }
    
    fn initialize() -> Result<(), String> {
        // Initialization logic if needed
        Ok(())
    }
}

fn handle_get_weather(arguments: &str) -> exports::spin::mcp_trigger::mcp_types::Response {
    use exports::spin::mcp_trigger::mcp_types::*;
    
    let input: GetWeatherInput = match serde_json::from_str(arguments) {
        Ok(input) => input,
        Err(e) => return Response::ToolsCall(ToolResult::Error(Error {
            code: -32602,
            message: format!("Invalid arguments: {}", e),
            data: None,
        }))
    };
    
    // Simulate weather data (in a real implementation, this would call an API)
    let weather = WeatherInfo {
        location: input.location.clone(),
        temperature: match input.units {
            WeatherUnits::Celsius => 22,
            WeatherUnits::Fahrenheit => 72,
        },
        unit: match input.units {
            WeatherUnits::Celsius => "C".to_string(),
            WeatherUnits::Fahrenheit => "F".to_string(),
        },
        conditions: "Partly cloudy".to_string(),
        humidity: 65,
        wind_speed: 12,
        wind_direction: "NW".to_string(),
    };
    
    Response::ToolsCall(ToolResult::Json(
        serde_json::to_string(&weather).unwrap()
    ))
}

fn handle_get_forecast(arguments: &str) -> exports::spin::mcp_trigger::mcp_types::Response {
    use exports::spin::mcp_trigger::mcp_types::*;
    
    #[derive(Deserialize)]
    struct GetForecastInput {
        location: String,
        #[serde(default = "default_days")]
        days: u8,
    }
    
    fn default_days() -> u8 { 3 }
    
    let input: GetForecastInput = match serde_json::from_str(arguments) {
        Ok(input) => input,
        Err(e) => return Response::ToolsCall(ToolResult::Error(Error {
            code: -32602,
            message: format!("Invalid arguments: {}", e),
            data: None,
        }))
    };
    
    // Simulate forecast data
    let forecast = json!({
        "location": input.location,
        "forecast": (0..input.days).map(|day| {
            json!({
                "day": day + 1,
                "high": 75 + day,
                "low": 55 + day,
                "conditions": if day % 2 == 0 { "Sunny" } else { "Partly cloudy" },
                "precipitation_chance": day * 10
            })
        }).collect::<Vec<_>>()
    });
    
    Response::ToolsCall(ToolResult::Json(forecast.to_string()))
}