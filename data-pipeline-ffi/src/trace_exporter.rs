// Copyright 2024-Present Datadog, Inc. https://www.datadoghq.com/
// SPDX-License-Identifier: Apache-2.0

use crate::error::{ExporterError, ExporterErrorCode as ErrorCode};
use data_pipeline::trace_exporter::agent_response::AgentResponse;
use data_pipeline::trace_exporter::{
    TraceExporter, TraceExporterInputFormat, TraceExporterOutputFormat,
};
use ddcommon_ffi::{
    CharSlice,
    {slice::AsBytes, slice::ByteSlice},
};
use std::{ptr::NonNull, time::Duration};

macro_rules! gen_error {
    ($l:expr) => {
        Some(Box::new(ExporterError::new($l, &$l.to_string())))
    };
}

#[inline]
fn sanitize_string(str: CharSlice) -> Result<String, Box<ExporterError>> {
    match str.try_to_utf8() {
        Ok(s) => Ok(s.to_string()),
        Err(_) => Err(Box::new(ExporterError::new(
            ErrorCode::InvalidInput,
            &ErrorCode::InvalidInput.to_string(),
        ))),
    }
}

/// The TraceExporterConfig object will hold the configuration properties for the TraceExporter.
/// Once the configuration is passed to the TraceExporter constructor the config is no longer
/// needed by the handle and it can be freed.
#[derive(Debug, Default, PartialEq)]
pub struct TraceExporterConfig {
    url: Option<String>,
    tracer_version: Option<String>,
    language: Option<String>,
    language_version: Option<String>,
    language_interpreter: Option<String>,
    hostname: Option<String>,
    env: Option<String>,
    version: Option<String>,
    service: Option<String>,
    input_format: TraceExporterInputFormat,
    output_format: TraceExporterOutputFormat,
    compute_stats: bool,
}

#[no_mangle]
pub unsafe extern "C" fn ddog_trace_exporter_config_new(
    out_handle: NonNull<Box<TraceExporterConfig>>,
) {
    out_handle
        .as_ptr()
        .write(Box::<TraceExporterConfig>::default());
}

/// Frees TraceExporterConfig handle internal resources.
#[no_mangle]
pub unsafe extern "C" fn ddog_trace_exporter_config_free(handle: Box<TraceExporterConfig>) {
    drop(handle);
}

/// Sets traces destination.
#[no_mangle]
pub unsafe extern "C" fn ddog_trace_exporter_config_set_url(
    config: Option<&mut TraceExporterConfig>,
    url: CharSlice,
) -> Option<Box<ExporterError>> {
    if let Some(handle) = config {
        handle.url = match sanitize_string(url) {
            Ok(s) => Some(s),
            Err(e) => return Some(e),
        };
        None
    } else {
        gen_error!(ErrorCode::InvalidArgument)
    }
}

/// Sets tracer's version to be included in the headers request.
#[no_mangle]
pub unsafe extern "C" fn ddog_trace_exporter_config_set_tracer_version(
    config: Option<&mut TraceExporterConfig>,
    version: CharSlice,
) -> Option<Box<ExporterError>> {
    if let Option::Some(handle) = config {
        handle.tracer_version = match sanitize_string(version) {
            Ok(s) => Some(s),
            Err(e) => return Some(e),
        };
        None
    } else {
        gen_error!(ErrorCode::InvalidArgument)
    }
}

/// Sets tracer's language to be included in the headers request.
#[no_mangle]
pub unsafe extern "C" fn ddog_trace_exporter_config_set_language(
    config: Option<&mut TraceExporterConfig>,
    lang: CharSlice,
) -> Option<Box<ExporterError>> {
    if let Option::Some(handle) = config {
        handle.language = match sanitize_string(lang) {
            Ok(s) => Some(s),
            Err(e) => return Some(e),
        };
        None
    } else {
        gen_error!(ErrorCode::InvalidArgument)
    }
}

