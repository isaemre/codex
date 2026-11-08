//! Makas onboarding for the gateway key and the model choice (K-30).
//!
//! Everything Makas-specific is gated on `codex_makas_config::LOCKED`. In an unlocked build
//! the sign-in picker, the OpenAI wording and the upstream model path stay in place, so the
//! upstream test suite keeps its expectations.

use std::sync::Arc;
use std::sync::PoisonError;
use std::sync::RwLock;

use codex_app_server_client::AppServerRequestHandle;
use codex_http_client::HttpClientFactory;
use codex_makas_models::ModelsError;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::widgets::WidgetRef;
use ratatui::widgets::Wrap;

use crate::bottom_pane::picker_option_list;
use crate::bottom_pane::render_menu_surface;
use crate::config_update::build_model_selection_edits;
use crate::config_update::write_config_batch;
use crate::key_hint;
use crate::key_hint::KeyBindingListExt;
use crate::render::Insets;
use crate::render::renderable::FlexRenderable;
use crate::render::renderable::Renderable;
use crate::render::renderable::RenderableExt as _;
use crate::render::renderable::RenderableItem;
use crate::tui::FrameRequester;

use super::auth::ApiKeyInputState;
use super::auth::SignInState;
use super::keys;
use super::onboarding_screen::KeyboardHandler;
use super::onboarding_screen::StepState;
use super::onboarding_screen::StepStateProvider;
use super::provider_setup::FieldDisplay;
use super::provider_setup::ProviderInput;

/// Model ids the gateway listed for the key checked on the key step. `None` means the list is
/// unavailable, so the model step asks for a name instead.
pub(crate) type GatewayModels = Arc<RwLock<Option<Vec<String>>>>;

pub(crate) fn new_gateway_models() -> GatewayModels {
    Arc::new(RwLock::new(None))
}

/// What the key step needs to check a gateway key before it is saved.
#[derive(Clone)]
pub(crate) struct GatewayKeyCheck {
    pub(crate) factory: HttpClientFactory,
    pub(crate) models: GatewayModels,
}

/// Makas is locked to one gateway, so its first screen is the gateway key rather than the picker.
pub(crate) fn initial_sign_in_state() -> SignInState {
    if codex_makas_config::LOCKED {
        SignInState::ApiKeyEntry(ApiKeyInputState::default())
    } else {
        SignInState::PickMode
    }
}

/// The model step runs only in locked builds, and only while the config names no model.
pub(crate) fn model_step_needed(config_model: Option<&str>) -> bool {
    codex_makas_config::LOCKED && config_model.is_none()
}

pub(crate) fn product_name() -> &'static str {
    if codex_makas_config::LOCKED {
        "Makas"
    } else {
        "Codex"
    }
}

pub(crate) fn welcome_tagline() -> &'static str {
    if codex_makas_config::LOCKED {
        ", your company's coding agent"
    } else {
        ", OpenAI's command-line coding agent"
    }
}

pub(crate) fn api_key_intro() -> &'static str {
    if codex_makas_config::LOCKED {
        "Enter the API key issued for Makas"
    } else {
        "Use your own OpenAI API key for usage-based billing"
    }
}

/// Snapshot names in tests differ between locked and unlocked builds because the copy differs.
#[cfg(test)]
pub(crate) fn snapshot_prefix() -> &'static str {
    if codex_makas_config::LOCKED {
        "makas_"
    } else {
        ""
    }
}

