//! A tool of the embedding program's own (`03-design/mcp-server.md` §7,
//! P9): listed and answered beside the SDK's records, under both
//! revisions and both transports, so a product built on the SDK serves
//! its own operations without forking the server.

use std::sync::Arc;

use serde_json::{Map, Value, json};
use teistro::{Context, Error};

use crate::watch;

/// What a tool's handler answers: its structured content, or the
/// refusal the model corrects, the SDK's own error record (D7).
///
/// ```
/// use serde_json::json;
/// use teistro_mcp::{Call, Handler};
///
/// // A handler answers its structured content, or a refusal.
/// let handler: Box<Handler> = Box::new(|call: &Call<'_>| Ok(json!({ "tool": call.name() })));
/// # let _ = handler;
/// ```
pub type Handler = dyn Fn(&Call<'_>) -> Result<Value, Error> + Send + Sync;

/// A tool of the embedding program's own.
///
/// ```
/// use serde_json::json;
/// use teistro_mcp::{Engine, Server, Tool};
///
/// let tool = Tool::new(
///     "acme.greet",
///     "Greets a querent by name.",
///     json!({ "type": "object", "properties": { "name": { "type": "string" } },
///             "required": ["name"] }),
///     |call| {
///         let name: String = call.argument("name")?.unwrap_or_default();
///         Ok(json!({ "greeting": format!("Namaste, {name}") }))
///     },
/// )
/// .read_only();
/// let server = Server::new(Engine::None).with_tool(tool).expect("a name of its own");
/// # let _ = server;
/// ```
#[derive(Clone)]
pub struct Tool {
    pub(crate) name: String,
    title: Option<String>,
    description: String,
    input: Value,
    output: Option<Value>,
    annotations: Option<Value>,
    pub(crate) context: bool,
    pub(crate) handler: Arc<Handler>,
}

impl core::fmt::Debug for Tool {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Tool")
            .field("name", &self.name)
            .field("title", &self.title)
            .field("context", &self.context)
            .finish_non_exhaustive()
    }
}

impl Tool {
    /// A tool called `name`, described for a model by `description`,
    /// whose arguments `input_schema` states (a JSON Schema object) and
    /// `handler` answers.
    ///
    /// ```
    /// # use serde_json::{Value, json};
    /// # use teistro_mcp::{Engine, Server};
    /// #
    /// # // One request under the stateless revision, its reply read back.
    /// # fn ask(server: &mut Server, method: &str, mut params: Value) -> Value {
    /// #     params["_meta"] = json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
    /// #                               "io.modelcontextprotocol/clientCapabilities": {} });
    /// #     let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    /// #     server.handle(&message.to_string()).and_then(|reply| serde_json::from_str(&reply).ok()).unwrap_or(Value::Null)
    /// # }
    /// # // The tool called `name` as `tools/list` states it.
    /// # fn listed(server: &mut Server, name: &str) -> Value {
    /// #     let tools = ask(server, "tools/list", json!({}))["result"]["tools"].clone();
    /// #     tools.as_array().and_then(|tools| tools.iter().find(|tool| tool["name"] == name).cloned()).unwrap_or(Value::Null)
    /// # }
    /// # use teistro_mcp::Tool;
    /// #
    /// # let greet = Tool::new(
    /// #     "acme.greet",
    /// #     "Greets a querent by name.",
    /// #     json!({ "type": "object", "properties": { "name": { "type": "string" } }, "required": ["name"] }),
    /// #     |call| {
    /// #         let name: String = call.argument("name")?.unwrap_or_default();
    /// #         Ok(json!({ "greeting": format!("Namaste, {name}") }))
    /// #     },
    /// # );
    ///
    /// let mut server = Server::new(Engine::None).with_tool(greet)?;
    /// let tool = listed(&mut server, "acme.greet");
    /// assert_eq!(tool["description"], "Greets a querent by name.");
    /// assert_eq!(tool["inputSchema"]["required"], json!(["name"]));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
        handler: impl Fn(&Call<'_>) -> Result<Value, Error> + Send + Sync + 'static,
    ) -> Tool {
        Tool {
            name: name.into(),
            title: None,
            description: description.into(),
            input: input_schema,
            output: None,
            annotations: None,
            context: false,
            handler: Arc::new(handler),
        }
    }

