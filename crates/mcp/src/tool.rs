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
    #[must_use]
    pub fn titled(self, title: impl Into<String>) -> Tool {
        Tool {
            title: Some(title.into()),
            ..self
        }
    }

    /// The same tool, its structured content held to `output_schema`,
    /// listed under [`crate::Detail::Full`] as the SDK's answers are.
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
    #[must_use]
    pub fn read_only(self) -> Tool {
        let title = self.title.clone().unwrap_or_else(|| self.name.clone());
        Tool {
            annotations: Some(crate::tools::annotations(&title)),
            ..self
        }
    }

    /// The same tool, with annotations of the program's own choosing.
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
    #[must_use]
    pub fn in_context(self) -> Tool {
        Tool {
            context: true,
            ..self
        }
    }

    /// The tool's name.
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
    #[must_use]
    pub fn name(&self) -> &str {
        self.name
    }

    /// The call's arguments, without `profile`, `settings` and `locale`
    /// for a tool in context.
    #[must_use]
    pub fn arguments(&self) -> &Map<String, Value> {
        self.arguments
    }

    /// The argument `key` read as `T`, or `None` where it is absent.
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
    #[must_use]
    pub fn cancelled(&self) -> bool {
        self.watch.stopping()
    }
}
