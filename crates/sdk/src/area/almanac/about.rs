//! What a festival is, how it is kept, and who keeps it where
//! (`03-design/festival-about.md`, SR-153): a pack listing's `about`, one
//! entry per festival or group the SDK describes. The facts are
//! [`FestivalFacts`]; the words are the locales' `festival` records'
//! `about` forms and the `observance`, `community` and `region` records,
//! said through three messages of `sdk.voice.festival`.

use serde::{Deserialize, Serialize};
use teistro_intl::{Item, Params, Value, params};
use teistro_panchanga::festival::{FestivalFacts, FestivalMeta};

/// The message that says a festival's `about` form.
pub(crate) const ABOUT: &str = "sdk.voice.festival.about";
/// The message that says one act a festival is kept with.
pub(crate) const OBSERVANCE: &str = "sdk.voice.festival.observance";
/// The message that says who keeps a festival and where.
pub(crate) const WHO: &str = "sdk.voice.festival.who";

/// A festival described: what it is, the acts it is kept with, who keeps
/// it where, and the works the facts come from. Each item renders with
/// `intl.render(item.key, item.params)`.
///
/// ```
/// use teistro::{Context, FestivalPack, PackRequest};
///
/// let sdk = Context::builder().build().map_err(|_| "a context")?;
/// let asked = PackRequest { pack: FestivalPack::NepalPanchanga, about: true };
/// let listed = sdk.almanac().festival_pack(&asked);
/// let tihar = listed.about.iter().find(|a| a.key == "TIHAR").ok_or("described")?;
/// let said = sdk.intl().render(&tihar.about.key, &tihar.about.params);
/// assert!(said.text.starts_with("Tihar, the festival of lights"));
/// # Ok::<(), &str>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FestivalAbout {
    /// The festival's rule key or its group's key.
    pub key: String,
    /// Two to four sentences on what it is and why it is kept.
    pub about: Item,
    /// The acts it is kept with, one item each, in a page's order.
    pub observance: Vec<Item>,
    /// Who keeps it and where, from the rule's meta (a group's main
    /// day's).
    pub who: Item,
    /// The works its facts come from.
    pub sources: Vec<String>,
}

impl FestivalAbout {
    /// The description of `key`, whose meta (a group's main member's) is
    /// `meta`; `None` when the SDK does not describe it.
    #[must_use]
    pub(crate) fn of(key: &str, meta: &FestivalMeta) -> Option<FestivalAbout> {
        let facts = FestivalFacts::of(key)?;
        let entity = |kind: &str, member: &str| Value::Entity(format!("{kind}.{member}"));
        Some(FestivalAbout {
            key: key.to_owned(),
            about: Item::new(ABOUT, params([("festival", entity("festival", key))])),
            observance: facts
                .observance
                .iter()
                .map(|act| Item::new(OBSERVANCE, params([("act", entity("observance", act))])))
                .collect(),
            who: who(meta),
            sources: facts.sources.clone(),
        })
    }
}

/// Who keeps a festival and where: the message's `shape` names the parts
/// the meta holds (`t` a tradition, `c` one community or `cs` several, `r`
/// regions), so each language words every combination whole.
fn who(meta: &FestivalMeta) -> Item {
    let mut params = Params::new();
    let mut shape: Vec<&str> = Vec::new();
    if let Some(tradition) = meta.tradition {
        shape.push("t");
        params.insert(
            String::from("tradition"),
            Value::Entity(tradition.full_key().to_owned()),
        );
    }
    let list = |kind: &str, keys: &[String]| {
        Value::List(
            keys.iter()
                .map(|key| Value::Entity(format!("{kind}.{}", key.replace('-', "_"))))
                .collect(),
        )
    };
    match meta.communities.len() {
        0 => {}
        1 => shape.push("c"),
        _ => shape.push("cs"),
    }
    if !meta.communities.is_empty() {
        params.insert(
            String::from("communities"),
            list("community", &meta.communities),
        );
    }
    if !meta.regions.is_empty() {
        shape.push("r");
        params.insert(String::from("regions"), list("region", &meta.regions));
    }
    let shape = if shape.is_empty() {
        String::from("none")
    } else {
        shape.join("_")
    };
    params.insert(String::from("shape"), Value::Str(shape));
    Item::new(WHO, params)
}
