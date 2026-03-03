/// Enhanced Chat Handler
///
/// Integrates all intelligent systems:
/// - Conversation context tracking
/// - Intelligent inference
/// - Smart response generation
/// - Continuous learning
///
/// This is the main chat intelligence layer

use actix_session::Session;
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use serde_json::json;

use crate::chat_history;
use crate::conversation_context;
use crate::continuous_learning;
use crate::intelligent_inference;
use crate::smart_response;
use crate::state::AppState;
use crate::user_chat::UserChatRequest;

/// Enhanced smart chat endpoint - the primary intelligent interface
#[post("/api/chat/smart")]
pub async fn smart_chat(
    data: web::Data<AppState>,
    req: web::Json<UserChatRequest>,
    session: Session,
    http_req: HttpRequest,
) -> impl Responder {
    // Verify authentication
    let username = match extract_username(&session) {
        Some(u) => u,
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "error": "Not authenticated"
            }))
        }
    };

    let message = req.message.trim();
    if message.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "error": "Message cannot be empty"
        }));
    }

    let session_id = session.get::<String>("session_id").ok().flatten();

    // Step 1: Load conversation context
    let mut context = match conversation_context::load_conversation_context(
        &data.db,
        session_id.as_deref().unwrap_or("default"),
        Some(&username),
    )
    .await
    {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("[SmartChat] Context error: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Context error: {}", e)
            }));
        }
    };

    // Step 2: Analyze user message
    let user_intent = conversation_context::analyze_user_message(message);
    let detected_topic_shift = conversation_context::detect_topic_shift(&context, message);

    if detected_topic_shift {
        context.previous_topics.clear();
        println!("[SmartChat] Topic shift detected");
    }

    // Step 3: Build intelligent inference context
    match intelligent_inference::build_context(&data.db, message, Some(&username)).await {
        Ok(inference_context) => {
            // Step 4: Perform inference
            match intelligent_inference::infer_response(&inference_context).await {
                Ok(inference_result) => {
                    // Step 5: Extract facts from inference
                    let facts: Vec<String> = inference_context
                        .relevant_facts
                        .iter()
                        .map(|f| f.fact.clone())
                        .collect();

                    // Quality check: if we have no meaningful facts and low confidence, use fallback
                    if facts.is_empty() || (facts.iter().all(|f| f.len() < 10) && inference_context.confidence < 0.5) {
                        eprintln!("[SmartChat] Insufficient facts (count: {}, confidence: {}), using fallback",
                                   facts.len(), inference_context.confidence);
                        return fallback_response(message, &context.current_topic);
                    }

                    // Step 6: Generate smart response
                    let response_config =
                        smart_response::get_response_config_for_intent(&user_intent.primary);
                    let smart_response = smart_response::generate_smart_response(
                        facts,
                        inference_context.confidence,
                        &user_intent.primary,
                        &context.current_topic,
                        &response_config,
                    );

                    // Step 7: Store messages in chat history
                    let _ = chat_history::insert_chat_message(
                        &data.db,
                        session_id.as_deref(),
                        Some(&username),
                        "user",
                        message,
                    )
                    .await;

                    let _ = chat_history::insert_chat_message(
                        &data.db,
                        session_id.as_deref(),
                        None,
                        "jeebs",
                        &smart_response.text,
                    )
                    .await;

                    // Step 8: Log learning outcome for continuous learning
                    let _ = intelligent_inference::log_inference_outcome(
                        &data.db,
                        &inference_result,
                        None,
                    )
                    .await;

                    // Step 9: Update conversation context
                    let _ = conversation_context::save_conversation_state(&data.db, &context).await;

                    // Step 10: Return comprehensive response
                    HttpResponse::Ok().json(json!({
                        "response": smart_response.text,
                        "summary": smart_response.summary,
                        "confidence": smart_response.confidence,
                        "follow_up": smart_response.follow_up_suggestion,
                        "understanding": {
                            "intent": user_intent.primary,
                            "topic": context.current_topic,
                            "context_messages": context.messages.len()
                        },
                        "learning": {
                            "consolidated": true,
                            "confidence_update": inference_context.confidence
                        }
                    }))
                }
                Err(e) => {
                    eprintln!("[SmartChat] Inference error: {}", e);
                    fallback_response(message, &context.current_topic)
                }
            }
        }
        Err(e) => {
            eprintln!("[SmartChat] Context building error: {}", e);
            fallback_response(message, &context.current_topic)
        }
    }
}

/// Fallback response when intelligent system can't answer
fn fallback_response(question: &str, topic: &str) -> HttpResponse {
    // Generate helpful fallback that acknowledges the question while being honest
    let response = if question.contains('?') {
        if question.to_lowercase().contains("who") || question.to_lowercase().contains("where") {
            format!("I don't have specific information about that yet. Can you tell me what you know about {}?", topic)
        } else if question.to_lowercase().contains("how") {
            format!("I'm learning how to explain {}. What's your experience with this?", topic)
        } else if question.to_lowercase().contains("why") {
            format!("I haven't fully understood why {} works this way yet. What's your perspective?", topic)
        } else {
            format!("I don't have enough knowledge about {} to answer confidently. Help me learn by sharing what you know.", topic)
        }
    } else {
        // For statements, acknowledge and ask for more info
        if question.len() < 50 {
            format!("Noted: {}. I'd like to learn more about {}. Can you elaborate?",
                    question.trim_end_matches('?'), topic)
        } else {
            format!("I'm processing that. Can you help me understand {} better?", topic)
        }
    };

    HttpResponse::Ok().json(json!({
        "response": response,
        "summary": "Building knowledge",
        "confidence": 0.25,
        "follow_up": Some(format!("What else should I know about {}?", topic)),
        "understanding": {
            "intent": "learning",
            "topic": topic,
            "context_messages": 0
        },
        "note": "Brain database appears to be empty. Starting fresh knowledge acquisition."
    }))
}

fn extract_username(session: &Session) -> Option<String> {
    session
        .get::<String>("username")
        .ok()
        .flatten()
}
