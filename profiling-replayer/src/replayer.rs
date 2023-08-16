// Unless explicitly stated otherwise all files in this repository are licensed under the Apache License Version 2.0.
// This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2023-Present Datadog, Inc.

use crate::profile_index::ProfileIndex;
use datadog_profiling::profile::{api, pprof};
use std::ops::{Add, Sub};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct Replayer<'pprof> {
    pub profile_index: ProfileIndex<'pprof>,

    pub start_time: SystemTime,
    pub duration: Duration,
    pub end_time: SystemTime, // start_time + duration
    pub sample_types: Vec<api::ValueType<'pprof>>,
    pub period: Option<api::Period<'pprof>>,
    pub endpoints: Vec<(u64, &'pprof str)>,
    pub samples: Vec<api::Sample<'pprof>>,
}

impl<'pprof> Replayer<'pprof> {
    fn system_time_add(system_time: SystemTime, ns: i64) -> SystemTime {
        if ns < 0 {
            let u64 = ns.unsigned_abs();
            system_time.sub(Duration::from_nanos(u64))
        } else {
            let u64 = ns as u64;
            system_time.add(Duration::from_nanos(u64))
        }
    }

    fn start_time(pprof: &pprof::Profile) -> SystemTime {
        Self::system_time_add(UNIX_EPOCH, pprof.time_nanos)
    }

    fn duration(pprof: &pprof::Profile) -> anyhow::Result<Duration> {
        match u64::try_from(pprof.duration_nanos) {
            Ok(nanos) => Ok(Duration::from_nanos(nanos)),
            Err(_err) => anyhow::bail!(
                "duration of pprof didn't fit in u64: {}",
                pprof.duration_nanos
            ),
        }
    }

