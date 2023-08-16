// Unless explicitly stated otherwise all files in this repository are licensed under the Apache License Version 2.0.
// This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2021-Present Datadog, Inc.

use crate::profile::pprof;
use std::ops::{Add, Sub};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValueType<'a> {
    pub r#type: &'a str,
    pub unit: &'a str,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Period<'a> {
    pub r#type: ValueType<'a>,
    pub value: i64,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Mapping<'a> {
    /// Address at which the binary (or DLL) is loaded into memory.
    pub memory_start: u64,

    /// The limit of the address range occupied by this mapping.
    pub memory_limit: u64,

    /// Offset in the binary that corresponds to the first mapped address.
    pub file_offset: u64,

    /// The object this entry is loaded from.  This can be a filename on
    /// disk for the main binary and shared libraries, or virtual
    /// abstractions like "[vdso]".
    pub filename: &'a str,

    /// A string that uniquely identifies a particular program version
    /// with high probability. E.g., for binaries generated by GNU tools,
    /// it could be the contents of the .note.gnu.build-id field.
    pub build_id: &'a str,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Function<'a> {
    /// Name of the function, in human-readable form if available.
    pub name: &'a str,

    /// Name of the function, as identified by the system.
    /// For instance, it can be a C++ mangled name.
    pub system_name: &'a str,

    /// Source file containing the function.
    pub filename: &'a str,

    /// Line number in source file.
    pub start_line: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Line<'a> {
    /// The corresponding profile.Function for this line.
    pub function: Function<'a>,

    /// Line number in source code.
    pub line: i64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Location<'a> {
    pub mapping: Mapping<'a>,

    /// The instruction address for this location, if available.  It
    /// should be within [Mapping.memory_start...Mapping.memory_limit]
    /// for the corresponding mapping. A non-leaf address may be in the
    /// middle of a call instruction. It is up to display tools to find
    /// the beginning of the instruction if necessary.
    pub address: u64,

    /// Multiple line indicates this location has inlined functions,
    /// where the last entry represents the caller into which the
    /// preceding entries were inlined.
    ///
    /// E.g., if memcpy() is inlined into printf:
    ///    line[0].function_name == "memcpy"
    ///    line[1].function_name == "printf"
    pub lines: Vec<Line<'a>>,

    /// Provides an indication that multiple symbols map to this location's
    /// address, for example due to identical code folding by the linker. In that
    /// case the line information above represents one of the multiple
    /// symbols. This field must be recomputed when the symbolization state of the
    /// profile changes.
    pub is_folded: bool,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Label<'a> {
    pub key: &'a str,

    /// At most one of the following must be present
    pub str: Option<&'a str>,
    pub num: i64,

    /// Should only be present when num is present.
    /// Specifies the units of num.
    /// Use arbitrary string (for example, "requests") as a custom count unit.
    /// If no unit is specified, consumer may apply heuristic to deduce the unit.
    /// Consumers may also  interpret units like "bytes" and "kilobytes" as memory
    /// units and units like "seconds" and "nanoseconds" as time units,
    /// and apply appropriate unit conversions to these.
    pub num_unit: Option<&'a str>,
}

impl<'a> Label<'a> {
    pub fn uses_at_most_one_of_str_and_num(&self) -> bool {
        self.str.is_none() || (self.num == 0 && self.num_unit.is_none())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sample<'a> {
    /// The leaf is at locations[0].
    pub locations: Vec<Location<'a>>,

    /// The type and unit of each value is defined by the corresponding
    /// entry in Profile.sample_type. All samples must have the same
    /// number of values, the same as the length of Profile.sample_type.
    /// When aggregating multiple samples into a single sample, the
    /// result has a list of values that is the element-wise sum of the
    /// lists of the originals.
    pub values: Vec<i64>,

    /// label includes additional context for this sample. It can include
    /// things like a thread id, allocation size, etc
    pub labels: Vec<Label<'a>>,
}

pub enum UpscalingInfo {
    Poisson {
        // sum_value_offset and count_value_offset are offsets in the profile values type array
        sum_value_offset: usize,
        count_value_offset: usize,
        sampling_distance: u64,
    },
    Proportional {
        scale: f64,
    },
}

impl std::fmt::Display for UpscalingInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpscalingInfo::Poisson {
                sum_value_offset,
                count_value_offset,
                sampling_distance,
            } => write!(
                f,
                "Poisson = sum_value_offset: {}, count_value_offset: {}, sampling_distance: {}",
                sum_value_offset, count_value_offset, sampling_distance
            ),
            UpscalingInfo::Proportional { scale } => {
                write!(f, "Proportional = scale: {}", scale)
            }
        }
    }
}

