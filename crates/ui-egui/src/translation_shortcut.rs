//! A persisted, conflict-checked shortcut for page-region translation.
use egui::{Key, KeyboardShortcut, Modifiers};

pub(crate) struct ShortcutSetting {
    pub value: String,
    pub capturing: bool,
    error: String,
}
impl Default for ShortcutSetting {
    fn default() -> Self {
        Self { value: "Command+Shift+X".into(), capturing: false, error: String::new() }
    }
}
fn parse(value: &str) -> Result<KeyboardShortcut, String> {
    let mut modifiers = Modifiers::NONE;
    let mut key = None;
    for part in value.split('+') {
        match part {
            "Command" | "Cmd" => modifiers |= Modifiers::COMMAND,
            "Ctrl" => modifiers |= Modifiers::CTRL,
            "Alt" => modifiers |= Modifiers::ALT,
            "Shift" => modifiers |= Modifiers::SHIFT,
            _ if key.is_none() => key = Key::from_name(part),
            _ => return Err("快捷键格式无效。".into()),
        }
    }
    let key = key.ok_or("请按下一个字母、数字或功能键。")?;
    if !(modifiers.command || modifiers.ctrl || modifiers.alt) || matches!(key, Key::Escape | Key::Tab | Key::Enter) {
        return Err("请使用包含 ⌘、Ctrl 或 Alt 的组合键。".into());
    }
    // Registry shortcuts and view-local editing/navigation keys must keep their meaning.
    for command in pdfcraft_engine::commands::COMMANDS {
        if let Some(s) = command.shortcut
            && Key::from_name(s.key) == Some(key)
            && s.command == modifiers.command
            && s.shift == modifiers.shift
            && s.mac_ctrl == (modifiers.ctrl && !modifiers.command)
            && !modifiers.alt
        {
            return Err(format!("该快捷键已用于 {}，请换一个组合。", command.label));
        }
    }
    if modifiers.command && !modifiers.alt && !modifiers.shift && matches!(key, Key::A | Key::C | Key::V | Key::X | Key::Z | Key::Q | Key::H | Key::M)
    {
        return Err("该快捷键已被编辑或系统操作占用。".into());
    }
    if modifiers.command && modifiers.shift && !modifiers.alt && matches!(key, Key::Num3 | Key::Num4 | Key::Num5) {
        return Err("该快捷键已被系统截图占用。".into());
    }
    Ok(KeyboardShortcut::new(modifiers, key))
}
impl ShortcutSetting {
    pub fn set(&mut self, value: &str) -> Result<(), String> {
        if !value.is_empty() {
            parse(value)?;
        }
        self.value = value.to_string();
        Ok(())
    }
    pub fn label(&self) -> String {
        if self.value.is_empty() { "未设置".into() } else { self.value.replace("Command", "⌘").replace("Shift", "⇧").replace("Alt", "⌥") }
    }
    pub fn consume(&self, ctx: &egui::Context) -> bool {
        parse(&self.value).is_ok_and(|s| ctx.input_mut(|i| i.consume_shortcut(&s)))
    }
    pub fn preferences(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.horizontal(|ui| {
            ui.strong("截图翻译快捷键");
            ui.label(self.label());
            if ui.button(if self.capturing { "正在录入…" } else { "修改快捷键" }).clicked() {
                self.capturing = true;
                self.error.clear();
            }
            if ui.button("清除快捷键").clicked() {
                self.value.clear();
                self.capturing = false;
            }
        });
        ui.weak("阅读器内生效；按快捷键后，在页面上拖动框选。点击译文框外自动关闭。");
        if self.capturing {
            ui.label("请按新的组合键（包含 ⌘ / Ctrl / Alt），Esc 取消录入。");
            let event = ui.input(|i| {
                i.events.iter().find_map(|event| match event {
                    egui::Event::Key { key, modifiers, pressed: true, repeat: false, .. } => Some((*key, *modifiers)),
                    _ => None,
                })
            });
            if let Some((key, modifiers)) = event {
                if key == Key::Escape {
                    self.capturing = false;
                } else {
                    let mut parts = Vec::new();
                    if modifiers.command {
                        parts.push("Command");
                    }
                    if modifiers.ctrl && (!modifiers.command || cfg!(target_os = "macos")) {
                        parts.push("Ctrl");
                    }
                    if modifiers.alt {
                        parts.push("Alt");
                    }
                    if modifiers.shift {
                        parts.push("Shift");
                    }
                    parts.push(key.name());
                    match self.set(&parts.join("+")) {
                        Ok(()) => {
                            self.capturing = false;
                            self.error.clear();
                        }
                        Err(error) => self.error = error,
                    }
                }
            }
        }
        if !self.error.is_empty() {
            ui.colored_label(ui.visuals().error_fg_color, &self.error);
        }
        ui.separator();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn screenshot_shortcut_rejects_conflicts_and_preserves_previous_setting() {
        let mut setting = ShortcutSetting::default();
        assert!(parse(&setting.value).is_ok());
        for bad in ["X", "Command+C", "Command+Shift+T", "Command+Shift+4", "invalid"] {
            assert!(setting.set(bad).is_err(), "{bad}");
            assert_eq!(setting.value, "Command+Shift+X");
        }
        assert!(setting.set("Alt+Shift+T").is_ok());
        assert!(setting.set("").is_ok());
    }
}
