use pdfcraft_automation::{Automation, Content};
use serde_json::json;

#[test]
fn translation_requires_explicit_network_opt_in() {
    let mut automation = Automation::new();
    let error = automation.call("text_translate", &json!({"text": "Hello", "allow_network": false})).unwrap_err();
    assert!(error.to_string().contains("allow_network=true"));
    assert!(automation.call("text_translate", &json!({"text": "Hello"})).is_err());
    assert!(automation.call("text_translate", &json!({"text": " ", "allow_network": true})).is_err());
    assert!(automation.call("text_translate", &json!({"text": "a".repeat(3001), "allow_network": true})).is_err());
}

#[test]
#[ignore = "Explicit live test: sends only this synthetic sentence to DeepSeek"]
fn selected_text_translates_to_chinese_live() {
    let mut automation = Automation::new();
    let result = automation
        .call(
            "text_translate",
            &json!({
                "text": "The model predicts future values from historical observations. The recurrence is (x_{t+1} = a x_t).", "allow_network": true
            }),
        )
        .unwrap();
    let Some(Content::Json(value)) = result.first() else { panic!("expected JSON translation") };
    let translated = value["translation"].as_str().unwrap();
    assert!(translated.contains("模型"), "{translated}");
    assert!(translated.contains("预测"), "{translated}");
    assert_eq!(value["target"], "zh-CN");
    assert_eq!(value["model"], "deepseek-flash");
    assert!(translated.contains("x_{t+1}"), "{translated}");
    assert!(automation.session().docs().is_empty());
}

#[test]
fn translation_math_renders_offline_through_automation() {
    let mut automation = Automation::new();
    let result =
        automation.call("text_render_math", &json!({"text": r"递推关系：\[x_{t+1}=\alpha x_t+\frac{1}{n}\sum_{i=1}^{n}\varepsilon_i\]"})).unwrap();
    assert!(result.iter().any(|part| matches!(part, Content::Png { width, height, .. } if *width > 100 && *height > 30)));
    assert!(automation.session().docs().is_empty());
    assert!(automation.call("text_render_math", &json!({"text": r"\[\input{private}\]"})).is_err());
}

#[test]
fn screenshot_translation_requires_network_consent_and_a_valid_region() {
    let mut automation = Automation::new();
    let error = automation.call("region_translate", &json!({"doc":1,"page":1,"rect":[0,0,20,20],"allow_network":false})).unwrap_err();
    assert!(error.to_string().contains("allow_network=true"));
    assert!(automation.call("region_translate", &json!({"allow_network":true})).is_err());
    assert!(automation.call("region_translate", &json!({"allow_network":true,"rect":[0,0,20]})).is_err());
}