impl UpscalingInfo {
    pub fn check_validity(&self, number_of_values: usize) -> anyhow::Result<()> {
        match self {
            UpscalingInfo::Poisson {
                sum_value_offset,
                count_value_offset,
                sampling_distance,
            } => {
                anyhow::ensure!(
                    sum_value_offset < &number_of_values && count_value_offset < &number_of_values,
                    "sum_value_offset {} and count_value_offset {} must be strictly less than {}",
                    sum_value_offset,
                    count_value_offset,
                    number_of_values
                );
                anyhow::ensure!(
                    sampling_distance != &0,
                    "sampling_distance {} must be greater than 0",
                    sampling_distance
                )
            }
            UpscalingInfo::Proportional { scale: _ } => (),
        }
        anyhow::Ok(())
    }
}

pub struct Profile<'a> {
    pub duration: Duration,
    pub period: Option<(i64, ValueType<'a>)>,
    pub sample_types: Vec<ValueType<'a>>,
    pub samples: Vec<Sample<'a>>,
    pub start_time: SystemTime,
}

fn string_table_fetch(pprof: &pprof::Profile, id: i64) -> anyhow::Result<&String> {
    pprof
        .string_table
        .get(id as u64 as usize)
        .ok_or_else(|| anyhow::anyhow!("String {id} was not found."))
}

fn mapping_fetch(pprof: &pprof::Profile, id: u64) -> anyhow::Result<Mapping> {
    if id == 0 {
        return Ok(Mapping::default());
    }

    match pprof.mappings.iter().find(|item| item.id == id) {
        Some(mapping) => Ok(Mapping {
            memory_start: mapping.memory_start,
            memory_limit: mapping.memory_limit,
            file_offset: mapping.file_offset,
            filename: string_table_fetch(pprof, mapping.filename)?,
            build_id: string_table_fetch(pprof, mapping.build_id)?,
        }),
        None => anyhow::bail!("Mapping {id} was not found."),
    }
}

fn function_fetch(pprof: &pprof::Profile, id: u64) -> anyhow::Result<Function> {
    if id == 0 {
        return Ok(Function::default());
    }

    match pprof.functions.iter().find(|item| item.id == id) {
        Some(function) => Ok(Function {
            name: string_table_fetch(pprof, function.name)?,
            system_name: string_table_fetch(pprof, function.system_name)?,
            filename: string_table_fetch(pprof, function.filename)?,
            start_line: function.start_line,
        }),
        None => anyhow::bail!("Function {id} was not found."),
    }
}

fn lines_fetch<'a>(
    pprof: &'a pprof::Profile,
    lines: &'a [pprof::Line],
) -> anyhow::Result<Vec<Line<'a>>> {
    let mut output = Vec::with_capacity(lines.len());
    for line in lines {
        output.push(Line {
            function: function_fetch(pprof, line.function_id)?,
            line: line.line,
        });
    }
    Ok(output)
}

fn location_fetch(pprof: &pprof::Profile, id: u64) -> anyhow::Result<Location> {
    if id == 0 {
        return Ok(Location::default());
    }

    match pprof.locations.iter().find(|item| item.id == id) {
        Some(location) => Ok(Location {
            mapping: mapping_fetch(pprof, location.mapping_id)?,
            address: location.address,
            lines: lines_fetch(pprof, &location.lines)?,
            is_folded: location.is_folded,
        }),
        None => anyhow::bail!("Location {id} was not found."),
    }
}