    /// The same tool, with a title for a person choosing it.
    ///
    /// ```
    /// # use serde_json::{Value, json};
    /// # use teistro_mcp::{Engine, Server};
    /// #
    /// # // One request under the stateless revision, its reply read back.
    /// # fn ask(server: &mut Server, method: &str, mut params: Value) -> Value {
    /// #     params["_meta"] = json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
    /// #                               "io.modelcontextprotocol/clientCapabilities": {} });
    /// #     let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    /// #     server.handle(&message.to_string()).and_then(|reply| serde_json::from_str(&reply).ok()).unwrap_or(Value::Null)
    /// # }
    /// # // The tool called `name` as `tools/list` states it.
    /// # fn listed(server: &mut Server, name: &str) -> Value {
    /// #     let tools = ask(server, "tools/list", json!({}))["result"]["tools"].clone();
    /// #     tools.as_array().and_then(|tools| tools.iter().find(|tool| tool["name"] == name).cloned()).unwrap_or(Value::Null)
    /// # }
    /// # use teistro_mcp::Tool;
    /// #
    /// # let greet = Tool::new(
    /// #     "acme.greet",
    /// #     "Greets a querent by name.",
    /// #     json!({ "type": "object", "properties": { "name": { "type": "string" } }, "required": ["name"] }),
    /// #     |call| {
    /// #         let name: String = call.argument("name")?.unwrap_or_default();
    /// #         Ok(json!({ "greeting": format!("Namaste, {name}") }))
    /// #     },
    /// # );
    ///
    /// let mut server = Server::new(Engine::None).with_tool(greet.titled("Greet"))?;
    /// assert_eq!(listed(&mut server, "acme.greet")["title"], "Greet");
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn titled(self, title: impl Into<String>) -> Tool {
        Tool {
            title: Some(title.into()),
            ..self
        }
    }