/// Sets tracer's language version to be included in the headers request.
#[no_mangle]
pub unsafe extern "C" fn ddog_trace_exporter_config_set_lang_version(
    config: Option<&mut TraceExporterConfig>,
    version: CharSlice,
) -> Option<Box<ExporterError>> {
    if let Option::Some(handle) = config {
        handle.language_version = match sanitize_string(version) {
            Ok(s) => Some(s),
            Err(e) => return Some(e),
        };
        None
    } else {
        gen_error!(ErrorCode::InvalidArgument)
    }
}

/// Sets tracer's language interpreter to be included in the headers request.
#[no_mangle]
pub unsafe extern "C" fn ddog_trace_exporter_config_set_lang_interpreter(
    config: Option<&mut TraceExporterConfig>,
    interpreter: CharSlice,
) -> Option<Box<ExporterError>> {
    if let Option::Some(handle) = config {
        handle.language_interpreter = match sanitize_string(interpreter) {
            Ok(s) => Some(s),
            Err(e) => return Some(e),
        };
        None
    } else {
        gen_error!(ErrorCode::InvalidArgument)
    }
}

/// Sets hostname information to be included in the headers request.
#[no_mangle]
pub unsafe extern "C" fn ddog_trace_exporter_config_set_hostname(
    config: Option<&mut TraceExporterConfig>,
    hostname: CharSlice,
) -> Option<Box<ExporterError>> {
    if let Option::Some(handle) = config {
        handle.hostname = match sanitize_string(hostname) {
            Ok(s) => Some(s),
            Err(e) => return Some(e),
        };
        None
    } else {
        gen_error!(ErrorCode::InvalidArgument)
    }
}