fn locations_fetch<'a>(
    pprof: &'a pprof::Profile,
    ids: &'a [u64],
) -> anyhow::Result<Vec<Location<'a>>> {
    let mut locations = Vec::with_capacity(ids.len());
    for id in ids {
        let location = location_fetch(pprof, *id)?;
        locations.push(location);
    }
    Ok(locations)
}

impl<'a> TryFrom<&'a pprof::Profile> for Profile<'a> {
    type Error = anyhow::Error;

    fn try_from(pprof: &'a pprof::Profile) -> Result<Self, Self::Error> {
        assert!(pprof.duration_nanos >= 0);
        let duration = Duration::from_nanos(pprof.duration_nanos as u64);
        let start_time = if pprof.time_nanos.is_negative() {
            UNIX_EPOCH.sub(Duration::from_nanos(pprof.time_nanos.unsigned_abs()))
        } else {
            UNIX_EPOCH.add(Duration::from_nanos(pprof.time_nanos as u64))
        };

        let period = match pprof.period_type {
            Some(t) => {
                let r#type = ValueType {
                    r#type: string_table_fetch(pprof, t.r#type)?,
                    unit: string_table_fetch(pprof, t.unit)?,
                };
                Some((pprof.period, r#type))
            }
            None => None,
        };

        let mut sample_types = Vec::with_capacity(pprof.samples.len());
        for t in pprof.sample_types.iter() {
            sample_types.push(ValueType {
                r#type: string_table_fetch(pprof, t.r#type)?,
                unit: string_table_fetch(pprof, t.unit)?,
            });
        }

        let mut samples = Vec::with_capacity(pprof.samples.len());
        for sample in pprof.samples.iter() {
            let locations = locations_fetch(pprof, &sample.location_ids)?;

            let mut labels = Vec::with_capacity(sample.labels.len());
            for label in sample.labels.iter() {
                labels.push(Label {
                    key: string_table_fetch(pprof, label.key)?,
                    str: if label.str == 0 {
                        None
                    } else {
                        Some(string_table_fetch(pprof, label.str)?)
                    },
                    num: label.num,
                    num_unit: if label.num_unit == 0 {
                        None
                    } else {
                        Some(string_table_fetch(pprof, label.num_unit)?)
                    },
                })
            }
            let sample = Sample {
                locations,
                values: sample.values.clone(),
                labels,
            };
            samples.push(sample);
        }

        Ok(Profile {
            duration,
            period,
            sample_types,
            samples,
            start_time,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn label_uses_at_most_one_of_str_and_num() {
        let label = Label {
            key: "name",
            str: Some("levi"),
            num: 0,
            num_unit: Some("name"), // can't use num_unit with str
        };
        assert!(!label.uses_at_most_one_of_str_and_num());

        let label = Label {
            key: "name",
            str: Some("levi"),
            num: 10, // can't use num with str
            num_unit: None,
        };
        assert!(!label.uses_at_most_one_of_str_and_num());

        let label = Label {
            key: "name",
            str: Some("levi"),
            num: 0,
            num_unit: None,
        };
        assert!(label.uses_at_most_one_of_str_and_num());

        let label = Label {
            key: "process_id",
            str: None,
            num: 0,
            num_unit: None,
        };
        assert!(label.uses_at_most_one_of_str_and_num());

        let label = Label {
            key: "local root span id",
            str: None,
            num: 10901,
            num_unit: None,
        };
        assert!(label.uses_at_most_one_of_str_and_num());

        let label = Label {
            key: "duration",
            str: None,
            num: 12345,
            num_unit: Some("nanoseconds"),
        };
        assert!(label.uses_at_most_one_of_str_and_num());
    }
}