    /// The same tool, its structured content held to `output_schema`,
    /// listed under [`crate::Detail::Full`] as the SDK's answers are.
    ///
    /// ```
    /// # use serde_json::{Value, json};
    /// # use teistro_mcp::{Engine, Server};
    /// #
    /// # // One request under the stateless revision, its reply read back.
    /// # fn ask(server: &mut Server, method: &str, mut params: Value) -> Value {
    /// #     params["_meta"] = json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
    /// #                               "io.modelcontextprotocol/clientCapabilities": {} });
    /// #     let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    /// #     server.handle(&message.to_string()).and_then(|reply| serde_json::from_str(&reply).ok()).unwrap_or(Value::Null)
    /// # }
    /// # // The tool called `name` as `tools/list` states it.
    /// # fn listed(server: &mut Server, name: &str) -> Value {
    /// #     let tools = ask(server, "tools/list", json!({}))["result"]["tools"].clone();
    /// #     tools.as_array().and_then(|tools| tools.iter().find(|tool| tool["name"] == name).cloned()).unwrap_or(Value::Null)
    /// # }
    /// # use teistro_mcp::Tool;
    /// #
    /// # let greet = Tool::new(
    /// #     "acme.greet",
    /// #     "Greets a querent by name.",
    /// #     json!({ "type": "object", "properties": { "name": { "type": "string" } }, "required": ["name"] }),
    /// #     |call| {
    /// #         let name: String = call.argument("name")?.unwrap_or_default();
    /// #         Ok(json!({ "greeting": format!("Namaste, {name}") }))
    /// #     },
    /// # );
    ///
    /// use teistro_mcp::Detail;
    ///
    /// let answer = json!({ "type": "object", "properties": { "greeting": { "type": "string" } } });
    /// let mut server = Server::new(Engine::None).with_detail(Detail::Full).with_tool(greet.answering(answer.clone()))?;
    /// assert_eq!(listed(&mut server, "acme.greet")["outputSchema"], answer);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn answering(self, output_schema: Value) -> Tool {
        Tool {
            output: Some(output_schema),
            ..self
        }
    }

    /// The same tool, annotated read-only, idempotent and closed-world, as
    /// every SDK tool is. Without it a host assumes the protocol's
    /// defaults: that the tool may write and is not idempotent.
    ///
    /// ```
    /// # use serde_json::{Value, json};
    /// # use teistro_mcp::{Engine, Server};
    /// #
    /// # // One request under the stateless revision, its reply read back.
    /// # fn ask(server: &mut Server, method: &str, mut params: Value) -> Value {
    /// #     params["_meta"] = json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
    /// #                               "io.modelcontextprotocol/clientCapabilities": {} });
    /// #     let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    /// #     server.handle(&message.to_string()).and_then(|reply| serde_json::from_str(&reply).ok()).unwrap_or(Value::Null)
    /// # }
    /// # // The tool called `name` as `tools/list` states it.
    /// # fn listed(server: &mut Server, name: &str) -> Value {
    /// #     let tools = ask(server, "tools/list", json!({}))["result"]["tools"].clone();
    /// #     tools.as_array().and_then(|tools| tools.iter().find(|tool| tool["name"] == name).cloned()).unwrap_or(Value::Null)
    /// # }
    /// # use teistro_mcp::Tool;
    /// #
    /// # let greet = Tool::new(
    /// #     "acme.greet",
    /// #     "Greets a querent by name.",
    /// #     json!({ "type": "object", "properties": { "name": { "type": "string" } }, "required": ["name"] }),
    /// #     |call| {
    /// #         let name: String = call.argument("name")?.unwrap_or_default();
    /// #         Ok(json!({ "greeting": format!("Namaste, {name}") }))
    /// #     },
    /// # );
    ///
    /// let mut server = Server::new(Engine::None).with_tool(greet.read_only())?;
    /// let annotations = &listed(&mut server, "acme.greet")["annotations"];
    /// assert_eq!((&annotations["readOnlyHint"], &annotations["idempotentHint"]), (&json!(true), &json!(true)));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn read_only(self) -> Tool {
        let title = self.title.clone().unwrap_or_else(|| self.name.clone());
        Tool {
            annotations: Some(crate::tools::annotations(&title)),
            ..self
        }
    }

    /// The same tool, with annotations of the program's own choosing.
    ///
    /// ```
    /// # use serde_json::{Value, json};
    /// # use teistro_mcp::{Engine, Server};
    /// #
    /// # // One request under the stateless revision, its reply read back.
    /// # fn ask(server: &mut Server, method: &str, mut params: Value) -> Value {
    /// #     params["_meta"] = json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
    /// #                               "io.modelcontextprotocol/clientCapabilities": {} });
    /// #     let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    /// #     server.handle(&message.to_string()).and_then(|reply| serde_json::from_str(&reply).ok()).unwrap_or(Value::Null)
    /// # }
    /// # // The tool called `name` as `tools/list` states it.
    /// # fn listed(server: &mut Server, name: &str) -> Value {
    /// #     let tools = ask(server, "tools/list", json!({}))["result"]["tools"].clone();
    /// #     tools.as_array().and_then(|tools| tools.iter().find(|tool| tool["name"] == name).cloned()).unwrap_or(Value::Null)
    /// # }
    /// # use teistro_mcp::Tool;
    /// #
    /// # let greet = Tool::new(
    /// #     "acme.greet",
    /// #     "Greets a querent by name.",
    /// #     json!({ "type": "object", "properties": { "name": { "type": "string" } }, "required": ["name"] }),
    /// #     |call| {
    /// #         let name: String = call.argument("name")?.unwrap_or_default();
    /// #         Ok(json!({ "greeting": format!("Namaste, {name}") }))
    /// #     },
    /// # );
    ///
    /// let mut server = Server::new(Engine::None).with_tool(greet.annotated(json!({ "openWorldHint": true })))?;
    /// assert_eq!(listed(&mut server, "acme.greet")["annotations"], json!({ "openWorldHint": true }));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn annotated(self, annotations: Value) -> Tool {
        Tool {
            annotations: Some(annotations),
            ..self
        }
    }

    /// The same tool, reading `profile`, `settings` and `locale` beside
    /// its own arguments as a record tool does (D5); its handler reaches
    /// the context they name through [`Call::context`], with the server's
    /// engine, watch and packs.
    ///
    /// ```
    /// # use serde_json::{Value, json};
    /// # use teistro_mcp::{Engine, Server};
    /// #
    /// # // One request under the stateless revision, its reply read back.
    /// # fn ask(server: &mut Server, method: &str, mut params: Value) -> Value {
    /// #     params["_meta"] = json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
    /// #                               "io.modelcontextprotocol/clientCapabilities": {} });
    /// #     let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    /// #     server.handle(&message.to_string()).and_then(|reply| serde_json::from_str(&reply).ok()).unwrap_or(Value::Null)
    /// # }
    /// # // The tool called `name` as `tools/list` states it.
    /// # fn listed(server: &mut Server, name: &str) -> Value {
    /// #     let tools = ask(server, "tools/list", json!({}))["result"]["tools"].clone();
    /// #     tools.as_array().and_then(|tools| tools.iter().find(|tool| tool["name"] == name).cloned()).unwrap_or(Value::Null)
    /// # }
    ///
    /// use teistro_mcp::Tool;
    ///
    /// // It reads the context the call's profile names.
    /// let tool = Tool::new("acme.hash", "The settings hash a call computes under.", json!({ "type": "object" }), |call| {
    ///     Ok(json!({ "hash": call.context()?.settings_hash().to_string() }))
    /// })
    /// .in_context();
    /// let mut server = Server::new(Engine::None).with_tool(tool)?;
    /// assert!(listed(&mut server, "acme.hash")["inputSchema"]["properties"].get("profile").is_some());
    /// let reply = ask(&mut server, "tools/call", json!({ "name": "acme.hash", "arguments": { "profile": "nepali-default" } }));
    /// assert!(reply["result"]["structuredContent"]["hash"].is_string(), "{reply}");
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn in_context(self) -> Tool {
        Tool {
            context: true,
            ..self
        }
    }

    /// The tool's name.
    ///
    /// ```
    /// # use serde_json::{Value, json};
    /// # use teistro_mcp::{Engine, Server};
    /// #
    /// # // One request under the stateless revision, its reply read back.
    /// # fn ask(server: &mut Server, method: &str, mut params: Value) -> Value {
    /// #     params["_meta"] = json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
    /// #                               "io.modelcontextprotocol/clientCapabilities": {} });
    /// #     let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    /// #     server.handle(&message.to_string()).and_then(|reply| serde_json::from_str(&reply).ok()).unwrap_or(Value::Null)
    /// # }
    /// # // The tool called `name` as `tools/list` states it.
    /// # fn listed(server: &mut Server, name: &str) -> Value {
    /// #     let tools = ask(server, "tools/list", json!({}))["result"]["tools"].clone();
    /// #     tools.as_array().and_then(|tools| tools.iter().find(|tool| tool["name"] == name).cloned()).unwrap_or(Value::Null)
    /// # }
    /// # use teistro_mcp::Tool;
    /// #
    /// # let greet = Tool::new(
    /// #     "acme.greet",
    /// #     "Greets a querent by name.",
    /// #     json!({ "type": "object", "properties": { "name": { "type": "string" } }, "required": ["name"] }),
    /// #     |call| {
    /// #         let name: String = call.argument("name")?.unwrap_or_default();
    /// #         Ok(json!({ "greeting": format!("Namaste, {name}") }))
    /// #     },
    /// # );
    ///
    /// assert_eq!(greet.name(), "acme.greet");
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether the tool may be registered beside `sdk`, the names the
    /// SDK lists, and `own`, the program's tools before it: a name the
    /// revision allows, held by neither, outside every namespace the SDK
    /// lists under, and object schemas that leave the context arguments
    /// to the server.
    pub(crate) fn check(&self, sdk: &[String], own: &[Tool]) -> Result<(), Error> {
        let name = self.name.as_str();
        let refuse = |message: String, hint: String| {
            Err(Error::invalid_arg(message)
                .with_field("tool.name")
                .with_hint(hint))
        };
        if !crate::tools::named_as_the_protocol_allows(name) {
            return refuse(
                format!("`{name}` is not a tool name the protocol allows"),
                String::from("letters, digits, `_`, `-` and `.`, one to 128 of them"),
            );
        }
        if sdk.iter().any(|held| held == name) || own.iter().any(|tool| tool.name == name) {
            return refuse(
                format!("a tool `{name}` is already listed"),
                String::from("give each tool a name of its own"),
            );
        }
        let segment = name.split('.').next().unwrap_or(name);
        if let Some(held) = sdk
            .iter()
            .filter(|held| held.contains('.'))
            .find(|held| held.split('.').next() == Some(segment))
        {
            return refuse(
                format!("`{segment}.` is a namespace the SDK lists tools under (`{held}`)"),
                format!(
                    "name it under the program's own prefix, as `acme.{}`",
                    name.rsplit('.').next().unwrap_or(name)
                ),
            );
        }
        for (field, schema) in [
            ("tool.inputSchema", Some(&self.input)),
            ("tool.outputSchema", self.output.as_ref()),
        ] {
            if let Some(schema) = schema {
                if schema.get("type") != Some(&json!("object")) {
                    return Err(Error::invalid_arg("a tool's schema is an object schema")
                        .with_field(field)
                        .with_hint("`{\"type\": \"object\", ...}`, as the protocol requires"));
                }
            }
        }
        if self.context {
            let properties = self.input.get("properties").and_then(Value::as_object);
            if let Some(scope) = crate::tools::SCOPE
                .iter()
                .find(|key| properties.is_some_and(|properties| properties.contains_key(**key)))
            {
                return Err(Error::invalid_arg(format!(
                    "`{scope}` is the server's to read for a tool in context"
                ))
                .with_field(format!("tool.inputSchema.properties.{scope}")));
            }
        }
        Ok(())
    }

    /// The tool as `tools/list` states it.
    pub(crate) fn listed(&self, detail: crate::Detail) -> Value {
        let mut input = self.input.clone();
        if self.context {
            if let Some(Value::Object(properties)) = input
                .as_object_mut()
                .map(|input| input.entry("properties").or_insert_with(|| json!({})))
            {
                properties.extend(crate::schemas::scope_properties());
            }
        }
        let mut tool = Map::new();
        tool.insert(String::from("name"), json!(self.name));
        if let Some(title) = &self.title {
            tool.insert(String::from("title"), json!(title));
        }
        tool.insert(String::from("description"), json!(self.description));
        tool.insert(String::from("inputSchema"), input);
        if let (crate::Detail::Full, Some(output)) = (detail, &self.output) {
            tool.insert(String::from("outputSchema"), output.clone());
        }
        if let Some(annotations) = &self.annotations {
            tool.insert(String::from("annotations"), annotations.clone());
        }
        Value::Object(tool)
    }
}

