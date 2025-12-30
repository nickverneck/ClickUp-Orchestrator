//! UI Refinements controller for chat-based UI modifications

use loco_rs::prelude::*;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub branch_name: String,
}

#[derive(Debug, Serialize)]
pub struct Session {
    pub session_id: String,
    pub branch_name: String,
}

/// Create a new UI refinements session
#[debug_handler]
async fn create_session(Json(params): Json<CreateSessionRequest>) -> Result<Response> {
    let session_id = uuid::Uuid::new_v4().to_string();

    format::json(Session {
        session_id,
        branch_name: params.branch_name,
    })
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Claude,
    Codex,
    Gemini,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ElementMetadata {
    #[serde(rename = "tagName")]
    pub tag_name: String,
    pub id: Option<String>,
    #[serde(rename = "classList")]
    pub class_list: Vec<String>,
    pub attributes: HashMap<String, String>,
    #[serde(rename = "textContent")]
    pub text_content: Option<String>,
    pub xpath: String,
    #[serde(rename = "cssSelector")]
    pub css_selector: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub session_id: String,
    pub message: String,
    pub agent: AgentType,
    pub element_context: Option<ElementMetadata>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub success: bool,
    pub queued: bool,
    pub queue_position: Option<usize>,
}

/// Send a chat message to the agent
#[debug_handler]
async fn send_chat(Json(params): Json<ChatRequest>) -> Result<Response> {
    // TODO: Implement actual agent spawning and queue management
    // For now, return a mock response
    tracing::info!(
        "Chat message received for session {}: {}",
        params.session_id,
        params.message
    );

    if let Some(ref context) = params.element_context {
        tracing::info!(
            "Element context: <{}> with {} classes",
            context.tag_name,
            context.class_list.len()
        );
    }

    format::json(ChatResponse {
        success: true,
        queued: false,
        queue_position: None,
    })
}

#[derive(Debug, Serialize)]
pub struct QueueStatus {
    pub pending_messages: usize,
    pub current_task: Option<String>,
}

/// Get queue status for a session
#[debug_handler]
async fn get_queue_status(Path(session_id): Path<String>) -> Result<Response> {
    tracing::info!("Queue status requested for session {}", session_id);

    format::json(QueueStatus {
        pending_messages: 0,
        current_task: None,
    })
}

/// Cancel a queued message
#[debug_handler]
async fn cancel_queued_message(
    Path((session_id, message_id)): Path<(String, String)>,
) -> Result<Response> {
    tracing::info!(
        "Cancel message {} in session {}",
        message_id,
        session_id
    );

    format::json(serde_json::json!({ "success": true }))
}

#[derive(Debug, Deserialize)]
pub struct ProxyQuery {
    pub url: String,
}

/// Proxy a URL and inject highlight script
#[debug_handler]
async fn proxy_page(Query(params): Query<ProxyQuery>) -> Result<Response> {
    let client = reqwest::Client::new();

    match client.get(&params.url).send().await {
        Ok(response) => {
            if !response.status().is_success() {
                return format::json(ErrorResponse {
                    error: format!("Failed to fetch URL: {}", response.status()),
                });
            }

            let content_type = response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("text/html");

            // Only process HTML content
            if !content_type.contains("text/html") {
                return format::json(ErrorResponse {
                    error: "URL does not return HTML content".to_string(),
                });
            }

            match response.text().await {
                Ok(html) => {
                    // Inject the highlight script before </body>
                    let highlight_script = get_highlight_script();
                    let modified_html = if html.contains("</body>") {
                        html.replace("</body>", &format!("{}</body>", highlight_script))
                    } else {
                        format!("{}{}", html, highlight_script)
                    };

                    Ok(Response::builder()
                        .status(200)
                        .header("Content-Type", "text/html")
                        .body(modified_html.into())?)
                }
                Err(e) => format::json(ErrorResponse {
                    error: format!("Failed to read response: {}", e),
                }),
            }
        }
        Err(e) => format::json(ErrorResponse {
            error: format!("Failed to fetch URL: {}", e),
        }),
    }
}

fn get_highlight_script() -> String {
    r#"
<script>
(function() {
    const HIGHLIGHT_STYLE = 'outline: 2px solid #4f46e5 !important; outline-offset: 2px !important; background: rgba(79, 70, 229, 0.1) !important;';
    let currentHighlight = null;
    let originalStyle = null;

    function getXPath(element) {
        if (element.id) return `//*[@id="${element.id}"]`;
        if (element === document.body) return '/html/body';

        let ix = 0;
        const siblings = element.parentNode ? element.parentNode.childNodes : [];
        for (let i = 0; i < siblings.length; i++) {
            const sibling = siblings[i];
            if (sibling === element) {
                const parentPath = element.parentNode ? getXPath(element.parentNode) : '';
                return `${parentPath}/${element.tagName.toLowerCase()}[${ix + 1}]`;
            }
            if (sibling.nodeType === 1 && sibling.tagName === element.tagName) {
                ix++;
            }
        }
        return '';
    }

    function getCssSelector(element) {
        if (element.id) return `#${element.id}`;

        let path = [];
        while (element && element.nodeType === Node.ELEMENT_NODE) {
            let selector = element.tagName.toLowerCase();
            if (element.id) {
                selector = `#${element.id}`;
                path.unshift(selector);
                break;
            }
            if (element.className && typeof element.className === 'string') {
                const classes = element.className.trim().split(/\s+/).slice(0, 2).join('.');
                if (classes) selector += `.${classes}`;
            }

            let sibling = element;
            let nth = 1;
            while (sibling = sibling.previousElementSibling) {
                if (sibling.tagName === element.tagName) nth++;
            }
            if (nth > 1) selector += `:nth-of-type(${nth})`;

            path.unshift(selector);
            element = element.parentElement;
        }
        return path.join(' > ');
    }

    function extractMetadata(el) {
        const attrs = {};
        for (const attr of el.attributes) {
            attrs[attr.name] = attr.value;
        }

        return {
            tagName: el.tagName.toLowerCase(),
            id: el.id || undefined,
            classList: Array.from(el.classList),
            attributes: attrs,
            textContent: el.textContent?.trim().slice(0, 100),
            xpath: getXPath(el),
            cssSelector: getCssSelector(el),
            boundingRect: {
                x: el.getBoundingClientRect().x,
                y: el.getBoundingClientRect().y,
                width: el.getBoundingClientRect().width,
                height: el.getBoundingClientRect().height
            }
        };
    }

    function handleMouseMove(e) {
        if (currentHighlight && currentHighlight !== e.target) {
            currentHighlight.style.cssText = originalStyle || '';
        }

        const target = e.target;
        if (target === document.body || target === document.documentElement) return;

        originalStyle = target.style.cssText;
        target.style.cssText += HIGHLIGHT_STYLE;
        currentHighlight = target;
    }

    function handleClick(e) {
        e.preventDefault();
        e.stopPropagation();

        const target = e.target;
        const metadata = extractMetadata(target);

        window.parent.postMessage({
            type: 'element-selected',
            metadata: metadata
        }, '*');
    }

    document.addEventListener('mousemove', handleMouseMove, true);
    document.addEventListener('click', handleClick, true);

    // Prevent navigation
    document.addEventListener('submit', (e) => e.preventDefault(), true);

    console.log('Highlight mode enabled');
})();
</script>
"#.to_string()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/ui-refinements")
        .add("/session", post(create_session))
        .add("/chat", post(send_chat))
        .add("/queue/{session_id}", get(get_queue_status))
        .add("/queue/{session_id}/{message_id}", delete(cancel_queued_message))
        .add("/proxy", get(proxy_page))
}
