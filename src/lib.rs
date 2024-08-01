// Copyright (c) 2024 fawn
// SPDX-License-Identifier: Apache-2.0

use copypasta_ext::{
    copypasta::ClipboardProvider, display::DisplayServer, osc52::Osc52ClipboardContext,
};
use faye::prelude::{Context as FayeContext, Parser};

rofi_mode::export_mode!(Mode<'_>);

const DEFAULT_MESSAGE: &str = "faye 0.6.1";

struct Mode<'rofi> {
    #[allow(dead_code)]
    api: rofi_mode::Api<'rofi>,
    faye: FayeContext,
    last_input: String,
    entries: Vec<Entry>,
}

impl<'rofi> rofi_mode::Mode<'rofi> for Mode<'rofi> {
    const NAME: &'static str = "faye\0";

    fn init(api: rofi_mode::Api<'rofi>) -> Result<Self, ()> {
        Ok(Self {
            api,
            faye: FayeContext::default(),
            last_input: DEFAULT_MESSAGE.to_owned(),
            entries: vec![Entry::new(
                String::from("Add to history"),
                String::from("Add to history"),
            )],
        })
    }

    fn entries(&mut self) -> usize {
        self.entries.len()
    }

    fn entry_content(&self, line: usize) -> rofi_mode::String {
        (&self.entries[line].output).into()
    }

    fn entry_icon(&mut self, _line: usize, _height: u32) -> Option<rofi_mode::cairo::Surface> {
        None
    }

    fn react(
        &mut self,
        event: rofi_mode::Event,
        input: &mut rofi_mode::String,
    ) -> rofi_mode::Action {
        match event {
            rofi_mode::Event::Cancel { selected: _ } => return rofi_mode::Action::Exit,
            rofi_mode::Event::Ok { selected, .. } => {
                if !self.is_history_button(selected) {
                    self.copy(selected);
                    println!(
                        "{} => {}",
                        self.entries[selected].input, self.entries[selected].output
                    );
                    return rofi_mode::Action::Exit;
                }

                if !self.is_init() {
                    if let Some(Ok(res)) = self.eval() {
                        self.entries.push(Entry::new(input.into(), res));
                    }
                }
            }
            rofi_mode::Event::DeleteEntry { selected } => {
                if !self.is_history_button(selected) {
                    self.entries.remove(selected);
                };
            }
            rofi_mode::Event::Complete {
                selected: Some(selected),
            } => {
                input.clear();
                input.push_str(&self.entries[selected].output);
            }
            rofi_mode::Event::Complete { .. }
            | rofi_mode::Event::CustomCommand { .. }
            | rofi_mode::Event::CustomInput { .. } => {}
        }
        rofi_mode::Action::Reload
    }

    fn matches(&self, _line: usize, _matcher: rofi_mode::Matcher<'_>) -> bool {
        true
    }

    fn message(&mut self) -> rofi_mode::String {
        match self.eval() {
            Some(Ok(res)) => rofi_mode::format!("Result: <b>{} => {res}</b>", self.last_input,),
            Some(Err(err)) => rofi_mode::format!("<span foreground='#ee9598'>Error: {err}</span>"),
            None => (&self.last_input).into(),
        }
    }

    fn preprocess_input(&mut self, input: &str) -> rofi_mode::String {
        self.last_input = input.to_string();
        unsafe { rofi_plugin_sys::view::reload() };
        input.into()
    }
}

impl Mode<'_> {
    fn copy(&mut self, selected: usize) {
        if let Some(mut ctx) = DisplayServer::select().try_context() {
            ctx.set_contents((&self.entries[selected].output).into())
                .unwrap();
        } else {
            let mut ctx = Osc52ClipboardContext::new().unwrap();
            ctx.set_contents((&self.entries[selected].output).into())
                .unwrap();
        }
    }

    fn eval(&mut self) -> Option<Result<String, String>> {
        if self.is_init() {
            return None;
        }

        // let mut faye_ctx = FayeContext::default();
        let mut parser = Parser::new(&self.last_input);

        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(err) => return Some(Err(err.to_string())),
        };

        let mut res = vec![];

        for node in ast {
            match self.faye.eval(&node) {
                Ok(expr) => res.push(expr),
                Err(err) => return Some(Err(err.to_string())),
            }
        }

        Some(Ok(res
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ")))
    }

    fn is_init(&self) -> bool {
        self.last_input.eq(DEFAULT_MESSAGE)
    }

    fn is_history_button(&self, selected: usize) -> bool {
        selected.eq(&0)
    }
}

struct Entry {
    input: String,
    output: String,
}

impl Entry {
    fn new(input: String, output: String) -> Self {
        Self { input, output }
    }
}
