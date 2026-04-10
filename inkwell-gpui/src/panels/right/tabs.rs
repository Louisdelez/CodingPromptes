//! Right panel tab implementations — extracted from right_panel.rs
//! Each tab method is `impl RightPanel { fn tab_xxx() }`.

use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName};
use crate::store::StoreEvent;
use crate::state::*;
use crate::ui::colors::*;

use super::{RightPanel, lint, kpi, export_btn, time_btn};
// ── Tab implementations ──
impl RightPanel {
    pub(crate) fn tab_preview(&self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let compiled = s.cached_prompt.clone();
        let is_copied = self.copy_feedback > 0;
        div().flex_1().flex().flex_col()
            .child(div().px(px(16.0)).py(px(10.0)).flex().items_center().gap(px(8.0))
                .border_b_1().border_color(border_c())
                .child(Icon::new(IconName::File).text_color(text_muted()))
                .child(div().flex_1().text_xs().text_color(text_muted()).child("Prompt compile"))
                .child({ let cc = compiled.clone();
                    div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).flex().items_center().gap(px(4.0))
                        .text_xs().text_color(if is_copied { success() } else { accent() })
                        .child(Icon::new(if is_copied { IconName::Check } else { IconName::Copy }))
                        .child(if is_copied { "Copie !" } else { "Copier" })
                        .cursor_pointer().hover(|s| s.bg(accent_bg()))
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(cc.clone()));
                            this.copy_feedback = 120;
                        }))
                }))
            .child(div().flex_1().p(px(16.0))
                .text_xs().text_color(if compiled.is_empty() { text_muted() } else { text_primary() })
                .child(if compiled.is_empty() { "Commencez a ecrire dans les blocs pour voir le prompt compile...".into() } else { compiled }))
    }

    pub(crate) fn tab_playground(&self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let response = s.playground_response.clone(); let loading = s.playground_loading;
        let selected = s.selected_model.clone();
        let last_exec = s.executions.last().cloned();

        const CLOUD_MODELS: &[&str] = &[
            "GPT-4o", "GPT-4o Mini", "GPT-4.1", "GPT-4.1 Mini",
            "GPT-4.1 Nano", "o3-mini", "Claude Sonnet 4.6",
            "Claude Opus 4.6", "Claude Haiku 4.5", "Gemini 2.5 Pro",
            "Gemini 2.5 Flash",
        ];

        // Execute + settings buttons
        let controls = div().flex().items_center().gap(px(8.0))
            .child(div().py(px(8.0)).px(px(16.0)).bg(if loading { text_muted() } else { accent() })
                .rounded(px(8.0)).flex().items_center().gap(px(6.0))
                .text_sm().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                .child(Icon::new(IconName::Play))
                .child(if loading { "Execution..." } else { "Executer" })
                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.store.update(cx, |s, _| { if s.playground_loading { return; } s.playground_loading = true; s.playground_response.clear(); });
                    let s = this.store.read(cx);
                    let prompt = s.cached_prompt.clone(); let model = s.selected_model.clone();
                    let server = s.server_url.clone(); let tx = s.msg_tx.clone();
                    let temp = s.playground_temperature; let max_tok = s.playground_max_tokens;
                    std::thread::spawn(move || { crate::app::rt().block_on(async {
                        let client = reqwest::Client::new();
                        let body = serde_json::json!({"model":model,"messages":[{"role":"user","content":prompt}],"temperature":temp,"max_tokens":max_tok,"stream":false});
                        if let Ok(resp) = crate::app::llm_post(&client, &model, &server, body).send().await {
                            if let Ok(data) = resp.json::<serde_json::Value>().await {
                                let text = crate::llm::parse_llm_response(&model, &data).unwrap_or_default();
                                let _ = tx.send(AsyncMsg::LlmResponse(text));
                            }
                        }
                        let _ = tx.send(AsyncMsg::LlmDone);
                    }); });
                })))
            .child(div().p(px(8.0)).rounded(px(8.0)).border_1().border_color(border_c())
                .child(Icon::new(IconName::Settings).text_color(text_muted()))
                .cursor_pointer().hover(|s| s.bg(bg_tertiary())));

        // Cloud models chips
        let mut cloud_chips = div().flex().flex_wrap().gap(px(6.0));
        for &m in CLOUD_MODELS {
            let model_id: String = m.to_lowercase().replace(' ', "-");
            let is_sel = selected == model_id || selected == m.to_lowercase().replace(' ', "-");
            let mid = model_id.clone();
            cloud_chips = cloud_chips.child(
                div().px(px(10.0)).py(px(4.0)).rounded(px(12.0))
                    .text_xs().cursor_pointer()
                    .bg(if is_sel { accent() } else { bg_tertiary() })
                    .text_color(if is_sel { gpui::hsla(0.0, 0.0, 1.0, 1.0) } else { text_secondary() })
                    .border_1().border_color(if is_sel { accent() } else { border_c() })
                    .hover(|s| s.bg(if is_sel { accent() } else { bg_secondary() }))
                    .child(m.to_string())
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        this.store.update(cx, |s, _| { s.selected_model = mid.clone(); }); cx.notify();
                    }))
            );
        }

        div().flex_1().p(px(16.0)).flex().flex_col().gap(px(12.0))
            .child(controls)
            .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(text_muted()).child("MODELES CLOUD (API)"))
            .child(cloud_chips)
            .child(div().flex_1().p(px(12.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                .text_xs().text_color(if response.is_empty() { text_muted() } else { text_primary() })
                .child(if response.is_empty() { "Selectionnez un ou plusieurs modeles et cliquez sur Executer".into() } else { response }))
            .children(last_exec.map(|e| div().flex().items_center().gap(px(8.0)).flex_wrap()
                .child(div().text_xs().text_color(accent()).child(format!("{}ms", e.latency_ms)))
                .child(div().text_xs().text_color(success()).child(format!("{}/{} tok", e.tokens_in, e.tokens_out)))))
    }

    pub(crate) fn tab_chat(&self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let messages: Vec<(String, String)> = s.chat_messages.clone();
        let model = s.selected_model.clone();
        let system_prompt = s.chat_system_prompt.clone();

        // Messages area
        let mut msg_view = div().flex().flex_col().gap(px(8.0));
        for (role, content) in &messages {
            let is_user = role == "user";
            let row = if is_user {
                div().flex().w_full().justify_end()
            } else {
                div().flex().w_full()
            };
            msg_view = msg_view.child(
                row.child(div().max_w(px(280.0)).px(px(12.0)).py(px(8.0)).rounded(px(12.0))
                    .bg(if is_user { accent() } else { bg_tertiary() })
                    .text_xs().text_color(if is_user { gpui::hsla(0.0, 0.0, 1.0, 1.0) } else { text_primary() })
                    .child(content.clone()))
            );
        }

        div().flex_1().flex().flex_col()
            // Header with model + clear
            .child(div().px(px(12.0)).py(px(8.0)).border_b_1().border_color(border_c()).flex().items_center().gap(px(6.0))
                .child(div().text_xs().text_color(text_muted()).child("Modele:"))
                .child(div().px(px(8.0)).py(px(3.0)).rounded(px(6.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                    .text_xs().text_color(text_secondary()).child(model))
                .child(div().flex_1())
                .children(if messages.is_empty() { None } else {
                    Some(div().p(px(4.0)).rounded(px(4.0)).cursor_pointer().hover(|s| s.bg(bg_tertiary()))
                        .child(Icon::new(IconName::Trash2).text_color(text_muted()))
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                            this.store.update(cx, |s, _| { s.chat_messages.clear(); }); cx.notify();
                        })))
                }))
            // System prompt hint
            .child(div().px(px(12.0)).py(px(6.0)).border_b_1().border_color(border_c()).flex().items_center().gap(px(4.0))
                .child(div().text_xs().text_color(text_muted()).child("Systeme:"))
                .child(div().flex_1().text_xs().text_color(text_secondary()).overflow_hidden()
                    .child(if system_prompt.is_empty() { "(prompt courant)".to_string() } else {
                        if system_prompt.len() > 40 { format!("{}...", &system_prompt[..40]) } else { system_prompt }
                    }))
                .child(div().px(px(6.0)).py(px(2.0)).rounded(px(4.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                    .text_xs().text_color(accent()).child(Icon::new(IconName::File))
                    .cursor_pointer().hover(|s| s.bg(bg_secondary()))))
            // Messages
            .child(div().flex_1().p(px(12.0)).child(msg_view)
                .child(if messages.is_empty() { div().text_xs().text_color(text_muted()).child("Commencez une conversation...") } else { div() }))
            // Input area
            .child(div().px(px(12.0)).py(px(8.0)).border_t_1().border_color(border_c()).flex().items_center().gap(px(6.0))
                .child(if let Some(ref e) = self.chat_input { div().flex_1().child(Input::new(e)) } else { div().flex_1() })
                .child(div().px(px(8.0)).py(px(6.0)).rounded(px(6.0)).bg(accent())
                    .child(Icon::new(IconName::Play).text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0)))
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                        let raw_msg = this.chat_input.as_ref().map(|e| e.read(cx).value().to_string()).unwrap_or_default();
                        if raw_msg.is_empty() { return; }
                        // Enrich with context providers (#codebase, #file, #git, #steering)
                        let steering_ctx = this.store.read(cx).steering.get_context(None);
                        let msg = crate::kiro::context::build_contextual_prompt(&raw_msg, &steering_ctx);
                        this.store.update(cx, |s, _| { s.chat_messages.push(("user".into(), raw_msg.clone())); });
                        // Re-create input for next message
                        this.chat_input = Some(cx.new(|cx| InputState::new(window, cx).placeholder("Envoyer un message...")));
                        cx.notify();
                        let msgs: Vec<serde_json::Value> = this.store.read(cx).chat_messages.iter()
                            .map(|(r, c)| serde_json::json!({"role":r,"content":c})).collect();
                        let server = this.store.read(cx).server_url.clone();
                        let tx = this.store.read(cx).msg_tx.clone();
                        std::thread::spawn(move || { crate::app::rt().block_on(async {
                            let client = reqwest::Client::new();
                            let body = serde_json::json!({"model":"gpt-4o-mini","messages":msgs,"temperature":0.7,"max_tokens":2048,"stream":false});
                            if let Ok(resp) = crate::app::llm_post(&client, "gpt-4o-mini", &server, body).send().await {
                                if let Ok(data) = resp.json::<serde_json::Value>().await {
                                    let text = crate::llm::parse_llm_response("gpt-4o-mini", &data).unwrap_or_default();
                                    let _ = tx.send(AsyncMsg::LlmResponse(format!("__CHAT__{text}")));
                                }
                            }
                        }); });
                    }))))
    }

    pub(crate) fn tab_stt(&self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let recording = s.stt_recording;
        let provider = s.stt_provider;

        const PROVIDERS: &[(&str, &str, SttProvider)] = &[
            ("Local", "Utilise le serveur GPU local", SttProvider::Local),
            ("OpenAI Whisper", "API cloud OpenAI", SttProvider::OpenaiWhisper),
            ("Groq Whisper", "API cloud Groq (rapide)", SttProvider::Groq),
            ("Deepgram Nova-3", "API cloud Deepgram", SttProvider::Deepgram),
        ];

        const LANGUAGES: &[&str] = &["Auto", "FR", "EN", "ES", "DE", "IT", "PT", "NL", "JA", "ZH", "KO", "RU", "AR"];

        // Provider selection
        let mut providers = div().flex().flex_col().gap(px(4.0));
        for &(label, desc, prov) in PROVIDERS {
            let is_sel = provider == prov;
            providers = providers.child(
                div().px(px(12.0)).py(px(8.0)).rounded(px(6.0))
                    .border_1().border_color(if is_sel { accent() } else { border_c() })
                    .bg(if is_sel { accent_bg() } else { bg_tertiary() })
                    .cursor_pointer().hover(|s| s.bg(bg_secondary()))
                    .flex().flex_col().gap(px(2.0))
                    .child(div().flex().items_center().gap(px(6.0))
                        .child(Icon::new(if prov == SttProvider::Local { IconName::Cpu } else { IconName::Globe }).text_color(if is_sel { accent() } else { text_muted() }))
                        .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(if is_sel { accent() } else { text_primary() }).child(label.to_string())))
                    .child(div().text_xs().text_color(text_muted()).child(desc.to_string()))
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        this.store.update(cx, |s, _| { s.stt_provider = prov; }); cx.notify();
                    }))
            );
        }

        // Language selector
        let mut lang_chips = div().flex().flex_wrap().gap(px(4.0));
        for &lang in LANGUAGES {
            let is_sel = lang == "Auto"; // default
            lang_chips = lang_chips.child(
                div().px(px(8.0)).py(px(3.0)).rounded(px(10.0))
                    .text_xs().cursor_pointer()
                    .bg(if is_sel { accent() } else { bg_tertiary() })
                    .text_color(if is_sel { gpui::hsla(0.0, 0.0, 1.0, 1.0) } else { text_secondary() })
                    .border_1().border_color(if is_sel { accent() } else { border_c() })
                    .child(lang.to_string())
            );
        }

        div().flex_1().p(px(16.0)).flex().flex_col().gap(px(12.0))
            .child(div().flex().items_center().gap(px(6.0))
                .child(Icon::new(IconName::Mic).text_color(text_muted()))
                .child(div().text_xs().text_color(text_muted()).child("Speech-to-Text")))
            // Provider selection
            .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(text_muted()).child("FOURNISSEUR"))
            .child(providers)
            // Language
            .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(text_muted()).child("LANGUE"))
            .child(lang_chips)
            // Status
            .child(div().p(px(10.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c()).flex().items_center().gap(px(6.0))
                .child(div().w(px(6.0)).h(px(6.0)).rounded(px(3.0)).bg(if recording { danger() } else { success() }))
                .child(div().text_xs().text_color(text_primary()).child(if recording { "Enregistrement..." } else { "Pret" })))
            // Usage hint
            .child(div().p(px(10.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c()).flex().flex_col().gap(px(4.0))
                .child(div().text_xs().text_color(text_primary()).child("Utilisation"))
                .child(div().text_xs().text_color(text_secondary()).child("Cliquez sur l'icone Mic dans l'en-tete d'un bloc pour dicter. Cliquez a nouveau pour arreter.")))
    }

    pub(crate) fn tab_optimize(&self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let response = s.playground_response.clone();
        let loading = s.playground_loading;
        let has_result = response.starts_with("--- Optimise ---");

        div().flex_1().p(px(16.0)).flex().flex_col().gap(px(12.0))
            .child(div().flex().items_center().gap(px(6.0))
                .child(Icon::new(IconName::Sparkles).text_color(text_muted()))
                .child(div().flex_1().text_xs().text_color(text_muted()).child("Optimiseur IA"))
                // Improve button (purple gradient style)
                .child(div().py(px(6.0)).px(px(14.0)).rounded(px(6.0))
                    .bg(hsla(270.0/360.0, 0.7, 0.5, 1.0))
                    .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                    .flex().items_center().gap(px(4.0))
                    .child(Icon::new(IconName::Sparkles))
                    .child(if loading { "Amelioration..." } else { "Ameliorer" })
                    .cursor_pointer().hover(|s| s.bg(hsla(270.0/360.0, 0.7, 0.45, 1.0)))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.store.update(cx, |s, _| { if s.playground_loading { return; } s.playground_loading = true; s.playground_response.clear(); });
                        let s = this.store.read(cx);
                        let prompt = s.cached_prompt.clone(); let server = s.server_url.clone(); let tx = s.msg_tx.clone();
                        std::thread::spawn(move || { crate::app::rt().block_on(async {
                            let client = reqwest::Client::new();
                            let body = serde_json::json!({"model":"gpt-4o-mini","messages":[
                                {"role":"system","content":"You are a prompt engineering expert. Rewrite the prompt to be clearer, more specific, and effective. Keep the same intent."},
                                {"role":"user","content":format!("Optimize this prompt:\n\n{prompt}")}
                            ],"temperature":0.3,"max_tokens":4096,"stream":false});
                            if let Ok(resp) = crate::app::llm_post(&client, "gpt-4o-mini", &server, body).send().await {
                                if let Ok(data) = resp.json::<serde_json::Value>().await {
                                    let text = crate::llm::parse_llm_response("gpt-4o-mini", &data).unwrap_or_default();
                                    let _ = tx.send(AsyncMsg::LlmResponse(format!("--- Optimise ---\n{text}")));
                                }
                            }
                            let _ = tx.send(AsyncMsg::LlmDone);
                        }); });
                    }))))
            .child(div().text_xs().text_color(text_secondary()).child("Ameliorez votre prompt avec l'IA. L'optimiseur reecrit pour plus de clarte et d'efficacite."))
            // Optimized result
            .child(if has_result {
                let optimized = response.strip_prefix("--- Optimise ---\n").unwrap_or(&response).to_string();
                div().flex().flex_col().gap(px(8.0))
                    .child(div().flex_1().p(px(12.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                        .text_xs().text_color(text_primary()).child(optimized))
                    .child(div().py(px(6.0)).px(px(12.0)).rounded(px(6.0)).bg(success())
                        .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0)).flex().items_center().justify_center().gap(px(4.0))
                        .child(Icon::new(IconName::Check)).child("Appliquer")
                        .cursor_pointer().hover(|s| s.bg(hsla(120.0/360.0, 0.6, 0.35, 1.0))))
            } else {
                div().flex_1().p(px(12.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                    .text_xs().text_color(text_muted())
                    .child("Le prompt optimise apparaitra ici...")
            })
    }

    pub(crate) fn tab_lint(&self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let blocks = &s.project.blocks;
        let enabled = blocks.iter().filter(|b| b.enabled).count();
        let empty = blocks.iter().filter(|b| b.enabled && b.content.trim().is_empty()).count();
        let has_task = blocks.iter().any(|b| b.enabled && b.block_type == inkwell_core::types::BlockType::Task);
        let unresolved = s.cached_prompt.matches("{{").count();
        let chars = s.cached_chars; let has_neg = s.cached_prompt.contains("don't") || s.cached_prompt.contains("never");
        let has_ex = blocks.iter().any(|b| b.block_type == inkwell_core::types::BlockType::Examples && b.enabled);
        let mut checks = div().flex().flex_col().gap(px(6.0));
        if enabled == 0 { checks = checks.child(lint("error", "Aucun bloc active")); }
        if empty > 0 { checks = checks.child(lint("warning", &format!("{empty} bloc(s) vide(s)"))); }
        if !has_task && enabled > 0 { checks = checks.child(lint("warning", "Pas de bloc tache/directive")); }
        if unresolved > 0 { checks = checks.child(lint("warning", &format!("{unresolved} variable(s) non resolue(s)"))); }
        if chars < 50 && enabled > 0 { checks = checks.child(lint("info", "Prompt tres court")); }
        if chars > 10000 { checks = checks.child(lint("warning", "Prompt tres long (>10K car.)")); }
        if has_neg { checks = checks.child(lint("info", "Instructions negatives — preferez le positif")); }
        if !has_ex && chars > 800 { checks = checks.child(lint("info", "Prompt complexe sans exemples")); }
        let all_good = enabled > 0 && empty == 0 && has_task && unresolved == 0 && chars >= 50 && chars <= 10000;
        if all_good {
            checks = checks.child(lint("success", "Tous les checks sont passes !"));
        }
        div().flex_1().p(px(16.0)).flex().flex_col().gap(px(10.0))
            .child(div().flex().items_center().gap(px(6.0))
                .child(Icon::new(if all_good { IconName::Check } else { IconName::TriangleAlert })
                    .text_color(if all_good { success() } else { text_muted() }))
                .child(div().text_xs().text_color(text_muted()).child("Linting")))
            .child(checks)
    }

    pub(crate) fn tab_fleet(&self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let nodes = s.gpu_nodes.clone(); let server = s.server_url.clone();
        let node_count = nodes.len();
        let mut c = div().flex_1().p(px(16.0)).flex().flex_col().gap(px(8.0));
        c = c.child(div().flex().items_center().gap(px(6.0))
            .child(Icon::new(IconName::Globe).text_color(text_muted()))
            .child(div().text_xs().text_color(text_muted()).child("GPU Fleet"))
            .child(div().px(px(6.0)).py(px(2.0)).rounded(px(8.0)).bg(bg_tertiary()).text_xs().text_color(text_secondary())
                .child(format!("{}", if node_count == 0 { 1 } else { node_count })))
            .child(div().flex_1())
            .child(div().p(px(4.0)).rounded(px(4.0)).cursor_pointer().hover(|s| s.bg(bg_tertiary()))
                .child(Icon::new(IconName::Redo).text_color(text_muted()))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    let token = this.store.read(cx).session.as_ref().map(|s| s.token.clone()).unwrap_or_default();
                    if !token.is_empty() {
                        let server = this.store.read(cx).server_url.clone();
                        let tx = this.store.read(cx).msg_tx.clone();
                        crate::app::rt().spawn(async move {
                            let mut client = inkwell_core::api_client::ApiClient::new(&server);
                            client.set_token(token);
                            if let Ok(nodes) = client.list_nodes().await { let _ = tx.send(AsyncMsg::NodesLoaded(nodes)); }
                        });
                    }
                }))));
        if nodes.is_empty() {
            c = c.child(div().p(px(12.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c()).flex().flex_col().gap(px(6.0))
                .child(div().flex().items_center().gap(px(6.0))
                    .child(Icon::new(IconName::Wifi).text_color(success()))
                    .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(text_primary()).child("Serveur local")))
                .child(div().text_xs().text_color(text_muted()).child(server))
                .child(div().flex().gap(px(4.0))
                    .child(div().px(px(6.0)).py(px(2.0)).rounded(px(4.0)).bg(accent_bg()).text_xs().text_color(accent()).child("STT"))
                    .child(div().px(px(6.0)).py(px(2.0)).rounded(px(4.0)).bg(accent_bg()).text_xs().text_color(accent()).child("LLM"))));
        } else {
            for node in &nodes {
                let online = node.status == "online";
                c = c.child(div().p(px(12.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c()).flex().flex_col().gap(px(6.0))
                    .child(div().flex().items_center().gap(px(6.0))
                        .child(Icon::new(if online { IconName::Wifi } else { IconName::WifiOff }).text_color(if online { success() } else { text_muted() }))
                        .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(text_primary()).child(node.name.clone()))
                        .child(div().flex_1())
                        .child(div().p(px(4.0)).rounded(px(4.0)).cursor_pointer().hover(|s| s.bg(bg_secondary()))
                            .child(Icon::new(IconName::Trash2).text_color(text_muted()))))
                    .child(if !node.gpu_info.is_empty() {
                        div().text_xs().text_color(text_secondary()).child(node.gpu_info.clone())
                    } else { div() })
                    .child(div().flex().items_center().gap(px(6.0))
                        .child(div().px(px(6.0)).py(px(2.0)).rounded(px(8.0))
                            .bg(if online { hsla(120.0/360.0, 0.5, 0.2, 0.3) } else { hsla(0.0, 0.5, 0.2, 0.3) })
                            .text_xs().text_color(if online { success() } else { danger() })
                            .child(node.status.clone())))
                    .child(div().text_xs().text_color(text_muted()).overflow_hidden().child(node.address.clone())));
            }
        }
        c
    }

    pub(crate) fn tab_terminal(&self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let output = s.terminal_sessions.get(s.active_terminal).map(|t| t.output.clone()).unwrap_or_default();
        div().flex_1().flex().flex_col()
            .child(div().flex_1().p(px(8.0)).bg(hsla(0.0, 0.0, 0.04, 1.0))
                .text_xs().text_color(hsla(120.0 / 360.0, 0.8, 0.6, 1.0))
                .child(if output.is_empty() { "Demarrez un terminal depuis le menu principal".into() } else {
                    let lines: Vec<&str> = output.lines().collect();
                    let start = if lines.len() > 50 { lines.len() - 50 } else { 0 };
                    lines[start..].join("\n")
                }))
    }

    pub(crate) fn tab_export(&self, cx: &mut Context<Self>) -> Div {
        let is_copied = self.copy_feedback > 0;
        div().flex_1().p(px(16.0)).flex().flex_col().gap(px(10.0))
            // Import section
            .child(div().flex().items_center().gap(px(6.0))
                .child(Icon::new(IconName::Upload).text_color(text_muted()))
                .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(text_muted()).child("Import")))
            .child(export_btn("Importer JSON", "Charger un projet depuis un fichier .json"))
            // Divider
            .child(div().h(px(1.0)).bg(border_c()))
            // Export section
            .child(div().flex().items_center().gap(px(6.0))
                .child(Icon::new(IconName::Download).text_color(text_muted()))
                .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(text_muted()).child("Export")))
            .child(export_btn("TXT (.txt)", "Export en texte brut")
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    let s = this.store.read(cx);
                    let content = s.cached_prompt.clone(); let name = s.project.name.clone(); drop(s);
                    std::thread::spawn(move || { let _ = std::fs::write(format!("{}.txt", name.replace(' ', "-").to_lowercase()), &content); });
                })))
            .child(export_btn("Markdown (.md)", "Export en fichier Markdown")
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    let s = this.store.read(cx);
                    let content = s.cached_prompt.clone(); let name = s.project.name.clone(); drop(s);
                    std::thread::spawn(move || { let _ = std::fs::write(format!("{}.md", name.replace(' ', "-").to_lowercase()), &content); });
                })))
            .child(export_btn("JSON", "Export complet du projet")
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    let s = this.store.read(cx);
                    let blocks: Vec<inkwell_core::types::PromptBlock> = s.project.blocks.iter().map(|b|
                        inkwell_core::types::PromptBlock { id: b.id.clone(), block_type: b.block_type, content: b.content.clone(), enabled: b.enabled }).collect();
                    let name = s.project.name.clone(); drop(s);
                    std::thread::spawn(move || { let _ = std::fs::write(format!("{}.json", name.replace(' ', "-").to_lowercase()), serde_json::to_string_pretty(&blocks).unwrap_or_default()); });
                })))
            .child(export_btn("OpenAI JSON", "Format API OpenAI"))
            .child(export_btn("Anthropic JSON", "Format API Anthropic"))
            // Divider
            .child(div().h(px(1.0)).bg(border_c()))
            .child(div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c()).bg(bg_tertiary())
                .flex().items_center().gap(px(6.0)).cursor_pointer().hover(|s| s.bg(bg_secondary()))
                .child(Icon::new(if is_copied { IconName::Check } else { IconName::Copy }).text_color(if is_copied { success() } else { accent() }))
                .child(div().text_xs().text_color(if is_copied { success() } else { text_secondary() })
                    .child(if is_copied { "Copie !" } else { "Copier dans le presse-papier" }))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    let compiled = this.store.read(cx).cached_prompt.clone();
                    cx.write_to_clipboard(ClipboardItem::new_string(compiled));
                    this.copy_feedback = 120;
                })))
    }

    pub(crate) fn tab_history(&self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let execs: Vec<Execution> = s.executions.iter().rev().take(20).cloned().collect();
        let mut c = div().flex_1().p(px(16.0)).flex().flex_col().gap(px(6.0));
        c = c.child(div().flex().items_center().gap(px(6.0))
            .child(Icon::new(IconName::Clock).text_color(text_muted()))
            .child(div().flex_1().text_xs().text_color(text_muted()).child("Historique des executions"))
            .child(div().p(px(4.0)).rounded(px(4.0)).cursor_pointer().hover(|s| s.bg(bg_tertiary()))
                .child(Icon::new(IconName::Trash2).text_color(text_muted()))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.store.update(cx, |s, _| { s.executions.clear(); }); cx.notify();
                }))));
        if execs.is_empty() {
            c = c.child(div().text_xs().text_color(text_muted()).child("Aucune execution. Lancez un prompt dans le Playground."));
        } else {
            for exec in execs {
                let preview: String = if exec.response_preview.len() > 80 { format!("{}...", &exec.response_preview[..80]) } else { exec.response_preview.clone() };
                c = c.child(div().py(px(8.0)).border_b_1().border_color(border_c()).flex().flex_col().gap(px(4.0))
                    .child(div().flex().items_center().gap(px(6.0))
                        .child(Icon::new(IconName::ChevronRight).text_color(text_muted()))
                        .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(accent()).child(exec.model))
                        .child(div().flex_1())
                        .child(div().text_xs().text_color(text_muted()).child(
                            chrono::DateTime::from_timestamp_millis(exec.timestamp).map(|d| d.format("%H:%M:%S").to_string()).unwrap_or_default())))
                    .child(div().flex().gap(px(8.0))
                        .child(div().text_xs().text_color(success()).child(format!("{}ms", exec.latency_ms)))
                        .child(div().text_xs().text_color(text_secondary()).child(format!("{}/{} tok", exec.tokens_in, exec.tokens_out))))
                    .child(div().text_xs().text_color(text_muted()).overflow_hidden().child(preview)));
            }
        }
        c
    }

    pub(crate) fn tab_analytics(&self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let exec_count = s.executions.len();
        let total_in: u64 = s.executions.iter().map(|e| e.tokens_in).sum();
        let total_out: u64 = s.executions.iter().map(|e| e.tokens_out).sum();
        let total_cost: f64 = s.executions.iter().map(|e| e.cost).sum();
        let avg_lat = if exec_count > 0 { s.executions.iter().map(|e| e.latency_ms).sum::<u64>() / exec_count as u64 } else { 0 };
        // Per-model usage
        let mut model_counts: Vec<(String, usize)> = Vec::new();
        for exec in &s.executions {
            if let Some(entry) = model_counts.iter_mut().find(|(m, _)| m == &exec.model) {
                entry.1 += 1;
            } else { model_counts.push((exec.model.clone(), 1)); }
        }
        model_counts.sort_by(|a, b| b.1.cmp(&a.1));
        let max_count = model_counts.first().map(|(_, c)| *c).unwrap_or(1);

        // Time range buttons
        let time_range = div().flex().gap(px(4.0))
            .child(time_btn("7 jours", true))
            .child(time_btn("30 jours", false))
            .child(time_btn("Tout", false));

        let mut c = div().flex_1().p(px(16.0)).flex().flex_col().gap(px(12.0))
            .child(div().flex().items_center().gap(px(6.0))
                .child(Icon::new(IconName::ChartPie).text_color(text_muted()))
                .child(div().text_xs().text_color(text_muted()).child("Statistiques")))
            .child(time_range)
            .child(div().flex().flex_wrap().gap(px(8.0))
                .child(kpi("Executions", &exec_count.to_string(), accent()))
                .child(kpi("Tokens", &format!("{}", total_in + total_out), success()))
                .child(kpi("Cout", &format!("${:.4}", total_cost), hsla(50.0 / 360.0, 0.8, 0.5, 1.0)))
                .child(kpi("Latence moy.", &format!("{}ms", avg_lat), text_secondary())));

        // Top model
        if let Some((top_model, top_count)) = model_counts.first() {
            c = c.child(div().p(px(10.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                .flex().items_center().gap(px(8.0))
                .child(Icon::new(IconName::Sparkles).text_color(accent()))
                .child(div().text_xs().text_color(text_primary()).child(format!("{top_model} — {top_count} utilisations"))));
        }

        // Per-model bar chart
        if model_counts.len() > 1 {
            let mut bars = div().flex().flex_col().gap(px(6.0));
            for (model, count) in &model_counts {
                let pct = (*count as f32 / max_count as f32) * 100.0;
                bars = bars.child(div().flex().items_center().gap(px(8.0))
                    .child(div().w(px(80.0)).text_xs().text_color(text_secondary()).overflow_hidden().child(model.clone()))
                    .child(div().flex_1().h(px(8.0)).rounded(px(4.0)).bg(bg_tertiary())
                        .child(div().h(px(8.0)).rounded(px(4.0)).bg(accent())
                            .w(px(pct * 2.0))))
                    .child(div().w(px(24.0)).text_xs().text_color(text_muted()).child(count.to_string())));
            }
            c = c.child(bars);
        }
        c
    }

    pub(crate) fn tab_chain(&self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let workspaces = s.workspaces.clone();
        let blocks: Vec<(usize, String)> = s.project.blocks.iter().enumerate()
            .filter(|(_, b)| b.enabled && !b.content.is_empty())
            .map(|(i, b)| (i, if b.content.len() > 40 { format!("{}...", &b.content[..40]) } else { b.content.clone() }))
            .collect();
        let model = s.selected_model.clone();

        // Workspace selector
        let ws_selector = div().flex().flex_col().gap(px(4.0))
            .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(text_muted()).child("WORKSPACE"))
            .child(div().px(px(10.0)).py(px(6.0)).rounded(px(6.0)).border_1().border_color(border_c()).bg(bg_tertiary())
                .flex().items_center().gap(px(6.0))
                .child(div().text_xs().text_color(text_secondary()).child(
                    if workspaces.is_empty() { "Aucun workspace".to_string() } else { workspaces.first().map(|w| w.name.clone()).unwrap_or_default() }
                ))
                .child(div().flex_1())
                .child(Icon::new(IconName::ChevronDown).text_color(text_muted())));

        // Model selector
        let model_sel = div().flex().flex_col().gap(px(4.0))
            .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(text_muted()).child("MODELE"))
            .child(div().px(px(10.0)).py(px(6.0)).rounded(px(6.0)).border_1().border_color(border_c()).bg(bg_tertiary())
                .flex().items_center().gap(px(6.0))
                .child(div().text_xs().text_color(text_secondary()).child(model))
                .child(div().flex_1())
                .child(Icon::new(IconName::ChevronDown).text_color(text_muted())));

        // Steps list (numbered)
        let mut steps = div().flex().flex_col().gap(px(4.0));
        if blocks.is_empty() {
            steps = steps.child(div().text_xs().text_color(text_muted()).child("Ajoutez des blocs pour creer une chaine."));
        } else {
            for (idx, (_, preview)) in blocks.iter().enumerate() {
                steps = steps.child(div().px(px(10.0)).py(px(6.0)).rounded(px(6.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                    .flex().items_center().gap(px(8.0))
                    .child(div().w(px(20.0)).h(px(20.0)).rounded(px(10.0)).bg(accent()).flex().items_center().justify_center()
                        .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0)).child(format!("{}", idx + 1)))
                    .child(div().flex_1().text_xs().text_color(text_secondary()).overflow_hidden().child(preview.clone())));
            }
        }

        div().flex_1().p(px(16.0)).flex().flex_col().gap(px(12.0))
            .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(text_muted()).child("Prompt Chain"))
            .child(ws_selector)
            .child(model_sel)
            .child(steps)
            .child(div().py(px(8.0)).px(px(12.0)).rounded(px(6.0)).bg(accent())
                .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0)).flex().items_center().justify_center().gap(px(6.0))
                .child(Icon::new(IconName::Play)).child("Executer la chaine")
                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    let s = this.store.read(cx);
                    let blocks: Vec<String> = s.project.blocks.iter().filter(|b| b.enabled && !b.content.is_empty())
                        .map(|b| b.content.clone()).collect();
                    let _server = s.server_url.clone(); let tx = s.msg_tx.clone(); drop(s);
                    std::thread::spawn(move || { crate::app::rt().block_on(async {
                        let client = reqwest::Client::new(); let mut output = String::new();
                        for (i, content) in blocks.iter().enumerate() {
                            let prompt = if output.is_empty() { content.clone() } else { format!("Sortie precedente:\n{output}\n\nMaintenant:\n{content}") };
                            let body = serde_json::json!({"model":"gpt-4o-mini","messages":[{"role":"user","content":prompt}],"temperature":0.7,"max_tokens":2048,"stream":false});
                            if let Ok(resp) = crate::app::llm_post(&client, "gpt-4o-mini", "", body).send().await {
                                if let Ok(data) = resp.json::<serde_json::Value>().await {
                                    let text = crate::llm::parse_llm_response("gpt-4o-mini", &data).unwrap_or_default();
                                    output = text.clone();
                                    let _ = tx.send(AsyncMsg::LlmResponse(format!("--- Etape {} ---\n{text}", i + 1)));
                                }
                            }
                        }
                        let _ = tx.send(AsyncMsg::LlmDone);
                    }); });
                })))
    }

    pub(crate) fn tab_collab(&self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let session = s.session.clone(); let users = s.collab_users.clone();
        let online_count = users.iter().filter(|u| u.online).count() + if session.is_some() { 1 } else { 0 };
        let mut c = div().flex_1().p(px(16.0)).flex().flex_col().gap(px(10.0));
        // Header with refresh
        c = c.child(div().flex().items_center().gap(px(6.0))
            .child(Icon::new(IconName::User).text_color(text_muted()))
            .child(div().flex_1().text_xs().text_color(text_muted()).child("Collaboration"))
            .child(div().p(px(4.0)).rounded(px(4.0)).cursor_pointer().hover(|s| s.bg(bg_tertiary()))
                .child(Icon::new(IconName::Redo).text_color(text_muted()))));
        // Active users section
        c = c.child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(text_muted()).child("UTILISATEURS ACTIFS"));
        // Current user
        if let Some(ref ses) = session {
            let initial = ses.email.chars().next().unwrap_or('U').to_uppercase().to_string();
            c = c.child(div().p(px(10.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c()).flex().items_center().gap(px(8.0))
                .hover(|s| s.bg(bg_secondary()))
                .child(div().w(px(28.0)).h(px(28.0)).rounded(px(14.0)).bg(accent()).flex().items_center().justify_center()
                    .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0)).child(initial))
                .child(div().flex().flex_col()
                    .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(text_primary()).child(ses.display_name.clone()))
                    .child(div().text_xs().text_color(success()).child("En ligne (vous)"))));
        }
        // Other collaborators
        let colors = [accent(), success(), hsla(280.0/360.0, 0.7, 0.6, 1.0), hsla(50.0/360.0, 0.8, 0.5, 1.0)];
        for (i, user) in users.iter().enumerate() {
            c = c.child(div().p(px(10.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c()).flex().items_center().gap(px(8.0))
                .hover(|s| s.bg(bg_secondary()))
                .child(div().w(px(28.0)).h(px(28.0)).rounded(px(14.0)).bg(colors[i % colors.len()]).flex().items_center().justify_center()
                    .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0)).child(user.name.chars().next().unwrap_or('?').to_uppercase().to_string()))
                .child(div().flex().flex_col()
                    .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(text_primary()).child(user.name.clone()))
                    .child(div().text_xs().text_color(if user.online { success() } else { text_muted() }).child(if user.online { "En ligne" } else { "Hors ligne" }))));
        }
        if users.is_empty() && session.is_none() {
            c = c.child(div().flex().items_center().gap(px(6.0))
                .child(Icon::new(IconName::User).text_color(text_muted()))
                .child(div().text_xs().text_color(text_muted()).child("Connectez-vous pour voir les collaborateurs.")));
        } else if users.is_empty() {
            c = c.child(div().flex().items_center().gap(px(6.0))
                .child(Icon::new(IconName::User).text_color(text_muted()))
                .child(div().text_xs().text_color(text_muted()).child("Aucun autre collaborateur pour le moment.")));
        }
        // Active count indicator
        c = c.child(div().flex().items_center().gap(px(6.0))
            .child(div().w(px(6.0)).h(px(6.0)).rounded(px(3.0)).bg(success()))
            .child(div().text_xs().text_color(text_muted()).child(format!("{online_count} collaborateur(s) actif(s)"))));
        c
    }

    pub(crate) fn tab_sdd(&self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let sdd_running = s.sdd_running;
        let blocks = &s.project.blocks;

        // Count SDD block status
        let sdd_blocks: Vec<(usize, &str, bool)> = blocks.iter().enumerate()
            .filter(|(_, b)| b.block_type.is_sdd() && b.enabled)
            .map(|(i, b)| (i, b.block_type.label("fr"), !b.content.trim().is_empty()))
            .collect();

        let completed = sdd_blocks.iter().filter(|(_, _, done)| *done).count();
        let total = sdd_blocks.len();

        // Validate all SDD blocks
        let validation = crate::spec::workflow::validate_all(blocks);
        let total_issues: usize = validation.iter().map(|(_, issues)| issues.len()).sum();

        div().flex_1().p(px(16.0)).flex().flex_col().gap(px(12.0))
            // Header
            .child(div().flex().items_center().gap(px(6.0))
                .child(Icon::new(IconName::Scroll).text_color(text_muted()))
                .child(div().text_xs().text_color(text_muted()).child("Spec-Driven Development")))
            // Progress
            .child(div().p(px(12.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                .flex().flex_col().gap(px(8.0))
                .child(div().flex().items_center().gap(px(8.0))
                    .child(div().text_sm().font_weight(FontWeight::SEMIBOLD).text_color(text_primary())
                        .child(format!("{completed}/{total} phases")))
                    .child(div().flex_1())
                    .child(if sdd_running {
                        div().text_xs().text_color(warning()).child("Generation...")
                    } else if completed == total && total > 0 {
                        div().text_xs().text_color(success()).child("Complet")
                    } else {
                        div().text_xs().text_color(text_muted()).child("En attente")
                    }))
                // Progress bar
                .child(div().w_full().h(px(4.0)).rounded(px(2.0)).bg(border_c())
                    .child(div().h(px(4.0)).rounded(px(2.0))
                        .bg(if completed == total && total > 0 { success() } else { accent() })
                        .w(px(if total > 0 { completed as f32 / total as f32 * 200.0 } else { 0.0 })))))
            // Phase list
            .child({
                let mut phases = div().flex().flex_col().gap(px(4.0));
                for (_, label, done) in &sdd_blocks {
                    phases = phases.child(
                        div().px(px(10.0)).py(px(6.0)).rounded(px(6.0))
                            .bg(if *done { accent_bg() } else { bg_tertiary() })
                            .flex().items_center().gap(px(8.0))
                            .child(Icon::new(if *done { IconName::Check } else { IconName::Circle })
                                .text_color(if *done { success() } else { text_muted() }))
                            .child(div().text_xs().text_color(if *done { text_primary() } else { text_secondary() })
                                .child(label.to_string()))
                    );
                }
                phases
            })
            // Validation results
            .child(div().flex().items_center().gap(px(6.0))
                .child(Icon::new(if total_issues == 0 { IconName::Check } else { IconName::TriangleAlert })
                    .text_color(if total_issues == 0 { success() } else { warning() }))
                .child(div().text_xs().text_color(text_muted())
                    .child(if total_issues == 0 { "Validation OK".to_string() } else { format!("{total_issues} probleme(s) detecte(s)") })))
            // Validation details
            .children(if total_issues > 0 {
                let mut issues_div = div().flex().flex_col().gap(px(4.0));
                for (block_idx, issues) in &validation {
                    let label = blocks.get(*block_idx).map(|b| b.block_type.label("fr")).unwrap_or("?");
                    for issue in issues {
                        let (color, icon) = match issue.severity {
                            crate::spec::validator::Severity::Error => (danger(), IconName::Close),
                            crate::spec::validator::Severity::Warning => (warning(), IconName::TriangleAlert),
                            crate::spec::validator::Severity::Info => (text_muted(), IconName::Info),
                        };
                        issues_div = issues_div.child(
                            div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                                .bg(hsla(color.h, color.s, color.l, 0.1))
                                .flex().items_center().gap(px(6.0))
                                .child(Icon::new(icon).text_color(color))
                                .child(div().text_xs().text_color(color)
                                    .child(format!("[{}] {}", label, issue.message)))
                        );
                    }
                }
                Some(issues_div)
            } else { None })
            // Git integration
            .child(div().h(px(1.0)).bg(border_c()))
            .child(div().flex().gap(px(8.0))
                .child(div().px(px(10.0)).py(px(6.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).flex().items_center().gap(px(4.0))
                    .text_xs().text_color(text_secondary()).cursor_pointer()
                    .hover(|s| s.bg(bg_hover()))
                    .child(Icon::new(IconName::GitBranch)).child("Creer branche")
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        let s = this.store.read(cx);
                        let num = s.feature_counter;
                        let name = s.project.name.clone();
                        std::thread::spawn(move || {
                            let dir = std::env::current_dir().unwrap_or_default();
                            match crate::spec::git::create_feature_branch(&dir, num, &name) {
                                Ok(branch) => eprintln!("Created branch: {}", branch),
                                Err(e) => eprintln!("Git error: {}", e),
                            }
                        });
                    })))
                .child(div().px(px(10.0)).py(px(6.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).flex().items_center().gap(px(4.0))
                    .text_xs().text_color(text_secondary()).cursor_pointer()
                    .hover(|s| s.bg(bg_hover()))
                    .child(Icon::new(IconName::GitBranch)).child("Commit specs")
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        let name = this.store.read(cx).project.name.clone();
                        std::thread::spawn(move || {
                            let dir = std::env::current_dir().unwrap_or_default();
                            let msg = format!("spec: {}", name);
                            let _ = crate::spec::git::commit_specs(&dir, &msg);
                        });
                    }))))
            // Export buttons
            .child(div().h(px(1.0)).bg(border_c()))
            .child(div().flex().gap(px(8.0))
                .child(div().px(px(10.0)).py(px(6.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).flex().items_center().gap(px(4.0))
                    .text_xs().text_color(text_secondary()).cursor_pointer()
                    .hover(|s| s.bg(bg_hover()))
                    .child(Icon::new(IconName::Download)).child("Export .specify/")
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        let s = this.store.read(cx);
                        let blocks: Vec<crate::types::Block> = s.project.blocks.clone();
                        let name = s.project.name.clone();
                        std::thread::spawn(move || {
                            let dir = dirs::document_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                            let _ = crate::spec::export::export_speckit(&blocks, &name, &dir);
                        });
                    })))
                .child(div().px(px(10.0)).py(px(6.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).flex().items_center().gap(px(4.0))
                    .text_xs().text_color(text_secondary()).cursor_pointer()
                    .hover(|s| s.bg(bg_hover()))
                    .child(Icon::new(IconName::Download)).child("Export .kiro/")
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        let s = this.store.read(cx);
                        let blocks: Vec<crate::types::Block> = s.project.blocks.clone();
                        let name = s.project.name.clone();
                        std::thread::spawn(move || {
                            let dir = dirs::document_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                            let _ = crate::spec::export::export_kiro(&blocks, &name, &dir);
                        });
                    })))
            ) // close export flex container
            // Import buttons
            .child(div().flex().gap(px(8.0))
                .child(div().px(px(10.0)).py(px(6.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).flex().items_center().gap(px(4.0))
                    .text_xs().text_color(text_secondary()).cursor_pointer()
                    .hover(|s| s.bg(bg_hover()))
                    .child(Icon::new(IconName::Upload)).child("Import .specify/")
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        let dir = dirs::document_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                        let imported = crate::spec::export::import_speckit(&dir);
                        if !imported.is_empty() {
                            this.store.update(cx, |s, cx| {
                                for (bt, content) in imported {
                                    s.project.blocks.push(crate::types::Block {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        block_type: bt, content, enabled: true, editing: false,
                                    });
                                }
                                s.prompt_dirty = true;
                                cx.emit(StoreEvent::ProjectChanged);
                            });
                        }
                    })))
                .child(div().px(px(10.0)).py(px(6.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).flex().items_center().gap(px(4.0))
                    .text_xs().text_color(text_secondary()).cursor_pointer()
                    .hover(|s| s.bg(bg_hover()))
                    .child(Icon::new(IconName::Upload)).child("Import .kiro/")
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        let dir = dirs::document_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                        let imported = crate::spec::export::import_kiro(&dir);
                        if !imported.is_empty() {
                            this.store.update(cx, |s, cx| {
                                for (bt, content) in imported {
                                    s.project.blocks.push(crate::types::Block {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        block_type: bt, content, enabled: true, editing: false,
                                    });
                                }
                                s.prompt_dirty = true;
                                cx.emit(StoreEvent::ProjectChanged);
                            });
                        }
                    })))
            )
            // Extensions section
            .child(div().h(px(1.0)).bg(border_c()))
            .child({
                let extensions = &self.store.read(cx).extensions;
                let mut ext_section = div().flex().flex_col().gap(px(4.0))
                    .child(div().text_xs().font_weight(FontWeight::SEMIBOLD).text_color(text_muted()).child("Extensions"));
                for ext in &extensions.extensions {
                    ext_section = ext_section.child(
                        div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(bg_tertiary())
                            .flex().items_center().gap(px(6.0))
                            .child(div().w(px(6.0)).h(px(6.0)).rounded(px(3.0))
                                .bg(if ext.enabled { success() } else { text_muted() }))
                            .child(div().flex_1().text_xs().text_color(text_primary()).child(ext.name.clone()))
                            .child(div().text_xs().text_color(text_muted()).child(ext.version.clone()))
                    );
                }
                ext_section
            })
    }
}

