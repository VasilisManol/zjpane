use std::collections::BTreeMap;
use zellij_tile::prelude::*;

#[derive(Debug)]
enum Mode {
    Pane,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Pane
    }
}

#[derive(Default)]
struct State {
    active_tab: usize,
    mode: Mode,
    panes: Vec<PaneInfo>,
    position: usize,
    has_permission_granted: bool,
}

register_plugin!(State);

impl State {
    fn handle_event(&mut self, event: &Event) -> bool {
        let mut should_render = false;
        match event {
            Event::TabUpdate(tabs) => {
                for tab in tabs {
                    if tab.active {
                        self.active_tab = tab.position;
                        break;
                    }
                }
            }
            Event::PaneUpdate(infos) => {
                self.panes.clear();
                self.position = 0;
                for pane in infos.panes.iter() {
                    if *pane.0 != self.active_tab {
                        continue;
                    }
                    for info in pane.1 {
                        if !info.is_plugin {
                            self.panes.push(info.clone());
                        }
                    }
                }

                should_render = true;
            }
            Event::Key(key) if key.bare_key == BareKey::Esc => {
                self.position = 0;
                hide_self();
            }
            _ => (),
        }
        should_render
    }

    fn parse_pipe(&mut self, input: &str) -> bool {
        let should_render = false;

        let parts = input.split("::").collect::<Vec<&str>>();

        if parts.len() < 3 {
            return false;
        }

        if parts[0] != "zjpane" {
            return false;
        }

        let action = parts[1];
        let payload = parts[2];

        match action {
            "focus_at" => {
                if let Ok(Some(pane)) = payload.parse::<usize>().map(|index| self.panes.get(index))
                {
                    focus_terminal_pane(pane.id, false);
                }
            }
            "focus" => {
                let pane = self.panes.iter_mut().find(|pane| pane.title.eq(payload));
                if let Some(pane) = pane {
                    focus_terminal_pane(pane.id, false);
                }
            }
            _ => (),
        }

        should_render
    }
}

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::OpenTerminalsOrPlugins,
            PermissionType::RunCommands,
        ]);

        subscribe(&[
            EventType::TabUpdate,
            EventType::PaneUpdate,
            EventType::Key,
            EventType::PermissionRequestResult,
            EventType::RunCommandResult,
        ]);
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        let mut should_render = false;

        match pipe_message.source {
            PipeSource::Cli(_) | PipeSource::Plugin(_) | PipeSource::Keybind => {
                if let Some(payload) = pipe_message.payload {
                    should_render = self.parse_pipe(&payload);
                }
            }
        }

        should_render
    }

    fn update(&mut self, event: Event) -> bool {
        if let Event::PermissionRequestResult(status) = event {
            match status {
                PermissionStatus::Granted => self.has_permission_granted = true,
                PermissionStatus::Denied => self.has_permission_granted = false,
            }
        }

        if !self.has_permission_granted {
            return false;
        }

        self.handle_event(&event)
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        match self.mode {
            Mode::Pane => {
                for (i, pane) in self.panes.iter().enumerate() {
                    let selected = if i == self.position { "*" } else { " " };
                    println!("{} #{} {}", selected, pane.id, pane.title);
                }
            }
        }
    }
}
