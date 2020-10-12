use crate::oauth::{OAuthProvider, RocketWebServer};
use gdnative::prelude::*;
use port_check;
use std::sync::mpsc::Receiver;

#[derive(NativeClass)]
#[inherit(Node)]
#[register_with(Self::register_signals)]
pub struct OAuthValidation {
    token_receiver: Option<Receiver<String>>,
}

#[methods]
impl OAuthValidation {
    fn new(_owner: &Node) -> Self {
        OAuthValidation {
            token_receiver: None,
        }
    }

    fn register_signals(builder: &ClassBuilder<Self>) {
        builder.add_signal(Signal {
            name: "login_url_ready",
            args: &[SignalArgument {
                name: "url",
                default: Variant::from_str(""),
                export_info: ExportInfo::new(VariantType::GodotString),
                usage: PropertyUsage::DEFAULT,
            }],
        });

        builder.add_signal(Signal {
            name: "token_received",
            args: &[SignalArgument {
                name: "token",
                default: Variant::from_str(""),
                export_info: ExportInfo::new(VariantType::GodotString),
                usage: PropertyUsage::DEFAULT,
            }],
        });
    }

    #[export]
    fn _ready(&mut self, owner: TRef<Node>) {
        let provider = OAuthProvider::new();
        let port = port_check::free_local_port();
        match RocketWebServer::builder().port(port).build() {
            Ok(rocket) => {
                let (token_receiver, url) = provider.provide(rocket);
                self.token_receiver = Some(token_receiver);
                owner.emit_signal("login_url_ready", &[Variant::from_str(url)]);
            }
            Err(err) => godot_error!("Error {:?} building rocket", err),
        };
    }

    #[export]
    fn _process(&self, owner: TRef<Node>, _delta: f64) {
        if let Some(token_receiver) = &self.token_receiver {
            if let Ok(token) = token_receiver.try_recv() {
                owner.emit_signal("token_received", &[Variant::from_str(token)]);
            }
        }
    }
}