/// One call of a tool, as its handler receives it.
///
/// ```
/// # use serde_json::{Value, json};
/// # use teistro_mcp::{Engine, Server};
/// #
/// # // One request under the stateless revision, its reply read back.
/// # fn ask(server: &mut Server, method: &str, mut params: Value) -> Value {
/// #     params["_meta"] = json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
/// #                               "io.modelcontextprotocol/clientCapabilities": {} });
/// #     let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
/// #     server.handle(&message.to_string()).and_then(|reply| serde_json::from_str(&reply).ok()).unwrap_or(Value::Null)
/// # }
/// # // The tool called `name` as `tools/list` states it.
/// # fn listed(server: &mut Server, name: &str) -> Value {
/// #     let tools = ask(server, "tools/list", json!({}))["result"]["tools"].clone();
/// #     tools.as_array().and_then(|tools| tools.iter().find(|tool| tool["name"] == name).cloned()).unwrap_or(Value::Null)
/// # }
///
/// use teistro_mcp::Tool;
///
/// // What a handler is given: its name and arguments.
/// let echo = Tool::new("acme.echo", "Echoes its arguments.", json!({ "type": "object" }), |call| {
///     Ok(json!({ "tool": call.name(), "arguments": call.arguments() }))
/// });
/// let mut server = Server::new(Engine::None).with_tool(echo)?;
/// let reply = ask(&mut server, "tools/call", json!({ "name": "acme.echo", "arguments": { "n": 3 } }));
/// assert_eq!(reply["result"]["structuredContent"], json!({ "tool": "acme.echo", "arguments": { "n": 3 } }));
/// # Ok::<(), teistro::Error>(())
/// ```
pub struct Call<'a> {
    pub(crate) name: &'a str,
    pub(crate) arguments: &'a Map<String, Value>,
    pub(crate) context: Option<&'a Context>,
    pub(crate) watch: &'a watch::Shared,
}

