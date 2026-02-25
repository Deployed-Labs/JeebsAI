use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize)]
pub struct Thought {
    pub internal_monologue: String,
    pub detected_sentiment: f32,
    pub suggested_angle: String,
    pub curiosity_target: String,
}

#[derive(Serialize)]
pub struct ThoughtResponse {
    pub success: bool,
    pub thought: Option<Thought>,
}

#[get("/api/brain/thought/{session}")]
pub async fn get_thought(web::Path(session): web::Path<String>) -> impl Responder {
    // TODO: Replace with real JeebsAI cognitive state lookup
    let thought = Thought {
        internal_monologue: format!("Session {}: JeebsAI is thinking about knowledge expansion and user queries.", session),
        detected_sentiment: 0.42,
        suggested_angle: "curious".to_string(),
        curiosity_target: "AI learning methods".to_string(),
    };
    HttpResponse::Ok().json(ThoughtResponse {
        success: true,
        thought: Some(thought),
    })
}