/// Sets environmet information to be included in the headers request.
#[no_mangle]
pub unsafe extern "C" fn ddog_trace_exporter_config_set_env(
    config: Option<&mut TraceExporterConfig>,
    env: CharSlice,
) -> Option<Box<ExporterError>> {
    if let Option::Some(handle) = config {
        handle.env = match sanitize_string(env) {
            Ok(s) => Some(s),
            Err(e) => return Some(e),
        };
        None
    } else {
        gen_error!(ErrorCode::InvalidArgument)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ddog_trace_exporter_config_set_version(
    config: Option<&mut TraceExporterConfig>,
    version: CharSlice,
) -> Option<Box<ExporterError>> {
    if let Option::Some(handle) = config {
        handle.version = match sanitize_string(version) {
            Ok(s) => Some(s),
            Err(e) => return Some(e),
        };
        None
    } else {
        gen_error!(ErrorCode::InvalidArgument)
    }
}

/// Sets service name to be included in the headers request.
#[no_mangle]
pub unsafe extern "C" fn ddog_trace_exporter_config_set_service(
    config: Option<&mut TraceExporterConfig>,
    service: CharSlice,
) -> Option<Box<ExporterError>> {
    if let Option::Some(handle) = config {
        handle.service = match sanitize_string(service) {
            Ok(s) => Some(s),
            Err(e) => return Some(e),
        };
        None
    } else {
        gen_error!(ErrorCode::InvalidArgument)
    }
}

/// Create a new TraceExporter instance.
///
/// # Arguments
///
/// * `out_handle` - The handle to write the TraceExporter instance in.
/// * `config` - The configuration used to set up the TraceExporter handle.
#[no_mangle]
pub unsafe extern "C" fn ddog_trace_exporter_new(
    out_handle: NonNull<Box<TraceExporter>>,
    config: Option<&TraceExporterConfig>,
) -> Option<Box<ExporterError>> {
    if let Some(config) = config {
        // let config = &*ptr;
        let mut builder = TraceExporter::builder()
            .set_url(config.url.as_ref().unwrap_or(&"".to_string()))
            .set_tracer_version(config.tracer_version.as_ref().unwrap_or(&"".to_string()))
            .set_language(config.language.as_ref().unwrap_or(&"".to_string()))
            .set_language_version(config.language_version.as_ref().unwrap_or(&"".to_string()))
            .set_language_interpreter(
                config
                    .language_interpreter
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .set_hostname(config.hostname.as_ref().unwrap_or(&"".to_string()))
            .set_env(config.env.as_ref().unwrap_or(&"".to_string()))
            .set_app_version(config.version.as_ref().unwrap_or(&"".to_string()))
            .set_service(config.service.as_ref().unwrap_or(&"".to_string()))
            .set_input_format(config.input_format)
            .set_output_format(config.output_format);
        if config.compute_stats {
            builder = builder.enable_stats(Duration::from_secs(10))
            // TODO: APMSP-1317 Enable peer tags aggregation and stats by span_kind based on agent
            // configuration
        }

        match builder.build() {
            Ok(exporter) => {
                out_handle.as_ptr().write(Box::new(exporter));
                None
            }
            Err(err) => Some(Box::new(ExporterError::from(err))),
        }
    } else {
        gen_error!(ErrorCode::InvalidArgument)
    }
}

/// Free the TraceExporter instance.
///
/// # Arguments
///
/// * handle - The handle to the TraceExporter instance.
#[no_mangle]
pub unsafe extern "C" fn ddog_trace_exporter_free(handle: Box<TraceExporter>) {
    drop(handle);
}

/// Send traces to the Datadog Agent.
///
/// # Arguments
///
/// * `handle` - The handle to the TraceExporter instance.
/// * `trace` - The traces to send to the Datadog Agent in the input format used to create the
///   TraceExporter. The memory for the trace must be valid for the life of the call to this
///   function.
/// * `trace_count` - The number of traces to send to the Datadog Agent.
/// * `response` - Optional parameter that will ontain the agent response information.
#[no_mangle]
pub unsafe extern "C" fn ddog_trace_exporter_send(
    handle: Option<&TraceExporter>,
    trace: ByteSlice,
    trace_count: usize,
    response: Option<&mut AgentResponse>,
) -> Option<Box<ExporterError>> {
    let exporter = match handle {
        Some(exp) => exp,
        None => return gen_error!(ErrorCode::InvalidArgument),
    };

    // necessary that the trace be static for the life of the FFI function call as the caller
    // currently owns the memory.
    //APMSP-1621 - Properly fix this sharp-edge by allocating memory on the Rust side
    let static_trace: ByteSlice<'static> = std::mem::transmute(trace);
    match exporter.send(
        tinybytes::Bytes::from_static(static_trace.as_slice()),
        trace_count,
    ) {
        Ok(resp) => {
            if let Some(result) = response {
                *result = resp;
            }
            None
        }
        Err(e) => Some(Box::new(ExporterError::from(e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ddog_trace_exporter_error_free;
    use crate::trace_exporter::AgentResponse;
    use std::{borrow::Borrow, mem::MaybeUninit};

    #[test]
    fn config_constructor_test() {
        unsafe {
            let mut config: MaybeUninit<Box<TraceExporterConfig>> = MaybeUninit::uninit();

            ddog_trace_exporter_config_new(NonNull::new_unchecked(&mut config).cast());

            let cfg = config.assume_init();
            assert_eq!(cfg.url, None);
            assert_eq!(cfg.tracer_version, None);
            assert_eq!(cfg.language, None);
            assert_eq!(cfg.language_version, None);
            assert_eq!(cfg.language_interpreter, None);
            assert_eq!(cfg.env, None);
            assert_eq!(cfg.hostname, None);
            assert_eq!(cfg.version, None);
            assert_eq!(cfg.service, None);
            assert_eq!(cfg.input_format, TraceExporterInputFormat::V04);
            assert_eq!(cfg.output_format, TraceExporterOutputFormat::V04);
            assert!(!cfg.compute_stats);

            ddog_trace_exporter_config_free(cfg);
        }
    }

    #[test]
    fn config_url_test() {
        unsafe {
            let error =
                ddog_trace_exporter_config_set_url(None, CharSlice::from("http://localhost"));
            assert_eq!(error.as_ref().unwrap().code, ErrorCode::InvalidArgument);

            ddog_trace_exporter_error_free(error);

            let mut config = Some(TraceExporterConfig::default());
            let error = ddog_trace_exporter_config_set_url(
                config.as_mut(),
                CharSlice::from("http://localhost"),
            );

            assert_eq!(error, None);

            let cfg = config.unwrap();
            assert_eq!(cfg.url.as_ref().unwrap(), "http://localhost");
        }
    }

    #[test]
    fn config_tracer_version() {
        unsafe {
            let error = ddog_trace_exporter_config_set_tracer_version(None, CharSlice::from("1.0"));
            assert_eq!(error.as_ref().unwrap().code, ErrorCode::InvalidArgument);

            ddog_trace_exporter_error_free(error);

            let mut config = Some(TraceExporterConfig::default());
            let error = ddog_trace_exporter_config_set_tracer_version(
                config.as_mut(),
                CharSlice::from("1.0"),
            );
            assert_eq!(error, None);

            let cfg = config.unwrap();
            assert_eq!(cfg.tracer_version.as_ref().unwrap(), "1.0");
        }
    }

    #[test]
    fn config_language() {
        unsafe {
            let error = ddog_trace_exporter_config_set_language(None, CharSlice::from("lang"));
            assert_eq!(error.as_ref().unwrap().code, ErrorCode::InvalidArgument);

            ddog_trace_exporter_error_free(error);

            let mut config = Some(TraceExporterConfig::default());
            let error =
                ddog_trace_exporter_config_set_language(config.as_mut(), CharSlice::from("lang"));

            assert_eq!(error, None);

            let cfg = config.unwrap();
            assert_eq!(cfg.language.as_ref().unwrap(), "lang");
        }
    }

    #[test]
    fn config_lang_version() {
        unsafe {
            let error = ddog_trace_exporter_config_set_lang_version(None, CharSlice::from("0.1"));
            assert_eq!(error.as_ref().unwrap().code, ErrorCode::InvalidArgument);

            ddog_trace_exporter_error_free(error);

            let mut config = Some(TraceExporterConfig::default());
            let error = ddog_trace_exporter_config_set_lang_version(
                config.as_mut(),
                CharSlice::from("0.1"),
            );

            assert_eq!(error, None);

            let cfg = config.unwrap();
            assert_eq!(cfg.language_version.as_ref().unwrap(), "0.1");
        }
    }

    #[test]
    fn config_lang_interpreter_test() {
        unsafe {
            let error =
                ddog_trace_exporter_config_set_lang_interpreter(None, CharSlice::from("foo"));
            assert_eq!(error.as_ref().unwrap().code, ErrorCode::InvalidArgument);

            ddog_trace_exporter_error_free(error);

            let mut config = Some(TraceExporterConfig::default());
            let error = ddog_trace_exporter_config_set_lang_interpreter(
                config.as_mut(),
                CharSlice::from("foo"),
            );

            assert_eq!(error, None);

            let cfg = config.unwrap();
            assert_eq!(cfg.language_interpreter.as_ref().unwrap(), "foo");
        }
    }

    #[test]
    fn config_hostname_test() {
        unsafe {
            let error = ddog_trace_exporter_config_set_hostname(None, CharSlice::from("hostname"));
            assert_eq!(error.as_ref().unwrap().code, ErrorCode::InvalidArgument);

            ddog_trace_exporter_error_free(error);

            let mut config = Some(TraceExporterConfig::default());
            let error = ddog_trace_exporter_config_set_hostname(
                config.as_mut(),
                CharSlice::from("hostname"),
            );

            assert_eq!(error, None);

            let cfg = config.unwrap();
            assert_eq!(cfg.hostname.as_ref().unwrap(), "hostname");
        }
    }

    #[test]
    fn config_env_test() {
        unsafe {
            let error = ddog_trace_exporter_config_set_env(None, CharSlice::from("env-test"));
            assert_eq!(error.as_ref().unwrap().code, ErrorCode::InvalidArgument);

            ddog_trace_exporter_error_free(error);

            let mut config = Some(TraceExporterConfig::default());
            let error =
                ddog_trace_exporter_config_set_env(config.as_mut(), CharSlice::from("env-test"));

            assert_eq!(error, None);

            let cfg = config.unwrap();
            assert_eq!(cfg.env.as_ref().unwrap(), "env-test");
        }
    }

    #[test]
    fn config_version_test() {
        unsafe {
            let error = ddog_trace_exporter_config_set_version(None, CharSlice::from("1.2"));
            assert_eq!(error.as_ref().unwrap().code, ErrorCode::InvalidArgument);

            ddog_trace_exporter_error_free(error);

            let mut config = Some(TraceExporterConfig::default());
            let error =
                ddog_trace_exporter_config_set_version(config.as_mut(), CharSlice::from("1.2"));

            assert_eq!(error, None);

            let cfg = config.unwrap();
            assert_eq!(cfg.version.as_ref().unwrap(), "1.2");
        }
    }

    #[test]
    fn config_service_test() {
        unsafe {
            let error = ddog_trace_exporter_config_set_service(None, CharSlice::from("service"));
            assert_eq!(error.as_ref().unwrap().code, ErrorCode::InvalidArgument);

            ddog_trace_exporter_error_free(error);

            let mut config = Some(TraceExporterConfig::default());
            let error =
                ddog_trace_exporter_config_set_service(config.as_mut(), CharSlice::from("service"));

            assert_eq!(error, None);

            let cfg = config.unwrap();
            assert_eq!(cfg.service.as_ref().unwrap(), "service");
        }
    }

    #[test]
    fn expoter_constructor_test() {
        unsafe {
            let mut config: MaybeUninit<Box<TraceExporterConfig>> = MaybeUninit::uninit();
            ddog_trace_exporter_config_new(NonNull::new_unchecked(&mut config).cast());

            let mut cfg = config.assume_init();
            let error = ddog_trace_exporter_config_set_url(
                Some(cfg.as_mut()),
                CharSlice::from("http://localhost"),
            );
            assert_eq!(error, None);

            let mut ptr: MaybeUninit<Box<TraceExporter>> = MaybeUninit::uninit();

            let ret = ddog_trace_exporter_new(
                NonNull::new_unchecked(&mut ptr).cast(),
                Some(cfg.borrow()),
            );
            let exporter = ptr.assume_init();

            assert_eq!(ret, None);

            ddog_trace_exporter_free(exporter);
            ddog_trace_exporter_config_free(cfg);
        }
    }

    #[test]
    fn expoter_constructor_error_test() {
        unsafe {
            let mut config: MaybeUninit<Box<TraceExporterConfig>> = MaybeUninit::uninit();
            ddog_trace_exporter_config_new(NonNull::new_unchecked(&mut config).cast());

            let mut cfg = config.assume_init();
            let error = ddog_trace_exporter_config_set_service(
                Some(cfg.as_mut()),
                CharSlice::from("service"),
            );
            assert_eq!(error, None);

            ddog_trace_exporter_error_free(error);

            let mut ptr: MaybeUninit<Box<TraceExporter>> = MaybeUninit::uninit();

            let ret = ddog_trace_exporter_new(NonNull::new_unchecked(&mut ptr).cast(), Some(&cfg));

            let error = ret.as_ref().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidUrl);

            ddog_trace_exporter_error_free(ret);

            ddog_trace_exporter_config_free(cfg);
        }
    }

    #[test]
    fn exporter_send_test_arguments_test() {
        unsafe {
            let trace = ByteSlice::from(b"dummy contents" as &[u8]);
            let mut resp = AgentResponse { rate: 0.0 };
            let ret = ddog_trace_exporter_send(None, trace, 0, Some(&mut resp));

            assert!(ret.is_some());
            assert_eq!(ret.unwrap().code, ErrorCode::InvalidArgument);
        }
    }

    #[test]
    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    // TODO(APMSP-1632): investigate why test fails on ARM platforms due to a CharSlice constructor.
    fn config_invalid_input_test() {
        unsafe {
            let mut config = Some(TraceExporterConfig::default());
            let invalid: [i8; 2] = [0x80u8 as i8, 0xFFu8 as i8];
            let error =
                ddog_trace_exporter_config_set_service(config.as_mut(), CharSlice::new(&invalid));

            assert_eq!(error.unwrap().code, ErrorCode::InvalidInput);
        }
    }
}