pub(crate) fn gateway_error_message(err: &ModelsError) -> String {
    match err {
        ModelsError::Unauthorized => {
            "The gateway rejected this API key. Check the key and try again.".to_string()
        }
        ModelsError::Unreachable => {
            "Could not reach the gateway. Check your network connection and try again.".to_string()
        }
        ModelsError::BadResponse => "The gateway returned an unexpected response.".to_string(),
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum SaveState {
    #[default]
    Idle,
    Saving,
    Saved,
    Failed(String),
}

/// Lets the user pick a gateway model, or type one when the gateway did not list any.
pub(crate) struct ModelSelectWidget {
    request_frame: FrameRequester,
    app_server_request_handle: Option<AppServerRequestHandle>,
    gateway_models: GatewayModels,
    highlighted: usize,
    manual: ProviderInput,
    save: Arc<RwLock<SaveState>>,
}

impl ModelSelectWidget {
    pub(crate) fn new(
        request_frame: FrameRequester,
        app_server_request_handle: Option<AppServerRequestHandle>,
        gateway_models: GatewayModels,
    ) -> Self {
        Self {
            request_frame,
            app_server_request_handle,
            gateway_models,
            highlighted: 0,
            manual: ProviderInput::default(),
            save: Arc::new(RwLock::new(SaveState::Idle)),
        }
    }

    fn model_ids(&self) -> Vec<String> {
        self.gateway_models
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
            .unwrap_or_default()
    }

    fn save_state(&self) -> SaveState {
        self.save
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn is_busy(&self) -> bool {
        matches!(self.save_state(), SaveState::Saving | SaveState::Saved)
    }

    fn clear_failure(&self) {
        let mut save = self.save.write().unwrap_or_else(PoisonError::into_inner);
        if matches!(*save, SaveState::Failed(_)) {
            *save = SaveState::Idle;
        }
    }

    fn set_failure(&self, message: &str) {
        *self.save.write().unwrap_or_else(PoisonError::into_inner) =
            SaveState::Failed(message.to_string());
        self.request_frame.schedule_frame();
    }

    /// `Some(has_text)` while the free-text model field is the active input.
    pub(crate) fn manual_text_entry(&self) -> Option<bool> {
        if self.is_busy() || !self.model_ids().is_empty() {
            return None;
        }
        Some(self.manual.value().is_some())
    }

    fn submit(&mut self, model: String) {
        let Some(handle) = self.app_server_request_handle.clone() else {
            self.set_failure("Model settings are unavailable in this session.");
            return;
        };
        *self.save.write().unwrap_or_else(PoisonError::into_inner) = SaveState::Saving;
        let save = self.save.clone();
        let request_frame = self.request_frame.clone();
        tokio::spawn(async move {
            let next = match write_config_batch(
                handle,
                build_model_selection_edits(&model, None::<String>),
            )
            .await
            {
                Ok(_) => SaveState::Saved,
                Err(err) => SaveState::Failed(format!("Could not save the model: {err}")),
            };
            *save.write().unwrap_or_else(PoisonError::into_inner) = next;
            request_frame.schedule_frame();
        });
        self.request_frame.schedule_frame();
    }

    fn status_line(&self) -> Option<Line<'static>> {
        match self.save_state() {
            SaveState::Idle => None,
            SaveState::Saving => Some("Saving…".dim().into()),
            SaveState::Saved => Some("✓ Model saved".green().into()),
            SaveState::Failed(message) => Some(message.red().into()),
        }
    }
}

impl KeyboardHandler for ModelSelectWidget {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if key_event.kind != KeyEventKind::Press || self.is_busy() {
            return;
        }
        let models = self.model_ids();
        if models.is_empty() {
            if keys::CONFIRM.is_pressed(key_event) {
                match self.manual.value() {
                    Some(model) => self.submit(model),
                    None => self.set_failure("Enter a model name."),
                }
            } else {
                self.manual.input(key_event);
                self.clear_failure();
            }
        } else if keys::MOVE_UP.is_pressed(key_event) {
            self.highlighted = (self.highlighted + models.len() - 1) % models.len();
            self.clear_failure();
        } else if keys::MOVE_DOWN.is_pressed(key_event) {
            self.highlighted = (self.highlighted + 1) % models.len();
            self.clear_failure();
        } else if keys::CONFIRM.is_pressed(key_event)
            && let Some(model) = models.get(self.highlighted).cloned()
        {
            self.submit(model);
        }
        self.request_frame.schedule_frame();
    }

    fn handle_paste(&mut self, pasted: String) {
        if self.manual_text_entry().is_none() {
            return;
        }
        self.manual.paste(&pasted);
        self.clear_failure();
        self.request_frame.schedule_frame();
    }
}

impl StepStateProvider for ModelSelectWidget {
    fn get_step_state(&self) -> StepState {
        if matches!(self.save_state(), SaveState::Saved) {
            StepState::Complete
        } else {
            StepState::InProgress
        }
    }
}

impl WidgetRef for ModelSelectWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let models = self.model_ids();
        let mut column = FlexRenderable::new();
        column.push(/*flex*/ 1, RenderableItem::Borrowed(&""));
        column.push(
            /*flex*/ 0,
            Paragraph::new("Choose the model".bold())
                .wrap(Wrap { trim: false })
                .inset(Insets::vh(/*v*/ 0, /*h*/ 2)),
        );
        if models.is_empty() {
            column.push(
                /*flex*/ 0,
                Paragraph::new("The gateway did not list models. Type the model name.".dim())
                    .wrap(Wrap { trim: false })
                    .inset(Insets::vh(/*v*/ 0, /*h*/ 2)),
            );
            column.push(
                /*flex*/ 0,
                Paragraph::new(self.manual.line(
                    "> ",
                    "model name",
                    FieldDisplay::Focused,
                    area.width.saturating_sub(4),
                ))
                .wrap(Wrap { trim: false })
                .inset(Insets::vh(/*v*/ 0, /*h*/ 2)),
            );
        } else {
            let labels: Vec<Line<'static>> = models
                .iter()
                .map(|model| Line::from(model.clone()))
                .collect();
            column.push(
                /*flex*/ 0,
                picker_option_list(labels, self.highlighted.min(models.len() - 1)),
            );
        }
        if let Some(status) = self.status_line() {
            column.push(
                /*flex*/ 0,
                Paragraph::new(status)
                    .wrap(Wrap { trim: false })
                    .inset(Insets::vh(/*v*/ 0, /*h*/ 2)),
            );
        }
        column.push(/*flex*/ 1, RenderableItem::Borrowed(&""));
        let footer = if models.is_empty() {
            Line::from(vec![key_hint::plain(KeyCode::Enter).into(), " save".dim()])
        } else {
            Line::from(vec![
                "↑/↓".bold(),
                " choose · ".dim(),
                key_hint::plain(KeyCode::Enter).into(),
                " save".dim(),
            ])
        };
        column.push(
            /*flex*/ 0,
            Paragraph::new(footer)
                .wrap(Wrap { trim: false })
                .inset(Insets::vh(/*v*/ 0, /*h*/ 2)),
        );
        column.push(/*flex*/ 1, RenderableItem::Borrowed(&""));
        let panel = Rect {
            height: column.desired_height(area.width).min(area.height),
            ..area
        };
        render_menu_surface(panel, buf);
        column.render(panel, buf);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use crossterm::event::KeyModifiers;
    use pretty_assertions::assert_eq;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn widget_with_models(models: Option<Vec<String>>) -> ModelSelectWidget {
        let gateway_models = new_gateway_models();
        *gateway_models.write().unwrap() = models;
        ModelSelectWidget::new(FrameRequester::test_dummy(), None, gateway_models)
    }

    #[test]
    fn list_selection_wraps_and_stays_in_range() {
        let mut widget = widget_with_models(Some(vec!["a".into(), "b".into()]));
        widget.handle_key_event(key(KeyCode::Down));
        assert_eq!(widget.highlighted, 1);
        widget.handle_key_event(key(KeyCode::Down));
        assert_eq!(widget.highlighted, 0);
        widget.handle_key_event(key(KeyCode::Up));
        assert_eq!(widget.highlighted, 1);
    }

    #[test]
    fn confirming_without_a_save_handle_reports_the_failure() {
        let mut widget = widget_with_models(Some(vec!["a".into()]));
        widget.handle_key_event(key(KeyCode::Enter));
        assert_eq!(
            widget.save_state(),
            SaveState::Failed("Model settings are unavailable in this session.".into())
        );
        assert_eq!(widget.get_step_state(), StepState::InProgress);
    }

    #[test]
    fn manual_entry_takes_printable_keys_including_quit_letters() {
        let mut widget = widget_with_models(None);
        widget.handle_key_event(key(KeyCode::Char('q')));
        widget.handle_key_event(key(KeyCode::Char('j')));
        assert_eq!(widget.manual.value().as_deref(), Some("qj"));
        assert_eq!(widget.manual_text_entry(), Some(true));
    }

    #[test]
    fn empty_manual_entry_is_rejected() {
        let mut widget = widget_with_models(None);
        widget.handle_key_event(key(KeyCode::Enter));
        assert_eq!(
            widget.save_state(),
            SaveState::Failed("Enter a model name.".into())
        );
    }

    #[test]
    fn list_mode_hides_the_text_field() {
        let widget = widget_with_models(Some(vec!["a".into()]));
        assert_eq!(widget.manual_text_entry(), None);
    }

    #[test]
    fn initial_sign_in_state_matches_the_build() {
        let state = initial_sign_in_state();
        if codex_makas_config::LOCKED {
            assert!(matches!(state, SignInState::ApiKeyEntry(_)));
        } else {
            assert!(matches!(state, SignInState::PickMode));
        }
    }

    #[test]
    fn gateway_errors_have_plain_messages() {
        assert_eq!(
            gateway_error_message(&ModelsError::Unauthorized),
            "The gateway rejected this API key. Check the key and try again."
        );
        assert_eq!(
            gateway_error_message(&ModelsError::Unreachable),
            "Could not reach the gateway. Check your network connection and try again."
        );
    }

    fn render_to_snapshot(widget: &ModelSelectWidget, width: u16, height: u16) -> String {
        let area = Rect::new(/*x*/ 0, /*y*/ 0, width, height);
        let mut terminal = crate::custom_terminal::Terminal::with_options(
            crate::test_backend::VT100Backend::new(width, height),
        )
        .expect("terminal");
        terminal.set_viewport_area(area);
        terminal
            .draw(|frame| widget.render_ref(area, frame.buffer_mut()))
            .expect("draw");
        terminal.backend().to_string()
    }

    #[test]
    fn model_list_snapshot() {
        let widget = widget_with_models(Some(vec![
            "qwen2.5-coder-32b".into(),
            "gpt-oss-120b".into(),
        ]));
        insta::assert_snapshot!("model_list", render_to_snapshot(&widget, 60, 12));
    }

    #[test]
    fn model_manual_snapshot() {
        let widget = widget_with_models(None);
        insta::assert_snapshot!("model_manual", render_to_snapshot(&widget, 60, 12));
    }
}
