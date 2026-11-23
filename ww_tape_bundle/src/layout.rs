//! How a recorder lays a stream out on a tape, from the stream's `#[track(..)]` in the bundle (API-71):
//! [stream_layout].

use ww_self::keys::keys_level;
use ww_self::{
    ApiBundleOwned, ApiItemKindOwned, ApiItemOwned, ApiLevelOwned, Multiplicity, SeriesShape,
    TrackLayout, TrackOwned, TypeOwned,
};
use ww_tape::{Ends, EventsDef};

use crate::NamePathError;

/// Why a stream has no layout on a tape.
#[derive(Debug)]
pub enum LayoutError {
    /// The path doesn't name a resource of the bundle.
    Path(NamePathError),
    /// The resource is no stream a device sends (a method, a property, a sink, a trait).
    NotAStream { name: String },
    /// A series whose item type isn't numbers, or whose types the bundle can't tell.
    Series { name: String, why: anyhow::Error },
    /// A series whose item type reaches a definition the bundle left out: resolve the bundle first.
    SeriesUnresolved { name: String },
    /// A writer gives a dimension the stream's `#[track(dims(..))]` doesn't name.
    UndeclaredDimension { name: String, dimension: String },
}

impl core::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LayoutError::Path(e) => write!(f, "{e}"),
            LayoutError::NotAStream { name } => {
                write!(f, "`{name}` is no stream a device sends, it has no track")
            }
            LayoutError::Series { name, why } => {
                write!(f, "`{name}` is a series, but {why:#}")
            }
            LayoutError::SeriesUnresolved { name } => write!(
                f,
                "`{name}` is a series of a type the bundle left out: put its definitions back first"
            ),
            LayoutError::UndeclaredDimension { name, dimension } => write!(
                f,
                "dimension `{dimension}` is not in the `#[track(dims(..))]` of `{name}`"
            ),
        }
    }
}

impl core::error::Error for LayoutError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            LayoutError::Path(e) => Some(e),
            LayoutError::Series { why, .. } => Some(why.as_ref()),
            _ => None,
        }
    }
}

/// What a stream's items are on a tape: ww_tape's track kind, without what the writer gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamKind {
    /// An events track (`points`, `spans`).
    Events(EventsDef),
    /// A series of `channels` numbers per sample (`series`): the writer gives the period and scale of its
    /// `SeriesDef`.
    Series { channels: u32 },
}

/// How a recorder lays a stream out: its `#[track]` as the bundle has it, and the track kind it maps to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamLayout {
    /// The stream's name, as the bundle has it.
    pub name: String,
    /// The stream's `#[track]`, `points` without dims or flags where it has none.
    pub track: TrackOwned,
    /// The track kind of a tape.
    pub kind: StreamKind,
}

impl StreamLayout {
    /// The layout of the stream `item` of a bundle: its `#[track]`, and for a series the channels of its item type,
    /// whose definitions `bundle` holds.
    pub fn of(bundle: &ApiBundleOwned, item: &ApiItemOwned) -> Result<Self, LayoutError> {
        let name = item.ident.clone();
        let ty: &TypeOwned = match &item.kind {
            ApiItemKindOwned::Stream { ty, is_up: true }
            | ApiItemKindOwned::StreamFn { ty, .. } => ty,
            _ => return Err(LayoutError::NotAStream { name }),
        };
        let track = item
            .meta
            .as_ref()
            .and_then(|m| m.track.clone())
            .unwrap_or(TrackOwned {
                layout: TrackLayout::Points,
                dims: vec![],
                flags: false,
            });
        let events = |ends| {
            StreamKind::Events(EventsDef {
                ends,
                flags: track.flags,
            })
        };
        let kind = match track.layout {
            TrackLayout::Points => events(Ends::Instants),
            TrackLayout::Spans => events(Ends::Spans),
            TrackLayout::Series => match bundle.series_shape(ty) {
                Ok(SeriesShape::Channels(channels)) => StreamKind::Series { channels },
                Ok(SeriesShape::Unresolved) => return Err(LayoutError::SeriesUnresolved { name }),
                Err(why) => return Err(LayoutError::Series { name, why }),
            },
        };
        Ok(StreamLayout { name, track, kind })
    }

    /// Checks that a writer may give dimensions named `names`: each is in the stream's `#[track(dims(..))]`.
    pub fn check_dimensions<'a>(
        &self,
        names: impl IntoIterator<Item = &'a str>,
    ) -> Result<(), LayoutError> {
        match names.into_iter().find(|n| !self.track.allows_dimension(n)) {
            Some(dimension) => Err(LayoutError::UndeclaredDimension {
                name: self.name.clone(),
                dimension: dimension.to_owned(),
            }),
            None => Ok(()),
        }
    }
}

/// The layout of the stream at the numeric `path` in `bundle`, from its root (item ids and array indices, as a
/// track's path). See [stream_layout_at].
pub fn stream_layout(bundle: &ApiBundleOwned, path: &[u32]) -> Result<StreamLayout, LayoutError> {
    stream_layout_at(bundle, &bundle.root, path)
}

/// The layout of the stream at the numeric `path` from `level` of `bundle` (a trait's level for a path within it):
/// its `#[track]`, `points` without dimensions where it has none.
pub fn stream_layout_at(
    bundle: &ApiBundleOwned,
    level: &ApiLevelOwned,
    path: &[u32],
) -> Result<StreamLayout, LayoutError> {
    let item = item_at(bundle, level, path).map_err(LayoutError::Path)?;
    StreamLayout::of(bundle, &item)
}

/// The item at `path` from `level`, an array's index stepped over.
fn item_at(
    bundle: &ApiBundleOwned,
    level: &ApiLevelOwned,
    path: &[u32],
) -> Result<ApiItemOwned, NamePathError> {
    let mut level = level.clone();
    let mut steps = path.iter().copied().enumerate().peekable();
    while let Some((at, id)) = steps.next() {
        let item = level
            .items
            .iter()
            .find(|item| item.id.0 == id)
            .ok_or(NamePathError::NoItem { at, id })?
            .clone();
        if let Multiplicity::Array { .. } = item.multiplicity {
            steps
                .next()
                .ok_or(NamePathError::MissingIndex { at: path.len() })?;
        }
        if steps.peek().is_none() {
            return Ok(item);
        }
        level = match &item.kind {
            ApiItemKindOwned::Trait { trait_idx, .. } => bundle
                .get_trait(trait_idx.0)
                .map_err(NamePathError::Trait)?
                .clone(),
            ApiItemKindOwned::Keys { .. } => {
                keys_level(&level, &item).map_err(NamePathError::Keys)?
            }
            _ => {
                let at = steps.peek().map_or(path.len(), |&(at, _)| at);
                return Err(NamePathError::TooLong { at });
            }
        };
    }
    Err(NamePathError::Empty)
}