    fn sample_types<'a>(
        profile_index: &'a ProfileIndex<'pprof>,
    ) -> anyhow::Result<Vec<api::ValueType<'pprof>>> {
        let mut sample_types = Vec::with_capacity(profile_index.pprof.sample_types.len());
        for sample_type in profile_index.pprof.sample_types.iter() {
            sample_types.push(api::ValueType {
                r#type: profile_index.get_string(sample_type.r#type)?,
                unit: profile_index.get_string(sample_type.unit)?,
            })
        }
        Ok(sample_types)
    }

    fn period<'a>(
        profile_index: &'a ProfileIndex<'pprof>,
    ) -> anyhow::Result<Option<api::Period<'pprof>>> {
        let value = profile_index.pprof.period;

        match profile_index.pprof.period_type {
            Some(period_type) => {
                let r#type = api::ValueType {
                    r#type: profile_index.get_string(period_type.r#type)?,
                    unit: profile_index.get_string(period_type.unit)?,
                };
                Ok(Some(api::Period { r#type, value }))
            }
            None => Ok(None),
        }
    }

    fn sample_labels<'a>(
        profile_index: &'a ProfileIndex<'pprof>,
        sample: &'pprof pprof::Sample,
    ) -> anyhow::Result<(Vec<api::Label<'pprof>>, Option<(u64, &'pprof str)>)> {
        let mut labels = Vec::with_capacity(sample.labels.len());
        for label in sample.labels.iter() {
            labels.push(api::Label {
                key: profile_index.get_string(label.key)?,
                str: if label.str == 0 {
                    None
                } else {
                    Some(profile_index.get_string(label.str)?)
                },
                num: label.num,
                num_unit: if label.num_unit == 0 {
                    None
                } else {
                    Some(profile_index.get_string(label.num_unit)?)
                },
            })
        }
        let lrsi = labels
            .iter()
            .find(|label| label.key == "local root span id");

        let endpoint = labels.iter().find(|label| label.key == "trace endpoint");

        let mut endpoint_info = None;
        if let (Some(lsri_label), Some(endpoint_label)) = (lrsi, endpoint) {
            let num: i64 = lsri_label.num;
            let local_root_span_id: u64 = unsafe { std::mem::transmute(num) };
            anyhow::ensure!(
                local_root_span_id != 0,
                "local root span ids of zero do not make sense"
            );

            let endpoint_value = match endpoint_label.str {
                Some(v) => v,
                None => anyhow::bail!("expected trace endpoint label value to have a string"),
            };

            endpoint_info.replace((local_root_span_id, endpoint_value));
        }

        // Remove all labels except "trace endpoint"
        labels.retain(|label| label.key != "trace endpoint");

        Ok((labels, endpoint_info))
    }

    fn get_mapping<'a>(
        profile_index: &'a ProfileIndex<'pprof>,
        id: u64,
    ) -> anyhow::Result<api::Mapping<'pprof>> {
        let mapping = profile_index.get_mapping(id)?;
        Ok(api::Mapping {
            memory_start: mapping.memory_start,
            memory_limit: mapping.memory_limit,
            file_offset: mapping.file_offset,
            filename: profile_index.get_string(mapping.filename)?,
            build_id: profile_index.get_string(mapping.build_id)?,
        })
    }

    fn get_line<'a>(
        profile_index: &'a ProfileIndex<'pprof>,
        line: &pprof::Line,
    ) -> anyhow::Result<api::Line<'pprof>> {
        Ok(api::Line {
            function: Self::get_function(profile_index, line.function_id)?,
            line: line.line,
        })
    }

    fn get_location<'a>(
        profile_index: &'a ProfileIndex<'pprof>,
        id: u64,
    ) -> anyhow::Result<api::Location<'pprof>> {
        let location = profile_index.get_location(id)?;
        let mapping = Self::get_mapping(profile_index, location.mapping_id)?;
        let mut lines = Vec::with_capacity(location.lines.len());
        for line in location.lines.iter() {
            lines.push(Self::get_line(profile_index, line)?);
        }
        Ok(api::Location {
            mapping,
            address: location.address,
            lines,
            is_folded: location.is_folded,
        })
    }

    fn get_function<'a>(
        profile_index: &'a ProfileIndex<'pprof>,
        id: u64,
    ) -> anyhow::Result<api::Function<'pprof>> {
        let function = profile_index.get_function(id)?;
        Ok(api::Function {
            name: profile_index.get_string(function.name)?,
            system_name: profile_index.get_string(function.system_name)?,
            filename: profile_index.get_string(function.filename)?,
            start_line: function.start_line,
        })
    }

    fn sample_locations<'a>(
        profile_index: &'a ProfileIndex<'pprof>,
        sample: &pprof::Sample,
    ) -> anyhow::Result<Vec<api::Location<'pprof>>> {
        let mut locations = Vec::with_capacity(sample.location_ids.len());
        for location_id in sample.location_ids.iter() {
            locations.push(Self::get_location(profile_index, *location_id)?);
        }
        Ok(locations)
    }

    fn samples<'a>(
        profile_index: &'a ProfileIndex<'pprof>,
    ) -> anyhow::Result<(Vec<api::Sample<'pprof>>, Vec<(u64, &'pprof str)>)> {
        // Find the "local root span id" and "trace endpoint" labels. If
        // they are found, then save them into a vec to replay later, and
        // drop the "trace endpoint" label from sample.
        let mut endpoints = Vec::with_capacity(1);
        let mut samples = Vec::with_capacity(profile_index.pprof.samples.len());

        for sample in profile_index.pprof.samples.iter() {
            let (labels, endpoint) = Self::sample_labels(profile_index, sample)?;
            samples.push(api::Sample {
                locations: Self::sample_locations(profile_index, sample)?,
                values: sample.values.clone(),
                labels,
            });
            if let Some(endpoint_info) = endpoint {
                endpoints.push(endpoint_info)
            }
        }

        Ok((samples, endpoints))
    }
}

impl<'pprof> TryFrom<&'pprof pprof::Profile> for Replayer<'pprof> {
    type Error = anyhow::Error;

    fn try_from(pprof: &'pprof pprof::Profile) -> Result<Self, Self::Error> {
        let profile_index = ProfileIndex::try_from(pprof)?;

        let start_time = Self::start_time(pprof);
        let duration = Self::duration(pprof)?;
        let end_time = start_time.add(duration);
        let sample_types = Self::sample_types(&profile_index)?;
        let period = Self::period(&profile_index)?;
        let (samples, endpoints) = Self::samples(&profile_index)?;

        Ok(Self {
            profile_index,
            start_time,
            duration,
            end_time,
            sample_types,
            period,
            endpoints,
            samples,
        })
    }
}
