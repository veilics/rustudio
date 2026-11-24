mod error;
mod header;
mod state;

use std::{io::Read, str};

use rbx_dom_weak::WeakDom;
use rbx_reflection::ReflectionDatabase;

use self::state::DeserializerState;

#[cfg(any(test, feature = "unstable_text_format"))]
pub(crate) use self::header::FileHeader;

pub use self::error::Error;

/// A configurable deserializer for Roblox binary models and places.
///
/// ## Example
/// ```no_run
/// use std::fs::File;
/// use std::io::BufReader;
///
/// use rbx_binary::Deserializer;
///
/// let input = BufReader::new(File::open("File.rbxm")?);
///
/// let deserializer = Deserializer::new();
/// let dom = deserializer.deserialize(input)?;
///
/// // rbx_binary always returns a DOM with a DataModel at the top level.
/// // To get to the instances from our file, we need to go one level deeper.
///
/// println!("Root instances in file:");
/// for &referent in dom.root().children() {
///     let instance = dom.get_by_ref(referent).unwrap();
///     println!("- {}", instance.name);
/// }
///
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Configuration
///
/// A custom [`ReflectionDatabase`][ReflectionDatabase] can be specified via
/// [`reflection_database`][reflection_database].
///
/// [ReflectionDatabase]: rbx_reflection::ReflectionDatabase
/// [reflection_database]: Deserializer#method.reflection_database
pub struct Deserializer<'db> {
    database: &'db ReflectionDatabase<'db>,
    /// rustudio: (class, canonical property) → a value not to store, because
    /// the reader treats a missing property as that value.
    skip_defaults: Option<&'db dyn Fn(&str, &str) -> Option<rbx_dom_weak::types::Variant>>,
}

impl<'db> Deserializer<'db> {
    /// Create a new `Deserializer` with the default settings.
    pub fn new() -> Self {
        Self {
            database: rbx_reflection_database::get().unwrap(),
            skip_defaults: None,
        }
    }

    /// Sets what reflection database for the deserializer to use.
    #[inline]
    pub fn reflection_database(self, database: &'db ReflectionDatabase<'db>) -> Self {
        Self { database, ..self }
    }

    /// rustudio: leave out property values the reader treats as implied.
    /// `skip(class, property)` gives the value a missing property reads as
    /// (None: always store). Asked once per class and property; only
    /// unmigrated properties. A big place is mostly defaults: a 44 MB
    /// place's 25.6 M stored values were 69% defaults (4.8 GB parsed).
    #[inline]
    pub fn skip_defaults(
        self,
        skip: &'db dyn Fn(&str, &str) -> Option<rbx_dom_weak::types::Variant>,
    ) -> Self {
        Self { skip_defaults: Some(skip), ..self }
    }

    /// Deserialize a Roblox binary model or place from the given stream using
    /// this deserializer.
    pub fn deserialize<R: Read>(&self, reader: R) -> Result<WeakDom, Error> {
        profiling::scope!("rbx_binary::deserialize");

        let mut deserializer = DeserializerState::new(self, reader)?;

        loop {
            let chunk = deserializer.next_chunk()?;

            match &chunk.name {
                b"META" => deserializer.decode_meta_chunk(&chunk.data)?,
                b"SSTR" => deserializer.decode_sstr_chunk(&chunk.data)?,
                b"INST" => deserializer.decode_inst_chunk(&chunk.data)?,
                b"PROP" => deserializer.decode_prop_chunk(&chunk.data)?,
                b"PRNT" => deserializer.decode_prnt_chunk(&chunk.data)?,
                b"END\0" => {
                    deserializer.decode_end_chunk(&chunk.data)?;
                    break;
                }
                _ => match str::from_utf8(&chunk.name) {
                    Ok(name) => log::info!("Unknown binary chunk name {name}"),
                    Err(_) => log::info!("Unknown binary chunk name {:?}", chunk.name),
                },
            }
        }

        Ok(deserializer.finish())
    }
}

impl Default for Deserializer<'_> {
    fn default() -> Self {
        Self::new()
    }
}
