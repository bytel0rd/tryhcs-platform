use tryhcs_shared::api_params::ErrorMessage;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use super::state_engine::{state_machine, StateAction, StateFeedBackTrait};
use js_sys::Function;

#[wasm_bindgen]
pub struct JsStateFeedBack {
    on_loading: Option<Function>,
    on_error: Option<Function>,
    on_success: Option<Function>,
}

#[wasm_bindgen]
impl JsStateFeedBack {
    #[wasm_bindgen(constructor)]
    pub fn new() -> JsStateFeedBack {
        JsStateFeedBack {
            on_loading: None,
            on_error: None,
            on_success: None,
        }
    }

    #[wasm_bindgen(js_name = setOnLoading)]
    pub fn set_on_loading(&mut self, callback: Function) {
        self.on_loading = Some(callback);
    }

    #[wasm_bindgen(js_name = setOnError)]
    pub fn set_on_error(&mut self, callback: Function) {
        self.on_error = Some(callback);
    }

    #[wasm_bindgen(js_name = setOnSuccess)]
    pub fn set_on_success(&mut self, callback: Function) {
        self.on_success = Some(callback);
    }
}

fn log_js_error(err_val: JsValue) {
    tracing::error!(
        "#JS function called failed error {}",
        err_val.as_string().unwrap_or("".into())
    );
}

const JS_SERIALIZATION_ERROR: &str = "#JS Serialization error";
const JS_ACTION_SERIALIZATION_ERROR: &str = "#JS action serialization error";
const JS_FUNCTION_CALLED_FAILED_ERROR: &str = "#JS function called failed error";

#[async_trait::async_trait(?Send)]
impl StateFeedBackTrait for JsStateFeedBack {
    async fn on_loading(&self) {
        if let Some(func) = &self.on_loading {
            if let Err(err) = func.call0(&JsValue::NULL) {
                log_js_error(err);
            }
            return;
        }
    }
    async fn on_success(&self, data: Box<dyn erased_serde::Serialize>) {
        match serde_wasm_bindgen::to_value(&data) {
            Ok(js_value) => {
                if let Some(func) = &self.on_success {
                    if let Err(err) = func.call1(&JsValue::NULL, &js_value) {
                        log_js_error(err);
                    }
                    return;
                }
            }
            Err(e) => {
                tracing::error!("JS Serialization Error: {}", e);
                self.on_error(JS_SERIALIZATION_ERROR.into()).await;
                return;
            }
        }
    }
    async fn on_error(&self, error: ErrorMessage) {
        match serde_wasm_bindgen::to_value(&error) {
            Ok(js_value) => {
                if let Some(func) = &self.on_error {
                    if let Err(err) = func.call1(&JsValue::NULL, &js_value) {
                        log_js_error(err);
                    }
                    return;
                }
            }
            Err(e) => {
                tracing::error!("JS Serialization Error: {}", e);
                self.on_error(JS_SERIALIZATION_ERROR.into()).await;
                return;
            }
        }
    }
}

#[wasm_bindgen]
pub async fn run_state_machine(action: JsValue, feedback: JsStateFeedBack) -> Result<(), JsValue> {
    let action: serde_json::Value = match serde_wasm_bindgen::from_value(action) {
        Ok(action) => action,
        Err(e) => {
            tracing::error!("Failed to deserialize action to serde_json value: {}", e);
            return Err(JS_ACTION_SERIALIZATION_ERROR.into());
        }
    };

    let action = serde_json::from_value(action).map_err(|e| {
        JsValue::from_str(&format!(
            "Failed to deserialize action to StateAction: {}",
            e
        ))
    })?;

    state_machine(action, Box::new(feedback)).await;
    Ok(())
}