impl core::fmt::Debug for Call<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Call")
            .field("name", &self.name)
            .field("arguments", &self.arguments)
            .field("in_context", &self.context.is_some())
            .finish_non_exhaustive()
    }
}

impl<'a> Call<'a> {
    /// The tool called.
    ///
    /// ```
    /// # use serde_json::{Value, json};
    /// # use teistro_mcp::{Engine, Server};
    /// #
    /// # // One request under the stateless revision, its reply read back.
    /// # fn ask(server: &mut Server, method: &str, mut params: Value) -> Value {
    /// #     params["_meta"] = json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
    /// #                               "io.modelcontextprotocol/clientCapabilities": {} });
    /// #     let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    /// #     server.handle(&message.to_string()).and_then(|reply| serde_json::from_str(&reply).ok()).unwrap_or(Value::Null)
    /// # }
    /// # // The tool called `name` as `tools/list` states it.
    /// # fn listed(server: &mut Server, name: &str) -> Value {
    /// #     let tools = ask(server, "tools/list", json!({}))["result"]["tools"].clone();
    /// #     tools.as_array().and_then(|tools| tools.iter().find(|tool| tool["name"] == name).cloned()).unwrap_or(Value::Null)
    /// # }
    ///
    /// use teistro_mcp::Tool;
    ///
    /// // One handler can answer two tools.
    /// let named = |name: &str| Tool::new(name, "Says its name.", json!({ "type": "object" }), |call| Ok(json!(call.name())));
    /// let mut server = Server::new(Engine::None).with_tool(named("acme.one"))?.with_tool(named("acme.two"))?;
    /// let reply = ask(&mut server, "tools/call", json!({ "name": "acme.two", "arguments": {} }));
    /// assert_eq!(reply["result"]["structuredContent"], "acme.two");
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        self.name
    }

    /// The call's arguments, without `profile`, `settings` and `locale`
    /// for a tool in context.
    ///
    /// ```
    /// # use serde_json::{Value, json};
    /// # use teistro_mcp::{Engine, Server};
    /// #
    /// # // One request under the stateless revision, its reply read back.
    /// # fn ask(server: &mut Server, method: &str, mut params: Value) -> Value {
    /// #     params["_meta"] = json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
    /// #                               "io.modelcontextprotocol/clientCapabilities": {} });
    /// #     let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    /// #     server.handle(&message.to_string()).and_then(|reply| serde_json::from_str(&reply).ok()).unwrap_or(Value::Null)
    /// # }
    /// # // The tool called `name` as `tools/list` states it.
    /// # fn listed(server: &mut Server, name: &str) -> Value {
    /// #     let tools = ask(server, "tools/list", json!({}))["result"]["tools"].clone();
    /// #     tools.as_array().and_then(|tools| tools.iter().find(|tool| tool["name"] == name).cloned()).unwrap_or(Value::Null)
    /// # }
    ///
    /// use teistro_mcp::Tool;
    ///
    /// let count = Tool::new("acme.count", "Counts its arguments.", json!({ "type": "object" }), |call| Ok(json!(call.arguments().len())));
    /// let mut server = Server::new(Engine::None).with_tool(count)?;
    /// let reply = ask(&mut server, "tools/call", json!({ "name": "acme.count", "arguments": { "a": 1, "b": 2 } }));
    /// assert_eq!(reply["result"]["structuredContent"], 2);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn arguments(&self) -> &Map<String, Value> {
        self.arguments
    }

    /// The argument `key` read as `T`, or `None` where it is absent.
    ///
    /// ```
    /// # use serde_json::{Value, json};
    /// # use teistro_mcp::{Engine, Server};
    /// #
    /// # // One request under the stateless revision, its reply read back.
    /// # fn ask(server: &mut Server, method: &str, mut params: Value) -> Value {
    /// #     params["_meta"] = json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
    /// #                               "io.modelcontextprotocol/clientCapabilities": {} });
    /// #     let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    /// #     server.handle(&message.to_string()).and_then(|reply| serde_json::from_str(&reply).ok()).unwrap_or(Value::Null)
    /// # }
    /// # // The tool called `name` as `tools/list` states it.
    /// # fn listed(server: &mut Server, name: &str) -> Value {
    /// #     let tools = ask(server, "tools/list", json!({}))["result"]["tools"].clone();
    /// #     tools.as_array().and_then(|tools| tools.iter().find(|tool| tool["name"] == name).cloned()).unwrap_or(Value::Null)
    /// # }
    /// # use teistro_mcp::Tool;
    /// #
    /// # let greet = Tool::new(
    /// #     "acme.greet",
    /// #     "Greets a querent by name.",
    /// #     json!({ "type": "object", "properties": { "name": { "type": "string" } }, "required": ["name"] }),
    /// #     |call| {
    /// #         let name: String = call.argument("name")?.unwrap_or_default();
    /// #         Ok(json!({ "greeting": format!("Namaste, {name}") }))
    /// #     },
    /// # );
    ///
    /// // A value that does not read is the tool's refusal, naming the argument.
    /// let mut server = Server::new(Engine::None).with_tool(greet)?;
    /// let reply = ask(&mut server, "tools/call", json!({ "name": "acme.greet", "arguments": { "name": 7 } }));
    /// assert_eq!(reply["result"]["isError"], true);
    /// assert!(reply.to_string().contains("arguments.name"), "{reply}");
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `arguments.{key}` for a value `T` does not
    /// read.
    pub fn argument<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error> {
        self.arguments
            .get(key)
            .map(|value| {
                serde_json::from_value(value.clone()).map_err(|why| {
                    Error::invalid_arg(format!("`{key}` is not one: {why}"))
                        .with_field(format!("arguments.{key}"))
                })
            })
            .transpose()
    }

    /// The context the call names, from the server's cache: its engine,
    /// its watch and its packs.
    ///
    /// ```
    /// # use serde_json::{Value, json};
    /// # use teistro_mcp::{Engine, Server};
    /// #
    /// # // One request under the stateless revision, its reply read back.
    /// # fn ask(server: &mut Server, method: &str, mut params: Value) -> Value {
    /// #     params["_meta"] = json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
    /// #                               "io.modelcontextprotocol/clientCapabilities": {} });
    /// #     let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    /// #     server.handle(&message.to_string()).and_then(|reply| serde_json::from_str(&reply).ok()).unwrap_or(Value::Null)
    /// # }
    /// # // The tool called `name` as `tools/list` states it.
    /// # fn listed(server: &mut Server, name: &str) -> Value {
    /// #     let tools = ask(server, "tools/list", json!({}))["result"]["tools"].clone();
    /// #     tools.as_array().and_then(|tools| tools.iter().find(|tool| tool["name"] == name).cloned()).unwrap_or(Value::Null)
    /// # }
    ///
    /// use teistro_mcp::Tool;
    ///
    /// // A tool not made `in_context` has none to read.
    /// let bare = Tool::new("acme.bare", "Reads a context it was not given.", json!({ "type": "object" }), |call| {
    ///     call.context().map(|_| json!(true))
    /// });
    /// let mut server = Server::new(Engine::None).with_tool(bare)?;
    /// let reply = ask(&mut server, "tools/call", json!({ "name": "acme.bare", "arguments": {} }));
    /// assert_eq!(reply["result"]["isError"], true, "{reply}");
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INTERNAL` for a tool not made [`Tool::in_context`].
    pub fn context(&self) -> Result<&'a Context, Error> {
        self.context.ok_or_else(|| {
            Error::internal(format!(
                "the tool `{}` reads no context; make it `in_context`",
                self.name
            ))
        })
    }

    /// Whether the call was cancelled. A handler computing through
    /// [`Call::context`] stops at its next engine request by itself; one
    /// looping without the engine asks this.
    ///
    /// ```
    /// # use serde_json::{Value, json};
    /// # use teistro_mcp::{Engine, Server};
    /// #
    /// # // One request under the stateless revision, its reply read back.
    /// # fn ask(server: &mut Server, method: &str, mut params: Value) -> Value {
    /// #     params["_meta"] = json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
    /// #                               "io.modelcontextprotocol/clientCapabilities": {} });
    /// #     let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    /// #     server.handle(&message.to_string()).and_then(|reply| serde_json::from_str(&reply).ok()).unwrap_or(Value::Null)
    /// # }
    /// # // The tool called `name` as `tools/list` states it.
    /// # fn listed(server: &mut Server, name: &str) -> Value {
    /// #     let tools = ask(server, "tools/list", json!({}))["result"]["tools"].clone();
    /// #     tools.as_array().and_then(|tools| tools.iter().find(|tool| tool["name"] == name).cloned()).unwrap_or(Value::Null)
    /// # }
    ///
    /// use teistro_mcp::Tool;
    ///
    /// // A call running uncancelled says so.
    /// let looking = Tool::new("acme.look", "Looks whether it was cancelled.", json!({ "type": "object" }), |call| Ok(json!(call.cancelled())));
    /// let mut server = Server::new(Engine::None).with_tool(looking)?;
    /// let reply = ask(&mut server, "tools/call", json!({ "name": "acme.look", "arguments": {} }));
    /// assert_eq!(reply["result"]["structuredContent"], false);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn cancelled(&self) -> bool {
        self.watch.stopping()
    }
}
